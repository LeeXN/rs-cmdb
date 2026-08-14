use anyhow::Result;
use common::models::{
    AgentTerminalOutputRequest, AgentTerminalPollResponse, AgentTerminalStateRequest,
    AgentTerminalStreamClientMessage, AgentTerminalStreamServerMessage, TerminalOutputChunk,
    TerminalSessionState,
};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use reqwest::Client as HttpClient;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, protocol::Message},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::ClientConfig;
use crate::service::load_agent_token_for;

const POLL_RETRY_DELAY: Duration = Duration::from_secs(5);
const TOKEN_WAIT_DELAY: Duration = Duration::from_secs(30);
const ACTIVE_POLL_INTERVAL: Duration = Duration::from_millis(150);

type TerminalWs =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type TerminalUpdateTx = mpsc::UnboundedSender<AgentTerminalPollResponse>;
type TerminalUpdateRx = mpsc::UnboundedReceiver<AgentTerminalPollResponse>;
type OutboundTx = mpsc::UnboundedSender<Message>;
type OutboundRx = mpsc::UnboundedReceiver<Message>;

#[derive(Clone)]
pub struct TerminalSessionManager {
    config: Arc<ClientConfig>,
    client_id: String,
    http: Arc<HttpClient>,
    claim_id: String,
}

impl TerminalSessionManager {
    pub fn new(config: Arc<ClientConfig>, client_id: String) -> Self {
        let http = HttpClient::builder()
            .danger_accept_invalid_certs(!config.server.verify_tls)
            .timeout(Duration::from_secs(35))
            .build()
            .unwrap_or_default();
        Self {
            config,
            client_id,
            http: Arc::new(http),
            claim_id: Uuid::new_v4().to_string(),
        }
    }

    fn auth_header(&self) -> Option<String> {
        load_agent_token_for(&self.client_id)
            .map(|token| format!("Bearer {}:{}", self.client_id, token))
    }

    pub async fn run_poll_loop(&self) {
        info!(
            "TerminalSessionManager: starting terminal manager for client {}",
            self.client_id
        );
        loop {
            if self.auth_header().is_none() {
                warn!(
                    "TerminalSessionManager: agent token missing, retrying in {:?}",
                    TOKEN_WAIT_DELAY
                );
                tokio::time::sleep(TOKEN_WAIT_DELAY).await;
                continue;
            }
            match self.run_stream_loop().await {
                Ok(()) => {
                    warn!(
                        "TerminalSessionManager: terminal stream disconnected, retrying in {:?}",
                        POLL_RETRY_DELAY
                    );
                    tokio::time::sleep(POLL_RETRY_DELAY).await;
                    continue;
                }
                Err(err) => {
                    warn!(
                        "TerminalSessionManager stream error: {}, falling back to polling",
                        err
                    );
                }
            }
            match self.poll_once().await {
                Ok(Some(work)) => {
                    if let Err(e) = self.run_session(work).await {
                        error!("TerminalSessionManager session error: {}", e);
                    }
                }
                Ok(None) => debug!("TerminalSessionManager: no pending terminal session"),
                Err(e) => {
                    warn!("TerminalSessionManager poll error: {}", e);
                    tokio::time::sleep(POLL_RETRY_DELAY).await;
                }
            }
        }
    }

    async fn run_stream_loop(&self) -> Result<()> {
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let url = self.stream_url()?;
        let mut request = url.into_client_request()?;
        request
            .headers_mut()
            .insert("Authorization", HeaderValue::from_str(&auth)?);
        let (stream, _) = connect_async(request).await?;
        let (mut writer, mut reader) = stream.split();
        let (outbound_tx, mut outbound_rx): (OutboundTx, OutboundRx) = mpsc::unbounded_channel();
        let writer_task = tokio::spawn(async move {
            while let Some(message) = outbound_rx.recv().await {
                if writer.send(message).await.is_err() {
                    break;
                }
            }
        });
        let mut session_updates: HashMap<String, TerminalUpdateTx> = HashMap::new();

        while let Some(message) = reader.next().await {
            match message? {
                Message::Text(text) => {
                    let message: AgentTerminalStreamServerMessage = serde_json::from_str(&text)?;
                    match message {
                        AgentTerminalStreamServerMessage::Sync { work } => {
                            self.dispatch_stream_work(work, &mut session_updates, &outbound_tx);
                        }
                        AgentTerminalStreamServerMessage::Heartbeat => {
                            let message = AgentTerminalStreamClientMessage::Heartbeat {
                                session_ids: session_updates.keys().cloned().collect(),
                                claim_id: self.claim_id.clone(),
                            };
                            outbound_tx
                                .send(Message::Text(serde_json::to_string(&message)?.into()))
                                .map_err(|_| {
                                    anyhow::anyhow!("terminal heartbeat channel closed")
                                })?;
                        }
                    }
                }
                Message::Ping(payload) => {
                    if outbound_tx.send(Message::Pong(payload)).is_err() {
                        break;
                    }
                }
                Message::Pong(_) => {}
                Message::Close(_) => break,
                Message::Binary(_) => {}
                Message::Frame(_) => {}
            }
        }

        drop(outbound_tx);
        let _ = writer_task.await;
        Ok(())
    }

