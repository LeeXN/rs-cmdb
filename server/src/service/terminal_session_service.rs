use crate::repository::terminal_session_repository::TerminalSessionRepository;
use crate::service::cast_recorder::CastRecorderInner;
use crate::service::execution_session_service::ExecutionSessionService;
use crate::service::web_terminal_service::WebTerminalService;
use chrono::Utc;
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};
use common::entity::permission::PermissionAction;
use common::entity::permission::TerminalMode;
use common::entity::user::Role;
use common::error::{CmdbError, CmdbResult};
use common::models::{
    AgentTerminalOutputRequest, AgentTerminalPollResponse, AgentTerminalStateRequest,
    CreateTerminalSessionRequest, TerminalInputChunk, TerminalOpsSummaryResponse,
    TerminalOutputChunk, TerminalResizeInstruction, TerminalSessionState, TerminalSessionSummary,
};
use std::collections::{HashMap, HashSet};
use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Mutex, broadcast};
use tracing::warn;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 512;
const ACTIVE_SESSION_STALE_SECS: i64 = 90;
const LEASE_SECS: i64 = 15;

type OutputSender = broadcast::Sender<Arc<TerminalOutputChunk>>;

pub struct TerminalSessionService {
    repo: Arc<TerminalSessionRepository>,
    policy: Arc<WebTerminalService>,
    output_channels: Mutex<HashMap<String, OutputSender>>,
    client_channels: Mutex<HashMap<String, broadcast::Sender<()>>>,
    session_svc: StdMutex<Option<Arc<ExecutionSessionService>>>,
    input_sequence: AtomicU64,
}

impl TerminalSessionService {
    pub fn new(repo: Arc<TerminalSessionRepository>, policy: Arc<WebTerminalService>) -> Arc<Self> {
        Arc::new(Self {
            repo,
            policy,
            output_channels: Mutex::new(HashMap::new()),
            client_channels: Mutex::new(HashMap::new()),
            session_svc: StdMutex::new(None),
            input_sequence: AtomicU64::new(Utc::now().timestamp_micros().max(0) as u64),
        })
    }

    pub fn configure_history(&self, session_svc: Arc<ExecutionSessionService>) {
        *self
            .session_svc
            .lock()
            .expect("terminal session history lock") = Some(session_svc);
    }

    pub async fn allows_client_permission_scope(
        &self,
        perm_ctx: &crate::middleware::permission::PermissionContext,
        client_id: &str,
        action: &PermissionAction,
    ) -> bool {
        self.policy
            .allows_client_permission_scope(perm_ctx, client_id, action)
            .await
    }

    #[allow(dead_code)]
    pub async fn create_session(
        &self,
        req: &CreateTerminalSessionRequest,
        user_id: &str,
        username: &str,
        role: &Role,
    ) -> CmdbResult<TerminalSessionSummary> {
        self.create_session_with_groups(req, user_id, username, role, &[])
            .await
    }

