use axum::{
    extract::{Extension, Path, Query},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE, HOST},
    },
    response::{Json, Response},
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, instrument};

use crate::config::ServerConfig;
use crate::service::client_service::ClientService;

#[derive(Deserialize, Debug)]
pub struct DownloadQuery {
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    arch: Option<String>,
}

#[derive(Serialize)]
pub struct ClientInfo {
    pub server_url: String,
    pub download_url: String,
    pub install_script: String,
    pub systemd_service: String,
    pub config_template: String,
}

/// Get client download information
#[debug_handler]
#[instrument(skip(config, _client_service))]
pub async fn get_client_info(
    headers: HeaderMap,
    Query(params): Query<DownloadQuery>,
    Extension(config): Extension<Arc<ServerConfig>>,
    Extension(_client_service): Extension<Arc<ClientService>>,
) -> Result<Json<ClientInfo>, StatusCode> {
    let platform = params.platform.unwrap_or_else(|| "linux".to_string());
    let arch = params.arch.unwrap_or_else(|| "x86_64".to_string());
    let binary_name = match platform.as_str() {
        "linux" => "rs-cmdb-client",
        "windows" => "rs-cmdb-client.exe",
        "darwin" => "rs-cmdb-client",
        _ => "rs-cmdb-client",
    };

    // Determine server URL from headers or config
    let host = headers
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| format!("{}:{}", config.host, config.port));

    // Check for X-Forwarded-Proto to determine scheme
    let scheme = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let server_url = format!("{}://{}", scheme, host);
    let download_url = format!(
        "{}/api/v1/download/client/{}/{}/{}",
        server_url, platform, arch, binary_name
    );

    let install_script = generate_install_script(&server_url, &platform, &arch);
    let systemd_service = generate_systemd_service(&server_url);
    let config_template = generate_config_template(&server_url);

    let response = ClientInfo {
        server_url: server_url.clone(),
        download_url,
        install_script,
        systemd_service,
        config_template,
    };

    Ok(Json(response))
}

/// 下载客户端二进制文件
#[debug_handler]
#[instrument]
pub async fn download_client(
    Path((platform, arch, binary_name)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, &'static str)> {
    let file_path = format!("binaries/{}/{}/{}", platform, arch, binary_name);
    println!("Downloading client binary from path: {}", file_path);

    match fs::read(&file_path).await {
        Ok(content) => {
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            headers.insert(
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", binary_name)
                    .parse()
                    .unwrap(),
            );

            info!("Client binary downloaded: {}", file_path);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(content.into())
                .unwrap())
        }
        Err(e) => {
            error!("Failed to read client binary: {}", e);
            Err((StatusCode::NOT_FOUND, "Client binary not found"))
        }
    }
}

/// Get install script directly
#[debug_handler]
#[instrument(skip(config))]
pub async fn get_install_script(
    headers: HeaderMap,
    Extension(config): Extension<Arc<ServerConfig>>,
) -> Result<Response, StatusCode> {
    // Determine server URL from headers or config
    let host = headers
        .get(HOST)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string())
        .unwrap_or_else(|| format!("{}:{}", config.host, config.port));

    // Check for X-Forwarded-Proto to determine scheme
    let scheme = headers
        .get("X-Forwarded-Proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");

    let server_url = format!("{}://{}", scheme, host);
    let script = generate_install_script(&server_url, "linux", "x86_64");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/x-shellscript")
        .body(script.into())
        .unwrap())
}

fn generate_install_script(server_url: &str, platform: &str, arch: &str) -> String {
    match platform {
        "linux" => format!(
            r#"#!/bin/bash
# RS-CMDB Client Install Script

set -e

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

CLIENT_URL="{}/api/v1/download/client/linux/$ARCH/rs-cmdb-client"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/rs-cmdb"
SERVICE_FILE="/etc/systemd/system/rs-cmdb-client.service"

echo "Installing RS-CMDB Client..."
echo "Detected architecture: $ARCH"

# Create directories
sudo mkdir -p $INSTALL_DIR
sudo mkdir -p $CONFIG_DIR

# Download client binary
echo "Downloading client binary..."
sudo wget -O $INSTALL_DIR/rs-cmdb-client "$CLIENT_URL"
sudo chmod +x $INSTALL_DIR/rs-cmdb-client

# Generate config
echo "Generating configuration..."
sudo $INSTALL_DIR/rs-cmdb-client generate-config --server {}/api/v1 --output $CONFIG_DIR/client.toml

# Create systemd service
echo "Creating systemd service..."
sudo tee $SERVICE_FILE > /dev/null << 'EOF'
[Unit]
Description=RS-CMDB Client
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/rs-cmdb-client --config /etc/rs-cmdb/client.toml service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable rs-cmdb-client
sudo systemctl start rs-cmdb-client

echo "RS-CMDB Client installed and started successfully!"
echo "Check status with: sudo systemctl status rs-cmdb-client"
"#,
            server_url, server_url
        ),
        "windows" => format!(
            r#"@echo off
REM RS-CMDB Client Install Script for Windows

echo Installing RS-CMDB Client...

REM Create directories
if not exist "C:\Program Files\rs-cmdb" mkdir "C:\Program Files\rs-cmdb"
if not exist "C:\ProgramData\rs-cmdb" mkdir "C:\ProgramData\rs-cmdb"

REM Download client binary (requires PowerShell)
echo Downloading client binary...
powershell -Command "Invoke-WebRequest -Uri '{}/api/v1/download/client/{}/{}/rs-cmdb-client.exe' -OutFile 'C:\Program Files\rs-cmdb\rs-cmdb-client.exe'"

REM Generate config
echo Generating configuration...
"C:\Program Files\rs-cmdb\rs-cmdb-client.exe" generate-config --server {} --output "C:\ProgramData\rs-cmdb\client.toml"

echo RS-CMDB Client installed successfully!
echo Run manually with: "C:\Program Files\rs-cmdb\rs-cmdb-client.exe" --config "C:\ProgramData\rs-cmdb\client.toml" service
"#,
            server_url, platform, arch, server_url
        ),
        _ => "# Unsupported platform".to_string(),
    }
}

fn generate_systemd_service(_server_url: &str) -> String {
    r#"[Unit]
Description=RS-CMDB Client
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/rs-cmdb-client --config /etc/rs-cmdb/client.toml service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target"#
        .to_string()
}

fn generate_config_template(server_url: &str) -> String {
    format!(
        r#"# RS-CMDB Client Configuration

# 客户端唯一标识符（留空将自动生成）
client_id = ""

# 客户端主机名（留空将自动检测）
hostname = ""

[server]
# 服务器地址
url = "{}/api/v1"
# 是否验证TLS证书
verify_tls = true

[report]
# 是否启用服务模式
service_mode = true
# 是否启用推送模式
push_enabled = true
# 推送间隔（秒）
push_interval = 300
# 是否启用拉取模式
pull_enabled = true
# 收集的硬件组件
components = ["sys", "os", "cpu", "ram", "disk", "nic", "gpu", "ipmi"]

[logging]
# 日志级别
level = "info"
# 日志文件路径（可选）
file = "/var/log/rs-cmdb-client.log"
"#,
        server_url
    )
}