    async fn poll_once(&self) -> Result<Option<AgentTerminalPollResponse>> {
        let url = format!(
            "{}/agent/terminal-sessions/pending?client_id={}&claim_id={}",
            self.config.server.url, self.client_id, self.claim_id
        );
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        if !resp.status().is_success() {
            anyhow::bail!("terminal poll returned {}", resp.status());
        }
        let body = resp
            .json::<common::models::ApiResponse<AgentTerminalPollResponse>>()
            .await?;
        Ok(body.data)
    }

    async fn run_session(&self, work: AgentTerminalPollResponse) -> Result<()> {
        self.run_session_internal(work, None, None).await
    }

    fn dispatch_stream_work(
        &self,
        work: AgentTerminalPollResponse,
        session_updates: &mut HashMap<String, TerminalUpdateTx>,
        outbound_tx: &OutboundTx,
    ) {
        let session_id = work.session.session_id.clone();
        if let Some(sender) = session_updates.get(&session_id) {
            if sender.send(work.clone()).is_ok() {
                return;
            }
            session_updates.remove(&session_id);
        }

        let (update_tx, update_rx) = mpsc::unbounded_channel();
        let manager = self.clone();
        let outbound = outbound_tx.clone();
        let session_key = session_id.clone();
        tokio::spawn(async move {
            if let Err(err) = manager
                .run_session_internal(work, Some(update_rx), Some(outbound))
                .await
            {
                warn!(session_id = %session_key, error = %err, "terminal stream session task failed");
            }
        });
        session_updates.insert(session_id, update_tx);
    }