    pub async fn create_session_with_groups(
        &self,
        req: &CreateTerminalSessionRequest,
        user_id: &str,
        username: &str,
        role: &Role,
        group_ids: &[String],
    ) -> CmdbResult<TerminalSessionSummary> {
        let mode = self
            .policy
            .reserve_session_slot_for_groups(user_id, role, group_ids, &req.client_id)
            .await?;
        let now = Utc::now().to_rfc3339();
        let session = TerminalSessionSummary {
            session_id: Uuid::new_v4().to_string(),
            client_id: req.client_id.clone(),
            user_id: user_id.to_string(),
            username: username.to_string(),
            role: role.clone(),
            group_ids: group_ids.to_vec(),
            mode,
            state: TerminalSessionState::Pending,
            shell: req.shell.clone().unwrap_or_else(|| "/bin/bash".to_string()),
            cols: req.cols.unwrap_or(120),
            rows: req.rows.unwrap_or(32),
            created_at: now,
            activated_at: None,
            closed_at: None,
            last_activity_at: None,
            last_heartbeat_at: None,
            lease_id: None,
            lease_expires_at: None,
            close_reason: None,
        };
        if let Err(err) = self.repo.save_session(&session).await {
            // Roll back the in-memory reservation if persistence fails.
            self.policy.on_session_close(user_id);
            return Err(err);
        }
        self.record_history_created(&session).await;
        self.notify_client(&session.client_id).await;
        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> CmdbResult<TerminalSessionSummary> {
        self.repo.get_session(session_id).await?.ok_or_else(|| {
            CmdbError::NotFound(format!("terminal session {} not found", session_id))
        })
    }

    pub async fn list_client_sessions(
        &self,
        client_id: &str,
        user_id: &str,
        is_admin: bool,
    ) -> CmdbResult<Vec<TerminalSessionSummary>> {
        let mut sessions = self.normalize_stale_sessions(client_id).await?;
        if !is_admin {
            sessions.retain(|session| session.user_id == user_id);
        }
        sessions.sort_by(|left, right| {
            session_rank(right)
                .cmp(&session_rank(left))
                .then_with(|| session_sort_ts(right).cmp(session_sort_ts(left)))
        });
        Ok(sessions)
    }

    pub async fn list_all_sessions(&self) -> CmdbResult<Vec<TerminalSessionSummary>> {
        let sessions = self.repo.list_all_sessions().await?;
        let client_ids: HashSet<String> = sessions
            .iter()
            .map(|session| session.client_id.clone())
            .collect();
        for client_id in client_ids {
            let _ = self.normalize_stale_sessions(&client_id).await?;
        }

        let mut sessions = self.repo.list_all_sessions().await?;
        sessions.sort_by(|left, right| {
            session_rank(right)
                .cmp(&session_rank(left))
                .then_with(|| session_sort_ts(right).cmp(session_sort_ts(left)))
        });
        Ok(sessions)
    }

    pub async fn terminal_ops_summary(&self) -> CmdbResult<TerminalOpsSummaryResponse> {
        let sessions = self.list_all_sessions().await?;
        let mut summary = TerminalOpsSummaryResponse {
            stale_session_threshold_secs: ACTIVE_SESSION_STALE_SECS,
            ..Default::default()
        };
        let mut active_clients = HashSet::new();

        for session in sessions {
            summary.total_sessions += 1;
            match session.state {
                TerminalSessionState::Pending => summary.pending_sessions += 1,
                TerminalSessionState::Active => summary.active_sessions += 1,
                TerminalSessionState::Closed => summary.closed_sessions += 1,
                TerminalSessionState::Failed => summary.failed_sessions += 1,
            }
            if matches!(
                session.state,
                TerminalSessionState::Pending | TerminalSessionState::Active
            ) {
                active_clients.insert(session.client_id);
            }
        }

        summary.active_clients = active_clients.len();
        Ok(summary)
    }

    pub async fn queue_input(&self, session_id: &str, data: String) -> CmdbResult<()> {
        let mut session = self.get_session(session_id).await?;
        self.validate_session_input(&session, &data).await?;
        self.enqueue_input(session_id, &mut session, data).await
    }

    pub async fn queue_browser_input(
        &self,
        session_id: &str,
        data: String,
        pending_line: &mut String,
    ) -> CmdbResult<()> {
        let mut session = self.get_session(session_id).await?;
        if session.mode == TerminalMode::ReadWrite {
            return self.enqueue_input(session_id, &mut session, data).await;
        }

        pending_line.push_str(&data);
        let Some(split_at) = pending_line.rfind(['\n', '\r']) else {
            return Ok(());
        };

        let raw_ready = pending_line[..=split_at].to_string();
        let remainder = pending_line[split_at + 1..].to_string();
        for command in raw_ready
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            self.policy
                .check_command_allowed_for_groups(
                    &session.user_id,
                    &session.role,
                    &session.group_ids,
                    &session.client_id,
                    command,
                )
                .await?;
        }
        *pending_line = remainder;
        self.enqueue_input(session_id, &mut session, raw_ready)
            .await
    }

    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) -> CmdbResult<()> {
        let mut session = self.get_session(session_id).await?;
        session.cols = cols;
        session.rows = rows;
        session.last_activity_at = Some(Utc::now().to_rfc3339());
        self.repo.save_session(&session).await?;
        self.repo
            .set_resize(
                session_id,
                &TerminalResizeInstruction {
                    cols,
                    rows,
                    timestamp: Utc::now().to_rfc3339(),
                },
            )
            .await?;
        self.notify_client(&session.client_id).await;
        Ok(())
    }

    pub async fn close_session(&self, session_id: &str, reason: Option<String>) -> CmdbResult<()> {
        let mut session = self.get_session(session_id).await?;
        if session.state != TerminalSessionState::Closed {
            if matches!(session.state, TerminalSessionState::Pending) {
                session.state = TerminalSessionState::Closed;
                session.closed_at = Some(Utc::now().to_rfc3339());
                session.last_activity_at = session.closed_at.clone();
                session.close_reason = reason;
                self.repo.save_session(&session).await?;
                self.policy.on_session_close(&session.user_id);
                self.close_channel(session_id).await;
                self.notify_client(&session.client_id).await;
                self.sync_history_status(&session, SessionStatus::Success)
                    .await;
            } else {
                self.repo.request_close(session_id).await?;
                self.notify_client(&session.client_id).await;
                self.sync_history_status(&session, SessionStatus::Success)
                    .await;
            }
        }
        Ok(())
    }

    pub async fn collect_agent_work(
        &self,
        client_id: &str,
        claim_id: Option<&str>,
    ) -> CmdbResult<Vec<AgentTerminalPollResponse>> {
        let sessions = self.normalize_stale_sessions(client_id).await?;
        let Some(claim_id) = claim_id.filter(|value| !value.trim().is_empty()) else {
            return Ok(Vec::new());
        };

        let mut work_items = Vec::new();
        for session in sessions.into_iter().filter(|session| {
            matches!(
                session.state,
                TerminalSessionState::Pending | TerminalSessionState::Active
            ) && self.can_claim_session(session, claim_id)
        }) {
            let Some(session) = self.claim_session(&session, client_id, claim_id).await? else {
                continue;
            };
            let pending_input = self.repo.drain_input(&session.session_id).await?;
            let resize = self.repo.take_resize(&session.session_id).await?;
            let close_requested = self.repo.take_close_request(&session.session_id).await?;
            let attach_requested = self.repo.take_attach_request(&session.session_id).await?;

            let should_sync = session.state == TerminalSessionState::Pending
                || !pending_input.is_empty()
                || resize.is_some()
                || close_requested
                || attach_requested;
            if !should_sync {
                continue;
            }

            work_items.push(AgentTerminalPollResponse {
                session,
                pending_input,
                resize,
                close_requested: close_requested || attach_requested,
            });
        }

        Ok(work_items)
    }

    pub async fn poll_for_agent(
        &self,
        client_id: &str,
        session_id: Option<&str>,
        claim_id: Option<&str>,
    ) -> CmdbResult<Option<AgentTerminalPollResponse>> {
        let sessions = self.normalize_stale_sessions(client_id).await?;
        let Some(claim_id) = claim_id.filter(|value| !value.trim().is_empty()) else {
            return Ok(None);
        };
        let session = sessions.into_iter().find(|s| {
            matches!(
                s.state,
                TerminalSessionState::Pending | TerminalSessionState::Active
            ) && session_id.map(|id| id == s.session_id).unwrap_or(true)
                && self.can_claim_session(s, claim_id)
        });
        let Some(session) = session else {
            return Ok(None);
        };
        let Some(session) = self.claim_session(&session, client_id, claim_id).await? else {
            return Ok(None);
        };
        let pending_input = self.repo.drain_input(&session.session_id).await?;
        let resize = self.repo.take_resize(&session.session_id).await?;
        let close_requested = self.repo.take_close_request(&session.session_id).await?;
        let attach_requested = self.repo.take_attach_request(&session.session_id).await?;
        Ok(Some(AgentTerminalPollResponse {
            session,
            pending_input,
            resize,
            close_requested: close_requested || attach_requested,
        }))
    }

    #[allow(dead_code)]
    pub async fn report_output(
        &self,
        session_id: &str,
        req: AgentTerminalOutputRequest,
    ) -> CmdbResult<()> {
        self.report_output_inner(session_id, req, None).await
    }

    pub async fn report_output_for_client(
        &self,
        session_id: &str,
        client_id: &str,
        req: AgentTerminalOutputRequest,
    ) -> CmdbResult<()> {
        self.report_output_inner(session_id, req, Some(client_id))
            .await
    }

    async fn report_output_inner(
        &self,
        session_id: &str,
        req: AgentTerminalOutputRequest,
        expected_client_id: Option<&str>,
    ) -> CmdbResult<()> {
        if req.chunks.is_empty() {
            return Ok(());
        }

        if let Some(client_id) = expected_client_id {
            let session = self.get_session(session_id).await?;
            ensure_agent_session_client(&session, client_id)?;
            let Some(claim_id) = req
                .claim_id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty())
            else {
                return Err(CmdbError::Forbidden(
                    "terminal session claim is required".into(),
                ));
            };
            if matches!(
                session.state,
                TerminalSessionState::Closed | TerminalSessionState::Failed
            ) {
                return Err(CmdbError::Forbidden(
                    "terminal session is no longer open".into(),
                ));
            }

            let now = Utc::now();
            let updated = self
                .repo
                .append_output_if_claimed(
                    session_id,
                    client_id,
                    claim_id,
                    &req.chunks,
                    &now.to_rfc3339(),
                    &(now + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339(),
                )
                .await?;
            let Some(session) = updated else {
                return Err(CmdbError::Forbidden(
                    "terminal session lease is no longer valid".into(),
                ));
            };

            self.sync_history_status(&session, SessionStatus::Running)
                .await;
            let sender = self.get_or_create_channel(session_id).await;
            for chunk in req.chunks {
                self.append_cast_frame(&session, &chunk);
                let _ = sender.send(Arc::new(chunk));
            }
            return Ok(());
        }

        let mut session = self.get_session(session_id).await?;
        self.ensure_claim(&mut session, req.claim_id.as_deref())?;
        self.repo.append_output(session_id, &req.chunks).await?;
        let now = Utc::now().to_rfc3339();
        session.last_activity_at = Some(now.clone());
        session.last_heartbeat_at = Some(now.clone());
        session.lease_expires_at =
            Some((Utc::now() + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339());
        if session.state == TerminalSessionState::Pending {
            session.state = TerminalSessionState::Active;
            session.activated_at = Some(now);
        }
        self.repo.save_session(&session).await?;
        self.sync_history_status(&session, SessionStatus::Running)
            .await;
        let sender = self.get_or_create_channel(session_id).await;
        for chunk in req.chunks {
            self.append_cast_frame(&session, &chunk);
            let _ = sender.send(Arc::new(chunk));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn report_state(
        &self,
        session_id: &str,
        req: AgentTerminalStateRequest,
    ) -> CmdbResult<()> {
        self.report_state_inner(session_id, req, None).await
    }

    pub async fn report_state_for_client(
        &self,
        session_id: &str,
        client_id: &str,
        req: AgentTerminalStateRequest,
    ) -> CmdbResult<()> {
        self.report_state_inner(session_id, req, Some(client_id))
            .await
    }

    pub async fn heartbeat_for_client(
        &self,
        session_id: &str,
        client_id: &str,
        claim_id: &str,
    ) -> CmdbResult<()> {
        let now = Utc::now();
        let refreshed = self
            .repo
            .heartbeat_session(
                session_id,
                client_id,
                claim_id,
                &now.to_rfc3339(),
                &(now + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339(),
            )
            .await?;
        if refreshed.is_some() {
            return Ok(());
        }

        // Re-read only to select the correct response. No stale object is
        // written back after the atomic repository check above.
        let session = self.get_session(session_id).await?;
        ensure_agent_session_client(&session, client_id)?;
        if session.state != TerminalSessionState::Active {
            return Ok(());
        }
        Err(CmdbError::Forbidden(
            "terminal session lease is held by another agent".into(),
        ))
    }

    async fn report_state_inner(
        &self,
        session_id: &str,
        req: AgentTerminalStateRequest,
        expected_client_id: Option<&str>,
    ) -> CmdbResult<()> {
        if let Some(client_id) = expected_client_id {
            let session = self.get_session(session_id).await?;
            ensure_agent_session_client(&session, client_id)?;
            let Some(claim_id) = req
                .claim_id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty())
            else {
                return Err(CmdbError::Forbidden(
                    "terminal session claim is required".into(),
                ));
            };
            if matches!(
                session.state,
                TerminalSessionState::Closed | TerminalSessionState::Failed
            ) {
                // A duplicate terminal state notification is harmless, but do
                // not let an old agent attach a new lease to a closed record.
                if req.state == session.state {
                    return Ok(());
                }
                return Err(CmdbError::Forbidden(
                    "terminal session is no longer open".into(),
                ));
            }

            let now = Utc::now();
            let updated = self
                .repo
                .update_state_if_claimed(
                    session_id,
                    client_id,
                    claim_id,
                    req.state.clone(),
                    req.message.clone(),
                    &now.to_rfc3339(),
                    &(now + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339(),
                )
                .await?;
            let Some(session) = updated else {
                return Err(CmdbError::Forbidden(
                    "terminal session lease is no longer valid".into(),
                ));
            };

            if matches!(
                session.state,
                TerminalSessionState::Closed | TerminalSessionState::Failed
            ) {
                self.policy.on_session_close(&session.user_id);
                self.close_channel(session_id).await;
            }
            let status = match session.state {
                TerminalSessionState::Pending | TerminalSessionState::Active => {
                    SessionStatus::Running
                }
                TerminalSessionState::Closed => SessionStatus::Success,
                TerminalSessionState::Failed => SessionStatus::Failed,
            };
            self.sync_history_status(&session, status).await;
            return Ok(());
        }

        let mut session = self.get_session(session_id).await?;
        self.ensure_claim(&mut session, req.claim_id.as_deref())?;
        session.state = req.state.clone();
        let now = Utc::now().to_rfc3339();
        session.last_activity_at = Some(now.clone());
        session.last_heartbeat_at = Some(now.clone());
        session.lease_expires_at =
            Some((Utc::now() + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339());
        if req.state == TerminalSessionState::Active && session.activated_at.is_none() {
            session.activated_at = Some(now.clone());
        }
        if matches!(
            req.state,
            TerminalSessionState::Closed | TerminalSessionState::Failed
        ) {
            session.closed_at = Some(now);
            session.close_reason = req.message;
            session.lease_id = None;
            session.lease_expires_at = None;
            self.policy.on_session_close(&session.user_id);
            self.close_channel(session_id).await;
        }
        self.repo.save_session(&session).await?;
        let status = match req.state {
            TerminalSessionState::Pending | TerminalSessionState::Active => SessionStatus::Running,
            TerminalSessionState::Closed => SessionStatus::Success,
            TerminalSessionState::Failed => SessionStatus::Failed,
        };
        self.sync_history_status(&session, status).await;
        Ok(())
    }

    pub async fn get_output(&self, session_id: &str) -> CmdbResult<Vec<TerminalOutputChunk>> {
        self.repo.get_output(session_id).await
    }

    pub async fn subscribe_output(
        &self,
        session_id: &str,
    ) -> broadcast::Receiver<Arc<TerminalOutputChunk>> {
        self.get_or_create_channel(session_id).await.subscribe()
    }

    pub async fn subscribe_client_events(&self, client_id: &str) -> broadcast::Receiver<()> {
        self.get_or_create_client_channel(client_id)
            .await
            .subscribe()
    }

    async fn get_or_create_channel(&self, session_id: &str) -> OutputSender {
        let mut map = self.output_channels.lock().await;
        if let Some(sender) = map.get(session_id) {
            sender.clone()
        } else {
            let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
            map.insert(session_id.to_string(), sender.clone());
            sender
        }
    }

    async fn get_or_create_client_channel(&self, client_id: &str) -> broadcast::Sender<()> {
        let mut map = self.client_channels.lock().await;
        if let Some(sender) = map.get(client_id) {
            sender.clone()
        } else {
            let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
            map.insert(client_id.to_string(), sender.clone());
            sender
        }
    }

    async fn notify_client(&self, client_id: &str) {
        let sender = self.get_or_create_client_channel(client_id).await;
        let _ = sender.send(());
    }

    async fn close_channel(&self, session_id: &str) {
        let mut map = self.output_channels.lock().await;
        map.remove(session_id);
    }

    async fn normalize_stale_sessions(
        &self,
        client_id: &str,
    ) -> CmdbResult<Vec<TerminalSessionSummary>> {
        let sessions = self.repo.list_open_sessions_for_client(client_id).await?;
        for session in &sessions {
            let configured_timeout = self
                .policy
                .get_session_timeout_for_groups(
                    &session.user_id,
                    &session.role,
                    &session.group_ids,
                    client_id,
                )
                .await;
            let stale_secs = if configured_timeout == 0 {
                ACTIVE_SESSION_STALE_SECS
            } else {
                configured_timeout as i64
            };
            let last_seen_str = match session.state {
                TerminalSessionState::Active => session
                    .last_heartbeat_at
                    .as_ref()
                    .or(session.activated_at.as_ref()),
                TerminalSessionState::Pending => session
                    .last_activity_at
                    .as_ref()
                    .or(Some(&session.created_at)),
                TerminalSessionState::Closed | TerminalSessionState::Failed => None,
            };
            let Some(last_seen_str) = last_seen_str else {
                continue;
            };
            let Ok(last_seen) = chrono::DateTime::parse_from_rfc3339(last_seen_str) else {
                continue;
            };
            if Utc::now()
                .signed_duration_since(last_seen.with_timezone(&Utc))
                .num_seconds()
                <= stale_secs
            {
                continue;
            }
            let now = Utc::now();
            let cutoff = now - chrono::Duration::seconds(stale_secs);
            let Some(closed) = self
                .repo
                .close_stale_session(
                    &session.session_id,
                    client_id,
                    last_seen_str,
                    &cutoff.to_rfc3339(),
                    &now.to_rfc3339(),
                )
                .await?
            else {
                continue;
            };
            self.sync_history_status(&closed, SessionStatus::Failed)
                .await;
            self.policy.on_session_close(&closed.user_id);
            self.close_channel(&closed.session_id).await;
        }
        self.repo.list_open_sessions_for_client(client_id).await
    }

    fn can_claim_session(&self, session: &TerminalSessionSummary, claim_id: &str) -> bool {
        match session.lease_id.as_deref() {
            None => true,
            Some(existing) if existing == claim_id => true,
            Some(_) => self.lease_expired(session),
        }
    }

    async fn claim_session(
        &self,
        session: &TerminalSessionSummary,
        client_id: &str,
        claim_id: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let now = Utc::now();
        let lease_expires_at = now + chrono::Duration::seconds(LEASE_SECS);
        self.repo
            .claim_session(
                &session.session_id,
                client_id,
                claim_id,
                &now.to_rfc3339(),
                &lease_expires_at.to_rfc3339(),
            )
            .await
    }

    fn ensure_claim(
        &self,
        session: &mut TerminalSessionSummary,
        claim_id: Option<&str>,
    ) -> CmdbResult<()> {
        let Some(claim_id) = claim_id.filter(|value| !value.trim().is_empty()) else {
            return Ok(());
        };
        match session.lease_id.as_deref() {
            None => {
                self.refresh_claim(session, claim_id);
                Ok(())
            }
            Some(existing) if existing == claim_id => {
                self.refresh_claim(session, claim_id);
                Ok(())
            }
            Some(_) if self.lease_expired(session) => {
                self.refresh_claim(session, claim_id);
                Ok(())
            }
            Some(existing) => Err(CmdbError::Forbidden(format!(
                "terminal session lease is held by {}",
                existing
            ))),
        }
    }

    fn refresh_claim(&self, session: &mut TerminalSessionSummary, claim_id: &str) {
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        if session.lease_id.as_deref() != Some(claim_id) {
            session.lease_id = Some(claim_id.to_string());
            session.last_activity_at = Some(now_str.clone());
        }
        session.last_heartbeat_at = Some(now_str);
        session.lease_expires_at = Some((now + chrono::Duration::seconds(LEASE_SECS)).to_rfc3339());
    }

    fn lease_expired(&self, session: &TerminalSessionSummary) -> bool {
        let Some(deadline) = session.lease_expires_at.as_ref() else {
            return false;
        };
        chrono::DateTime::parse_from_rfc3339(deadline)
            .map(|parsed| parsed.with_timezone(&Utc) <= Utc::now())
            .unwrap_or(false)
    }

    async fn validate_session_input(
        &self,
        session: &TerminalSessionSummary,
        data: &str,
    ) -> CmdbResult<()> {
        if session.state == TerminalSessionState::Closed
            || session.state == TerminalSessionState::Failed
        {
            return Err(CmdbError::Validation(
                "terminal session already closed".to_string(),
            ));
        }
        if session.mode == TerminalMode::ReadOnly {
            for command in data
                .split(['\n', '\r'])
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                self.policy
                    .check_command_allowed_for_groups(
                        &session.user_id,
                        &session.role,
                        &session.group_ids,
                        &session.client_id,
                        command,
                    )
                    .await?;
            }
        }
        Ok(())
    }

    fn history_service(&self) -> Option<Arc<ExecutionSessionService>> {
        self.session_svc
            .lock()
            .expect("terminal session history lock")
            .clone()
    }

    async fn record_history_created(&self, session: &TerminalSessionSummary) {
        let Some(session_svc) = self.history_service() else {
            return;
        };

        let mut execution = ExecutionSession::new(
            &session.user_id,
            &session.username,
            vec![session.client_id.clone()],
            &format!("[Web Terminal] {}", session.shell),
            ExecutionType::Terminal,
        );
        execution.session_id = session.session_id.clone();
        execution.start_time = session.created_at.clone();

        match CastRecorderInner::init_cast(
            &session.session_id,
            session.cols as u64,
            session.rows as u64,
        ) {
            Ok(()) => {
                execution.cast_file_path = Some(
                    CastRecorderInner::cast_path(&session.session_id)
                        .display()
                        .to_string(),
                );
            }
            Err(err) => {
                warn!(session_id = %session.session_id, error = %err, "failed to init terminal cast")
            }
        }

        if let Err(err) = session_svc.save_session(&execution).await {
            warn!(session_id = %session.session_id, error = %err, "failed to create terminal execution session");
        }
    }

    async fn sync_history_status(&self, session: &TerminalSessionSummary, status: SessionStatus) {
        let Some(session_svc) = self.history_service() else {
            return;
        };

        match session_svc.get_session(&session.session_id).await {
            Ok(Some(mut execution)) => {
                execution.status = status.clone();
                if matches!(
                    status,
                    SessionStatus::Success | SessionStatus::Failed | SessionStatus::Partial
                ) {
                    let end_time = session
                        .closed_at
                        .clone()
                        .or_else(|| Some(Utc::now().to_rfc3339()));
                    let start = chrono::DateTime::parse_from_rfc3339(&execution.start_time).ok();
                    let end = end_time
                        .as_deref()
                        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok());
                    execution.end_time = end_time;
                    execution.duration_secs = start.zip(end).map(|(start, end)| {
                        end.signed_duration_since(start).num_seconds().max(0) as u64
                    });
                }
                if let Err(err) = session_svc.save_session(&execution).await {
                    warn!(session_id = %session.session_id, error = %err, "failed to save terminal execution session");
                }
            }
            Ok(None) => {}
            Err(err) => {
                warn!(session_id = %session.session_id, error = %err, "failed to load terminal execution session")
            }
        }
    }

    fn append_cast_frame(&self, session: &TerminalSessionSummary, chunk: &TerminalOutputChunk) {
        let elapsed = chrono::DateTime::parse_from_rfc3339(&chunk.timestamp)
            .ok()
            .zip(chrono::DateTime::parse_from_rfc3339(&session.created_at).ok())
            .map(|(chunk_ts, start_ts)| {
                chunk_ts
                    .signed_duration_since(start_ts)
                    .num_microseconds()
                    .unwrap_or(0)
                    .max(0) as f64
                    / 1_000_000.0
            })
            .unwrap_or(0.0);
        if let Err(err) = CastRecorderInner::append_frame(&session.session_id, elapsed, &chunk.data)
        {
            warn!(session_id = %session.session_id, error = %err, "failed to append terminal cast frame");
        }
    }

    async fn enqueue_input(
        &self,
        session_id: &str,
        session: &mut TerminalSessionSummary,
        data: String,
    ) -> CmdbResult<()> {
        if session.state == TerminalSessionState::Closed
            || session.state == TerminalSessionState::Failed
        {
            return Err(CmdbError::Validation(
                "terminal session already closed".to_string(),
            ));
        }
        session.last_activity_at = Some(Utc::now().to_rfc3339());
        self.repo.save_session(session).await?;
        let timestamp = Utc::now().to_rfc3339();
        // A timestamp alone can collide when concurrent browser requests
        // arrive in the same microsecond. Use a process-wide monotonic
        // sequence so queued input is never silently overwritten.
        let seq = self.input_sequence.fetch_add(1, Ordering::Relaxed);
        let chunk = TerminalInputChunk {
            seq,
            data,
            timestamp,
        };
        self.repo.queue_input(session_id, &chunk).await?;
        self.notify_client(&session.client_id).await;
        Ok(())
    }
}

fn ensure_agent_session_client(
    session: &TerminalSessionSummary,
    client_id: &str,
) -> CmdbResult<()> {
    if session.client_id == client_id {
        Ok(())
    } else {
        Err(CmdbError::Forbidden(
            "terminal session does not belong to this agent".into(),
        ))
    }
}

fn session_rank(session: &TerminalSessionSummary) -> u8 {
    match session.state {
        TerminalSessionState::Active => 2,
        TerminalSessionState::Pending => 1,
        TerminalSessionState::Closed | TerminalSessionState::Failed => 0,
    }
}

fn session_sort_ts(session: &TerminalSessionSummary) -> &str {
    session
        .last_activity_at
        .as_deref()
        .or(session.activated_at.as_deref())
        .unwrap_or(&session.created_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::execution_session_repository::ExecutionSessionRepository;
    use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
    use async_trait::async_trait;
    use common::entity::permission::{
        CommandRules, SubjectType, TargetScope, TerminalMode, WebTerminalPolicy,
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MemoryDb {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl Database for MemoryDb {
        async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn get(&self, key: &str) -> CmdbResult<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn delete(&self, key: &str) -> CmdbResult<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn list_keys(&self, prefix: &str) -> CmdbResult<Vec<String>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .keys()
                .filter(|key| key.starts_with(prefix))
                .cloned()
                .collect())
        }

        async fn list_values(&self, prefix: &str) -> CmdbResult<Vec<Vec<u8>>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(_, value)| value.clone())
                .collect())
        }

        async fn list_entries(&self, prefix: &str) -> CmdbResult<Vec<(String, Vec<u8>)>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .iter()
                .filter(|(key, _)| key.starts_with(prefix))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect())
        }

        async fn exists(&self, key: &str) -> CmdbResult<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        async fn update_all(
            &self,
            prefix: &str,
            callback: Box<dyn Fn(String, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>,
        ) -> CmdbResult<()> {
            let snapshot = self.data.lock().unwrap().clone();
            let mut updates = Vec::new();
            for (key, value) in snapshot {
                if key.starts_with(prefix) {
                    if let Some(new_value) = callback(key.clone(), value) {
                        updates.push((key, new_value));
                    }
                }
            }
            let mut data = self.data.lock().unwrap();
            for (key, value) in updates {
                data.insert(key, value);
            }
            Ok(())
        }
    }

    async fn build_service() -> Arc<TerminalSessionService> {
        let db: Arc<dyn Database> = Arc::new(MemoryDb::default());
        let repo = Arc::new(TerminalSessionRepository::new(db.clone()));
        let policy_repo = Arc::new(WebTerminalPolicyRepository::new(db.clone()));
        policy_repo
            .save(&WebTerminalPolicy {
                id: "policy-1".into(),
                name: "default".into(),
                description: "default test policy".into(),
                subject_type: SubjectType::Role,
                subject_id: "User".into(),
                target_scope: TargetScope::All,
                mode: TerminalMode::ReadWrite,
                terminal_command_rules: CommandRules::default(),
                allowed_commands: vec![],
                session_timeout_secs: 300,
                max_concurrent_sessions: 5,
                require_approval: false,
                priority: 1,
            })
            .await
            .unwrap();
        let policy = Arc::new(WebTerminalService::new(policy_repo));
        let svc = TerminalSessionService::new(repo, policy);
        let exec_repo = Arc::new(ExecutionSessionRepository::new(db.clone()));
        let exec_svc = Arc::new(ExecutionSessionService::new(
            exec_repo,
            Arc::new(crate::repository::client_repository::ClientRepository::new(
                db.clone(),
            )),
        ));
        svc.configure_history(exec_svc);
        svc
    }

    #[tokio::test]
    async fn closing_pending_terminal_session_marks_history_failed() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-hist".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-1",
                "user-1",
                &Role::User,
            )
            .await
            .unwrap();

        svc.close_session(&session.session_id, Some("user left terminal page".into()))
            .await
            .unwrap();

        let session_svc = svc
            .session_svc
            .lock()
            .expect("terminal session history lock")
            .as_ref()
            .unwrap()
            .clone();
        let history = session_svc
            .get_session(&session.session_id)
            .await
            .unwrap()
            .expect("execution history should exist");
        assert_eq!(history.status, SessionStatus::Success);
    }

    #[tokio::test]
    async fn close_request_is_exposed_to_agent_poll() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-a".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-1",
                "user-1",
                &Role::User,
            )
            .await
            .unwrap();
        svc.report_state(
            &session.session_id,
            AgentTerminalStateRequest {
                claim_id: Some("claim-a".into()),
                state: TerminalSessionState::Active,
                message: None,
            },
        )
        .await
        .unwrap();

        svc.close_session(&session.session_id, Some("closed by operator".into()))
            .await
            .unwrap();

        let poll = svc
            .poll_for_agent("client-a", Some(&session.session_id), Some("claim-a"))
            .await
            .unwrap()
            .unwrap();
        assert!(poll.close_requested);
        assert_eq!(poll.session.session_id, session.session_id);
    }

    #[tokio::test]
    async fn stale_active_session_is_closed_and_skipped() {
        let svc = build_service().await;
        let mut session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-b".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-2",
                "user-2",
                &Role::User,
            )
            .await
            .unwrap();
        let stale = (Utc::now() - chrono::Duration::seconds(360)).to_rfc3339();
        session.state = TerminalSessionState::Active;
        session.activated_at = Some(stale.clone());
        session.last_activity_at = Some(stale);
        svc.repo.save_session(&session).await.unwrap();

        let poll = svc
            .poll_for_agent("client-b", None, Some("claim-stale"))
            .await
            .unwrap();
        assert!(poll.is_none());

        let stored = svc.get_session(&session.session_id).await.unwrap();
        assert_eq!(stored.state, TerminalSessionState::Closed);
        assert_eq!(stored.close_reason.as_deref(), Some("agent heartbeat lost"));
    }

    #[tokio::test]
    async fn late_heartbeat_does_not_reopen_closed_session() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-heartbeat".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-heartbeat",
                "user-heartbeat",
                &Role::User,
            )
            .await
            .unwrap();
        svc.report_state(
            &session.session_id,
            AgentTerminalStateRequest {
                claim_id: Some("claim-heartbeat".into()),
                state: TerminalSessionState::Active,
                message: None,
            },
        )
        .await
        .unwrap();
        svc.report_state(
            &session.session_id,
            AgentTerminalStateRequest {
                claim_id: Some("claim-heartbeat".into()),
                state: TerminalSessionState::Closed,
                message: Some("agent exited".into()),
            },
        )
        .await
        .unwrap();

        svc.heartbeat_for_client(&session.session_id, "client-heartbeat", "claim-heartbeat")
            .await
            .unwrap();
        let stored = svc.get_session(&session.session_id).await.unwrap();
        assert_eq!(stored.state, TerminalSessionState::Closed);
        assert!(stored.lease_id.is_none());
    }

    #[tokio::test]
    async fn late_agent_output_and_state_cannot_overwrite_new_claim() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-claim-race".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-claim-race",
                "user-claim-race",
                &Role::User,
            )
            .await
            .unwrap();

        svc.poll_for_agent(
            "client-claim-race",
            Some(&session.session_id),
            Some("claim-old"),
        )
        .await
        .unwrap()
        .unwrap();

        let mut expired = svc.get_session(&session.session_id).await.unwrap();
        expired.lease_expires_at = Some((Utc::now() - chrono::Duration::seconds(1)).to_rfc3339());
        svc.repo.save_session(&expired).await.unwrap();

        svc.poll_for_agent(
            "client-claim-race",
            Some(&session.session_id),
            Some("claim-new"),
        )
        .await
        .unwrap()
        .unwrap();

        let output_result = svc
            .report_output_for_client(
                &session.session_id,
                "client-claim-race",
                AgentTerminalOutputRequest {
                    claim_id: Some("claim-old".into()),
                    chunks: vec![TerminalOutputChunk {
                        seq: 1,
                        data: "late output".into(),
                        timestamp: Utc::now().to_rfc3339(),
                    }],
                },
            )
            .await;
        assert!(output_result.is_err());

        let state_result = svc
            .report_state_for_client(
                &session.session_id,
                "client-claim-race",
                AgentTerminalStateRequest {
                    claim_id: Some("claim-old".into()),
                    state: TerminalSessionState::Closed,
                    message: Some("late close".into()),
                },
            )
            .await;
        assert!(state_result.is_err());

        let stored = svc.get_session(&session.session_id).await.unwrap();
        assert_eq!(stored.state, TerminalSessionState::Pending);
        assert_eq!(stored.lease_id.as_deref(), Some("claim-new"));
        assert!(
            svc.get_output(&session.session_id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn create_session_always_creates_new_open_session_for_same_user_and_client() {
        let svc = build_service().await;
        let first = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-reuse".into(),
                    shell: Some("/bin/bash".into()),
                    cols: Some(120),
                    rows: Some(32),
                },
                "user-1",
                "user-1",
                &Role::User,
            )
            .await
            .unwrap();

        let second = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-reuse".into(),
                    shell: Some("/bin/zsh".into()),
                    cols: Some(140),
                    rows: Some(40),
                },
                "user-1",
                "user-1",
                &Role::User,
            )
            .await
            .unwrap();

        assert_ne!(first.session_id, second.session_id);
        assert_eq!(first.shell, "/bin/bash");
        assert_eq!(second.shell, "/bin/zsh");
        assert_eq!(second.cols, 140);
        assert_eq!(second.rows, 40);
    }

    #[tokio::test]
    async fn active_session_requires_matching_claim_until_lease_expires() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-claim".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-9",
                "user-9",
                &Role::User,
            )
            .await
            .unwrap();

        let first_poll = svc
            .poll_for_agent("client-claim", Some(&session.session_id), Some("claim-a"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(first_poll.session.lease_id.as_deref(), Some("claim-a"));

        let second_poll = svc
            .poll_for_agent("client-claim", Some(&session.session_id), Some("claim-b"))
            .await
            .unwrap();
        assert!(second_poll.is_none());

        let mut stored = svc.get_session(&session.session_id).await.unwrap();
        stored.lease_expires_at = Some((Utc::now() - chrono::Duration::seconds(1)).to_rfc3339());
        svc.repo.save_session(&stored).await.unwrap();

        let reclaimed = svc
            .poll_for_agent("client-claim", Some(&session.session_id), Some("claim-b"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reclaimed.session.lease_id.as_deref(), Some("claim-b"));
    }

    #[tokio::test]
    async fn list_client_sessions_filters_non_admin_results_and_sorts_by_state_then_activity() {
        let svc = build_service().await;

        let own_pending = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-list".into(),
                    shell: Some("/bin/bash".into()),
                    cols: Some(100),
                    rows: Some(30),
                },
                "user-a",
                "user-a",
                &Role::User,
            )
            .await
            .unwrap();

        let other_active = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-list".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-b",
                "user-b",
                &Role::User,
            )
            .await
            .unwrap();

        let other_pending = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-list".into(),
                    shell: Some("/bin/zsh".into()),
                    cols: Some(120),
                    rows: Some(32),
                },
                "user-c",
                "user-c",
                &Role::User,
            )
            .await
            .unwrap();

        let oldest_ts = (Utc::now() - chrono::Duration::seconds(120)).to_rfc3339();
        let older_ts = (Utc::now() - chrono::Duration::seconds(60)).to_rfc3339();
        let newest_ts = Utc::now().to_rfc3339();

        let mut own_pending = svc.get_session(&own_pending.session_id).await.unwrap();
        own_pending.last_activity_at = Some(older_ts.clone());
        svc.repo.save_session(&own_pending).await.unwrap();

        let mut other_active = svc.get_session(&other_active.session_id).await.unwrap();
        other_active.state = TerminalSessionState::Active;
        other_active.activated_at = Some(newest_ts.clone());
        other_active.last_activity_at = Some(newest_ts.clone());
        other_active.last_heartbeat_at = Some(newest_ts);
        svc.repo.save_session(&other_active).await.unwrap();

        let mut other_pending = svc.get_session(&other_pending.session_id).await.unwrap();
        other_pending.last_activity_at = Some(oldest_ts);
        svc.repo.save_session(&other_pending).await.unwrap();

        let user_sessions = svc
            .list_client_sessions("client-list", "user-a", false)
            .await
            .unwrap();
        let user_ids = user_sessions
            .iter()
            .map(|session| session.session_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(user_ids, vec![own_pending.session_id.as_str()]);

        let admin_sessions = svc
            .list_client_sessions("client-list", "admin-1", true)
            .await
            .unwrap();
        let admin_ids = admin_sessions
            .iter()
            .map(|session| session.session_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            admin_ids,
            vec![
                other_active.session_id.as_str(),
                own_pending.session_id.as_str(),
                other_pending.session_id.as_str(),
            ]
        );
    }

    #[tokio::test]
    async fn output_from_mismatched_claim_is_rejected_before_persisting_chunks() {
        let svc = build_service().await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-output-claim".into(),
                    shell: Some("/bin/sh".into()),
                    cols: Some(80),
                    rows: Some(24),
                },
                "user-3",
                "user-3",
                &Role::User,
            )
            .await
            .unwrap();

        svc.poll_for_agent(
            "client-output-claim",
            Some(&session.session_id),
            Some("claim-a"),
        )
        .await
        .unwrap()
        .unwrap();

        let err = svc
            .report_output(
                &session.session_id,
                AgentTerminalOutputRequest {
                    claim_id: Some("claim-b".into()),
                    chunks: vec![TerminalOutputChunk {
                        seq: 1,
                        data: "forbidden".into(),
                        timestamp: Utc::now().to_rfc3339(),
                    }],
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CmdbError::Forbidden(_)));
        assert!(
            svc.get_output(&session.session_id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    async fn build_read_only_service(allowed_commands: Vec<&str>) -> Arc<TerminalSessionService> {
        let db: Arc<dyn Database> = Arc::new(MemoryDb::default());
        let repo = Arc::new(TerminalSessionRepository::new(db.clone()));
        let policy_repo = Arc::new(WebTerminalPolicyRepository::new(db));
        policy_repo
            .save(&WebTerminalPolicy {
                id: "policy-ro".into(),
                name: "read-only".into(),
                description: "read-only test policy".into(),
                subject_type: SubjectType::Role,
                subject_id: "User".into(),
                target_scope: TargetScope::All,
                mode: TerminalMode::ReadOnly,
                terminal_command_rules: CommandRules::default(),
                allowed_commands: allowed_commands.into_iter().map(str::to_string).collect(),
                session_timeout_secs: 300,
                max_concurrent_sessions: 5,
                require_approval: false,
                priority: 1,
            })
            .await
            .unwrap();
        let policy = Arc::new(WebTerminalService::new(policy_repo));
        TerminalSessionService::new(repo, policy)
    }

    #[tokio::test]
    async fn browser_input_read_only_waits_for_newline_before_queueing() {
        let svc = build_read_only_service(vec!["ls", "pwd"]).await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-browser-ro".into(),
                    shell: Some("/bin/bash".into()),
                    cols: Some(120),
                    rows: Some(32),
                },
                "user-ro",
                "user-ro",
                &Role::User,
            )
            .await
            .unwrap();

        let mut pending_line = String::new();
        svc.queue_browser_input(&session.session_id, "ls -l".into(), &mut pending_line)
            .await
            .unwrap();

        let first_poll = svc
            .poll_for_agent(
                "client-browser-ro",
                Some(&session.session_id),
                Some("claim-ro"),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(first_poll.pending_input.is_empty());
        assert_eq!(pending_line, "ls -l");

        svc.queue_browser_input(&session.session_id, "\npwd\n".into(), &mut pending_line)
            .await
            .unwrap();

        let second_poll = svc
            .poll_for_agent(
                "client-browser-ro",
                Some(&session.session_id),
                Some("claim-ro"),
            )
            .await
            .unwrap()
            .unwrap();
        let queued = second_poll
            .pending_input
            .iter()
            .map(|chunk| chunk.data.as_str())
            .collect::<Vec<_>>();
        assert_eq!(queued, vec!["ls -l\npwd\n"]);
        assert!(pending_line.is_empty());
    }

    #[tokio::test]
    async fn browser_input_read_only_rejects_disallowed_completed_line() {
        let svc = build_read_only_service(vec!["ls"]).await;
        let session = svc
            .create_session(
                &CreateTerminalSessionRequest {
                    client_id: "client-browser-deny".into(),
                    shell: Some("/bin/bash".into()),
                    cols: Some(120),
                    rows: Some(32),
                },
                "user-deny",
                "user-deny",
                &Role::User,
            )
            .await
            .unwrap();

        let mut pending_line = String::new();
        let err = svc
            .queue_browser_input(&session.session_id, "hostname\n".into(), &mut pending_line)
            .await
            .unwrap_err();

        assert!(matches!(err, CmdbError::Forbidden(_)));
        assert_eq!(pending_line, "hostname\n");

        let poll = svc
            .poll_for_agent(
                "client-browser-deny",
                Some(&session.session_id),
                Some("claim-deny"),
            )
            .await
            .unwrap()
            .unwrap();
        assert!(poll.pending_input.is_empty());
    }
}
