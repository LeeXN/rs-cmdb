use anyhow::Result;
use axum::serve::Listener;
use rcgen::{CertificateParams, KeyPair, DistinguishedName};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};

fn default_cert_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".rs-cmdb")
}

fn default_cert_path() -> PathBuf {
    default_cert_dir().join("server.crt")
}

fn default_key_path() -> PathBuf {
    default_cert_dir().join("server.key")
}

pub fn generate_self_signed_cert() -> Result<(String, String)> {
    let cert_dir = default_cert_dir();
    fs::create_dir_all(&cert_dir)?;

    let mut params = CertificateParams::new(vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
    ])?;
    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(
        rcgen::DnType::CommonName,
        "rs-cmdb Self-Signed",
    );

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let cert_path = default_cert_path();
    let key_path = default_key_path();

    fs::write(&cert_path, &cert_pem)?;
    fs::write(&key_path, &key_pem)?;

    info!("Generated self-signed certificate: {:?}", cert_path);
    info!("Generated private key: {:?}", key_path);

    Ok((cert_pem, key_pem))
}

pub fn load_or_generate_tls_config(
    tls_cert: &Option<String>,
    tls_key: &Option<String>,
    enable_tls: bool,
) -> Result<Option<Arc<ServerConfig>>> {
    if !enable_tls {
        return Ok(None);
    }

    let (cert_pem, key_pem) = match (tls_cert.as_ref(), tls_key.as_ref()) {
        (Some(cert_path), Some(key_path)) => {
            let cert = fs::read_to_string(cert_path)?;
            let key = fs::read_to_string(key_path)?;
            (cert, key)
        }
        _ => {
            warn!("No TLS certificate configured. Generating self-signed certificate.");
            warn!("Self-signed certificates are not trusted by browsers. Configure proper TLS for production.");
            let cert_path = default_cert_path();
            let key_path = default_key_path();
            if cert_path.exists() && key_path.exists() {
                info!("Using existing self-signed certificate from {:?}", cert_path);
                (fs::read_to_string(&cert_path)?, fs::read_to_string(&key_path)?)
            } else {
                generate_self_signed_cert()?
            }
        }
    };

    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(CertificateDer::from)
        .collect();

    let key_der = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())
        .next()
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("No private key found in TLS key file"))?;
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    info!("TLS configured successfully");
    Ok(Some(Arc::new(config)))
}

/// A TLS-aware listener that wraps TCP connections with TLS.
pub struct TlsListener {
    tcp: tokio::net::TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsListener {
    pub async fn bind(addr: &str, tls_config: Arc<ServerConfig>) -> io::Result<Self> {
        let tcp = tokio::net::TcpListener::bind(addr).await?;
        let acceptor = TlsAcceptor::from(tls_config);
        Ok(Self { tcp, acceptor })
    }
}

impl Listener for TlsListener {
    type Io = tokio_rustls::server::TlsStream<TcpStream>;
    type Addr = SocketAddr;

    fn accept(&mut self) -> impl std::future::Future<Output = (Self::Io, SocketAddr)> + Send {
        let tcp = &self.tcp;
        let acceptor = &self.acceptor;
        async move {
            loop {
                match tcp.accept().await {
                    Ok((stream, addr)) => {
                        match acceptor.accept(stream).await {
                            Ok(tls_stream) => return (tls_stream, addr),
                            Err(e) => {
                                warn!("TLS handshake error: {}", e);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("TCP accept error: {}", e);
                        continue;
                    }
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.tcp.local_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_self_signed_cert() {
        let (cert, key) = generate_self_signed_cert().unwrap();
        assert!(cert.contains("BEGIN CERTIFICATE"));
        assert!(key.contains("BEGIN PRIVATE KEY"));
        let cert_path = default_cert_path();
        let key_path = default_key_path();
        let _ = fs::remove_file(&cert_path);
        let _ = fs::remove_file(&key_path);
    }

    #[test]
    fn test_load_without_tls() {
        let config = load_or_generate_tls_config(&None, &None, false).unwrap();
        assert!(config.is_none());
    }
}