    async fn run_session_internal(
        &self,
        work: AgentTerminalPollResponse,
        mut update_rx: Option<TerminalUpdateRx>,
        outbound_tx: Option<OutboundTx>,
    ) -> Result<()> {
        let session = work.session.clone();
        self.send_state(
            &session.session_id,
            TerminalSessionState::Active,
            None,
            outbound_tx.as_ref(),
        )
        .await?;

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: session.rows,
            cols: session.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let mut cmd = CommandBuilder::new(session.shell.clone());
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        let mut child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let mut writer = pair.master.take_writer()?;
        for input in work.pending_input {
            writer.write_all(input.data.as_bytes())?;
        }
        writer.flush()?;

        if let Some(resize) = work.resize.as_ref() {
            let _ = pair.master.resize(PtySize {
                rows: resize.rows,
                cols: resize.cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }

        let mut reader = pair.master.try_clone_reader()?;
        let session_id = session.session_id.clone();
        let (output_tx, mut output_rx) = mpsc::unbounded_channel::<TerminalOutputChunk>();
        let output_task = task::spawn_blocking(move || {
            let mut seq = 1u64;
            let mut buf = [0u8; 2048];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = TerminalOutputChunk {
                            seq,
                            data: String::from_utf8_lossy(&buf[..n]).to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        seq += 1;
                        if output_tx.send(chunk).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut close_requested = work.close_requested;
        loop {
            if close_requested {
                let _ = child.kill();
                break;
            }

            tokio::select! {
                Some(chunk) = output_rx.recv() => {
                    self.send_output(&session_id, vec![chunk], outbound_tx.as_ref()).await?;
                }
                maybe_update = recv_update(&mut update_rx), if update_rx.is_some() => {
                    if let Some(update) = maybe_update {
                        Self::apply_update(&mut writer, pair.master.as_ref(), update, &mut close_requested)?;
                    }
                }
                _ = tokio::time::sleep(ACTIVE_POLL_INTERVAL) => {
                    if let Some(status) = child.try_wait()? {
                        while let Ok(chunk) = output_rx.try_recv() {
                            self.send_output(&session_id, vec![chunk], outbound_tx.as_ref()).await?;
                        }
                        let _ = output_task.await;
                        let close_state = if status.success() {
                            TerminalSessionState::Closed
                        } else {
                            TerminalSessionState::Failed
                        };
                        return self
                            .send_state(
                                &session.session_id,
                                close_state,
                                Some(format!("exit status: {:?}", status)),
                                outbound_tx.as_ref(),
                            )
                            .await;
                    }

                    if update_rx.is_none() {
                        if let Some(update) = self.poll_session_update(&session.session_id).await? {
                            Self::apply_update(&mut writer, pair.master.as_ref(), update, &mut close_requested)?;
                        }
                    }
                }
            }
        }

        let status = child.wait()?;
        while let Ok(chunk) = output_rx.try_recv() {
            self.send_output(&session_id, vec![chunk], outbound_tx.as_ref())
                .await?;
        }
        let _ = output_task.await;
        let close_state = if status.success() {
            TerminalSessionState::Closed
        } else {
            TerminalSessionState::Failed
        };
        self.send_state(
            &session.session_id,
            close_state,
            Some(format!("exit status: {:?}", status)),
            outbound_tx.as_ref(),
        )
        .await
    }

    async fn poll_session_update(
        &self,
        session_id: &str,
    ) -> Result<Option<AgentTerminalPollResponse>> {
        let url = format!(
            "{}/agent/terminal-sessions/pending?client_id={}&session_id={}&claim_id={}",
            self.config.server.url, self.client_id, session_id, self.claim_id
        );
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        if !resp.status().is_success() {
            anyhow::bail!("terminal session poll returned {}", resp.status());
        }
        let body = resp
            .json::<common::models::ApiResponse<AgentTerminalPollResponse>>()
            .await?;
        Ok(body.data)
    }

    async fn push_output(&self, session_id: &str, chunks: Vec<TerminalOutputChunk>) -> Result<()> {
        let url = format!(
            "{}/agent/terminal-sessions/{}/output",
            self.config.server.url, session_id
        );
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(&AgentTerminalOutputRequest {
                claim_id: Some(self.claim_id.clone()),
                chunks,
            })
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!("push terminal output returned {}", resp.status());
        }
        Ok(())
    }

    async fn send_output(
        &self,
        session_id: &str,
        chunks: Vec<TerminalOutputChunk>,
        outbound_tx: Option<&OutboundTx>,
    ) -> Result<()> {
        if let Some(tx) = outbound_tx {
            tx.send(Message::Text(
                serde_json::to_string(&AgentTerminalStreamClientMessage::Output {
                    session_id: session_id.to_string(),
                    payload: AgentTerminalOutputRequest {
                        claim_id: Some(self.claim_id.clone()),
                        chunks,
                    },
                })?
                .into(),
            ))
            .map_err(|_| anyhow::anyhow!("terminal stream outbound channel closed"))?;
            Ok(())
        } else {
            self.push_output(session_id, chunks).await
        }
    }

    async fn report_state(
        &self,
        session_id: &str,
        state: TerminalSessionState,
        message: Option<String>,
    ) -> Result<()> {
        let url = format!(
            "{}/agent/terminal-sessions/{}/state",
            self.config.server.url, session_id
        );
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let resp = self
            .http
            .post(&url)
            .header("Authorization", auth)
            .json(&AgentTerminalStateRequest {
                claim_id: Some(self.claim_id.clone()),
                state,
                message,
            })
            .send()
            .await?;
        if !resp.status().is_success() {
            anyhow::bail!("report terminal state returned {}", resp.status());
        }
        Ok(())
    }

    async fn send_state(
        &self,
        session_id: &str,
        state: TerminalSessionState,
        message: Option<String>,
        outbound_tx: Option<&OutboundTx>,
    ) -> Result<()> {
        if let Some(tx) = outbound_tx {
            tx.send(Message::Text(
                serde_json::to_string(&AgentTerminalStreamClientMessage::State {
                    session_id: session_id.to_string(),
                    payload: AgentTerminalStateRequest {
                        claim_id: Some(self.claim_id.clone()),
                        state,
                        message,
                    },
                })?
                .into(),
            ))
            .map_err(|_| anyhow::anyhow!("terminal stream outbound channel closed"))?;
            Ok(())
        } else {
            self.report_state(session_id, state, message).await
        }
    }

    fn stream_url(&self) -> Result<String> {
        let base = self.config.server.url.trim_end_matches('/');
        let without_api = base.strip_suffix("/api/v1").unwrap_or(base);
        let scheme = if without_api.starts_with("https://") {
            "wss://"
        } else if without_api.starts_with("http://") {
            "ws://"
        } else {
            anyhow::bail!(
                "unsupported server url for terminal stream: {}",
                self.config.server.url
            );
        };
        let host = without_api
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        Ok(format!(
            "{}{}/api/v1/agent/terminal-sessions/stream?claim_id={}",
            scheme, host, self.claim_id
        ))
    }

    fn apply_update(
        writer: &mut dyn Write,
        master: &dyn portable_pty::MasterPty,
        update: AgentTerminalPollResponse,
        close_requested: &mut bool,
    ) -> Result<()> {
        for input in update.pending_input {
            writer.write_all(input.data.as_bytes())?;
        }
        writer.flush()?;

        if let Some(resize) = update.resize {
            let _ = master.resize(PtySize {
                rows: resize.rows,
                cols: resize.cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }

        *close_requested = update.close_requested;
        Ok(())
    }
}

async fn recv_update(
    update_rx: &mut Option<TerminalUpdateRx>,
) -> Option<AgentTerminalPollResponse> {
    match update_rx {
        Some(rx) => rx.recv().await,
        None => None,
    }
}
