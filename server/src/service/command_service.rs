//! Command service: orchestrates task creation, agent dispatch, log ingestion,
//! and completion handling for the remote command execution feature.

use crate::repository::command_repository::CommandRepository;
use crate::service::approval_service::ApprovalService;
use crate::service::cast_recorder::CastRecorder;
use crate::service::danger_detection::DangerDetectionService;
use crate::service::exec_policy_engine::{ExecPolicyEngine, PolicyDecision};
use crate::service::execution_session_service::ExecutionSessionService;
use crate::service::sse_hub::SseHub;
use chrono::{DateTime, Utc};
use common::command::{
    AuditAction, AuditLogEntry, CommandStatus, CommandTask, normalise_command, split_command,
};
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};
use common::entity::permission::{ApprovalCommandPayload, ApprovalPayload};
use common::entity::user::Role;
use common::error::{CmdbError, CmdbResult};
use common::models::{
    AgentCompleteRequest, AgentLogRequest, CreateCommandRequest, CreateCommandResponse,
    MAX_COMMAND_LENGTH, RemoteExecConfigResponse, UpdateRemoteExecConfigRequest,
};
use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// 10 MB per-task log cap (in bytes of stored JSON).
const LOG_CAP_BYTES: usize = 10 * 1024 * 1024;
const DISPATCH_LEASE_SECS: i64 = 90;

pub struct CommandService {
    repo: Arc<CommandRepository>,
    danger: Arc<DangerDetectionService>,
    sse: Arc<SseHub>,
    /// In-memory fast switch; mirrors `config:remote_exec_enabled` in Redb.
    enabled: Arc<AtomicBool>,
    policy_engine: Option<Arc<ExecPolicyEngine>>,
    approval_svc: Option<Arc<ApprovalService>>,
    session_svc: Option<Arc<ExecutionSessionService>>,
    cast_recorder: Option<CastRecorder>,
}

impl CommandService {
    pub async fn new(
        repo: Arc<CommandRepository>,
        danger: Arc<DangerDetectionService>,
        sse: Arc<SseHub>,
    ) -> CmdbResult<Self> {
        // Bootstrap the in-memory switch from persisted config.
        let persisted = repo.get_remote_exec_enabled().await.unwrap_or(false);
        let enabled = Arc::new(AtomicBool::new(persisted));
        Ok(Self {
            repo,
            danger,
            sse,
            enabled,
            policy_engine: None,
            approval_svc: None,
            session_svc: None,
            cast_recorder: None,
        })
    }

    pub fn with_policy_engine(mut self, engine: Arc<ExecPolicyEngine>) -> Self {
        self.policy_engine = Some(engine);
        self
    }

    pub fn with_approval_svc(mut self, svc: Arc<ApprovalService>) -> Self {
        self.approval_svc = Some(svc);
        self
    }

    pub fn with_session_svc(mut self, svc: Arc<ExecutionSessionService>) -> Self {
        self.session_svc = Some(svc);
        self
    }

    pub fn with_cast_recorder(mut self, recorder: CastRecorder) -> Self {
        self.cast_recorder = Some(recorder);
        self
    }

    // ── global switch ─────────────────────────────────────────────────────────

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub async fn get_config(&self) -> CmdbResult<RemoteExecConfigResponse> {
        Ok(RemoteExecConfigResponse {
            enabled: self.is_enabled(),
        })
    }

    pub async fn update_config(
        &self,
        req: UpdateRemoteExecConfigRequest,
        operator: &str,
    ) -> CmdbResult<RemoteExecConfigResponse> {
        self.enabled.store(req.enabled, Ordering::Relaxed);
        self.repo.set_remote_exec_enabled(req.enabled).await?;

        // Audit: config change
        let audit = AuditLogEntry {
            id: 0,
            action: AuditAction::ConfigChange,
            operator: operator.to_string(),
            task_id: None,
            client_id: None,
            detail: format!("Remote exec enabled = {}", req.enabled),
            created_at: Utc::now().to_rfc3339(),
        };
        let _ = self.repo.append_audit(&audit).await;

        Ok(RemoteExecConfigResponse {
            enabled: req.enabled,
        })
    }

    // ── task creation ─────────────────────────────────────────────────────────

    /// Create a new command task.
    ///
    /// Returns `Err(Forbidden)` if the global switch is OFF and the request
    /// does not supply `force = true` (only Admins can override).
    pub async fn create_task(
        &self,
        req: &CreateCommandRequest,
        submitted_by: &str,
        allow_force: bool,
        user_id: &str,
        user_role: &Role,
        group_ids: &[String],
    ) -> CmdbResult<CreateCommandResponse> {
        self.create_task_internal(
            req,
            submitted_by,
            allow_force,
            user_id,
            user_role,
            group_ids,
            false,
        )
        .await
    }

    async fn create_task_internal(
        &self,
        req: &CreateCommandRequest,
        submitted_by: &str,
        allow_force: bool,
        user_id: &str,
        user_role: &Role,
        group_ids: &[String],
        bypass_approval: bool,
    ) -> CmdbResult<CreateCommandResponse> {
        if req.command.len() > MAX_COMMAND_LENGTH {
            return Err(CmdbError::Client(format!(
                "Command too long: {} bytes (max {})",
                req.command.len(),
                MAX_COMMAND_LENGTH
            )));
        }

        if !(self.is_enabled() || req.force && allow_force) {
            return Err(CmdbError::Forbidden(
                "Remote command execution is disabled".to_string(),
            ));
        }

        // Layer 2: ExecPolicy check
        if let Some(ref engine) = self.policy_engine {
            let policy_decision = if req.shell_mode {
                engine
                    .evaluate_script(user_id, user_role, group_ids, &req.client_id, &req.command)
                    .await
            } else {
                engine
                    .evaluate(user_id, user_role, group_ids, &req.client_id, &req.command)
                    .await
            };
            match policy_decision {
                PolicyDecision::Denied(msg) => {
                    let audit = AuditLogEntry {
                        id: 0,
                        action: AuditAction::CommandBlocked,
                        operator: submitted_by.to_string(),
                        task_id: None,
                        client_id: Some(req.client_id.clone()),
                        detail: format!("Blocked by exec policy: {}", msg),
                        created_at: Utc::now().to_rfc3339(),
                    };
                    let _ = self.repo.append_audit(&audit).await;
                    return Err(CmdbError::Forbidden(msg));
                }
                PolicyDecision::Warning => {
                    if !req.force && !bypass_approval {
                        if self.approval_svc.is_none() {
                            return Ok(CreateCommandResponse {
                                task_id: None,
                                approval_id: None,
                                danger_level: "policy_warning".to_string(),
                                requires_confirmation: true,
                                matched_rule: None,
                                message: Some(
                                    "Command requires confirmation per execution policy"
                                        .to_string(),
                                ),
                            });
                        }
                        let approval = self
                            .create_command_approval(
                                "exec_policy_warn",
                                "policy_warning",
                                user_id,
                                submitted_by,
                                user_role,
                                group_ids,
                                req,
                            )
                            .await?;
                        return Ok(CreateCommandResponse {
                            task_id: None,
                            approval_id: Some(approval.id),
                            danger_level: "policy_warning".to_string(),
                            requires_confirmation: true,
                            matched_rule: None,
                            message: Some("Command requires admin approval because execution policy matched Warn".to_string()),
                        });
                    }
                }
                PolicyDecision::Allowed { require_approval } => {
                    if require_approval && !bypass_approval {
                        if self.approval_svc.is_some() {
                            let approval = self
                                .create_command_approval(
                                    "exec_policy",
                                    "policy",
                                    user_id,
                                    submitted_by,
                                    user_role,
                                    group_ids,
                                    req,
                                )
                                .await?;
                            return Ok(CreateCommandResponse {
                                task_id: None,
                                approval_id: Some(approval.id),
                                danger_level: "requires_approval".to_string(),
                                requires_confirmation: true,
                                matched_rule: None,
                                message: Some(
                                    "Command requires admin approval before execution".to_string(),
                                ),
                            });
                        }
                        // No approval service - fall through to execution
                    }
                }
            }
        }

        let (cmd, args) = if req.shell_mode {
            req.command
                .lines()
                .find(|line| !line.trim().is_empty())
                .map(split_command)
                .unwrap_or_else(|| (String::new(), vec![]))
        } else {
            normalise_command(&req.command, req.args.clone().unwrap_or_default())
        };
        let detection = if req.shell_mode {
            analyse_shell_script(&self.danger, &req.command)
        } else {
            self.danger.analyse(&cmd, &args)
        };

        // Blocked commands are NEVER queued, regardless of force=true.
        // This is a hard security boundary — not a confirmation gate.
        if detection.level == common::command::DangerLevel::Blocked {
            let audit = AuditLogEntry {
                id: 0,
                action: AuditAction::CommandBlocked,
                operator: submitted_by.to_string(),
                task_id: None,
                client_id: Some(req.client_id.clone()),
                detail: format!(
                    "Blocked by rule '{}': {}",
                    detection.matched_rule.as_deref().unwrap_or("?"),
                    req.command
                ),
                created_at: Utc::now().to_rfc3339(),
            };
            let _ = self.repo.append_audit(&audit).await;
            return Ok(CreateCommandResponse {
                task_id: None,
                approval_id: None,
                danger_level: detection.level.to_string(),
                requires_confirmation: false,
                matched_rule: detection.matched_rule,
                message: Some(detection.message),
            });
        }

        // Warning commands require `force = true` to queue.
        let requires_confirmation =
            detection.level == common::command::DangerLevel::Warning && !req.force;

        if requires_confirmation {
            return Ok(CreateCommandResponse {
                task_id: None,
                approval_id: None,
                danger_level: detection.level.to_string(),
                requires_confirmation: true,
                matched_rule: detection.matched_rule,
                message: Some(detection.message),
            });
        }

        let timeout = req.timeout_secs.unwrap_or(60);
        let mut task = CommandTask::new(
            req.client_id.clone(),
            if req.shell_mode {
                req.command.clone()
            } else {
                cmd
            },
            if req.shell_mode { vec![] } else { args },
            submitted_by.to_string(),
            detection.level.clone(),
            Some(timeout),
        );
        task.shell_mode = req.shell_mode;
        task.submitted_by_id = Some(user_id.to_string());

        self.repo.save_task(&task).await?;

        // Create execution session if available
        if let Some(ref session_svc) = self.session_svc {
            let session = ExecutionSession::new(
                user_id,
                submitted_by,
                vec![req.client_id.clone()],
                &req.command,
                req.execution_type
                    .clone()
                    .unwrap_or(ExecutionType::Terminal),
            );
            // Use task_id as session_id for 1:1 mapping
            let mut session_with_task_id = session;
            session_with_task_id.session_id = task.id.clone();
            if let Err(e) = session_svc.save_session(&session_with_task_id).await {
                tracing::warn!("Failed to save execution session: {}", e);
            }

            // Initialize cast file
            if self.cast_recorder.is_some() {
                use crate::service::cast_recorder::CastRecorderInner;
                if CastRecorderInner::init_cast(&task.id, 80, 24).is_ok() {
                    let cast_path = CastRecorderInner::cast_path(&task.id)
                        .to_string_lossy()
                        .to_string();
                    let _ = session_svc.update_cast_path(&task.id, &cast_path).await;
                }
            }
        }

        // Audit: command queued
        let audit = AuditLogEntry {
            id: 0,
            action: AuditAction::CommandCreate,
            operator: submitted_by.to_string(),
            task_id: Some(task.id.clone()),
            client_id: Some(req.client_id.clone()),
            detail: format!("Queued: {} (danger={})", req.command, detection.level),
            created_at: Utc::now().to_rfc3339(),
        };
        let _ = self.repo.append_audit(&audit).await;

        Ok(CreateCommandResponse {
            task_id: Some(task.id),
            approval_id: None,
            danger_level: detection.level.to_string(),
            requires_confirmation: false,
            matched_rule: detection.matched_rule,
            message: if detection.message.is_empty() {
                None
            } else {
                Some(detection.message)
            },
        })
    }

    async fn create_command_approval(
        &self,
        policy_type: &str,
        policy_id: &str,
        user_id: &str,
        submitted_by: &str,
        user_role: &Role,
        group_ids: &[String],
        req: &CreateCommandRequest,
    ) -> CmdbResult<common::entity::permission::PendingApproval> {
        let approval_svc = self
            .approval_svc
            .as_ref()
            .ok_or_else(|| CmdbError::Internal("Approval service unavailable".to_string()))?;

        approval_svc
            .create_approval(
                policy_type,
                policy_id,
                user_id,
                submitted_by,
                &req.client_id,
                &req.command,
                Some(ApprovalPayload::Command {
                    request: ApprovalCommandPayload {
                        client_id: req.client_id.clone(),
                        command: req.command.clone(),
                        submitted_role: user_role.clone(),
                        group_ids: group_ids.to_vec(),
                        shell_mode: req.shell_mode,
                        args: req.args.clone(),
                        execution_type: req.execution_type.clone(),
                        timeout_secs: req.timeout_secs,
                    },
                }),
                3600,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to create approval request: {}", e);
                CmdbError::Internal("Failed to create approval request".to_string())
            })
    }

    pub async fn create_task_from_approval(
        &self,
        req: &CreateCommandRequest,
        submitted_by: &str,
        user_id: &str,
        user_role: &Role,
        group_ids: &[String],
    ) -> CmdbResult<CreateCommandResponse> {
        let mut approved_req = req.clone();
        approved_req.force = true;
        self.create_task_internal(
            &approved_req,
            submitted_by,
            true,
            user_id,
            user_role,
            group_ids,
            true,
        )
        .await
    }

    pub async fn delete_task(&self, task_id: &str) -> CmdbResult<()> {
        let mut first_error = None;
        if let Err(error) = self.repo.delete_task(task_id).await {
            first_error = Some(error);
        }

        // An execution task creates same-ID history records and possibly a
        // Cast file. Remove those records as part of approval compensation so
        // a failed attach cannot leave an apparently executable orphan.
        if let Some(session_svc) = &self.session_svc
            && let Err(error) = session_svc.delete_session(task_id).await
            && first_error.is_none()
        {
            first_error = Some(error);
        }
        if self.cast_recorder.is_some()
            && let Err(error) =
                crate::service::cast_recorder::CastRecorderInner::delete_cast(task_id)
            && first_error.is_none()
        {
            first_error = Some(error);
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    // ── agent: fetch pending ──────────────────────────────────────────────────

    /// Called by agent long-poll: returns the first Pending task for the client.
    /// Returns `Ok(None)` when no pending task exists (agent should re-poll).
    #[allow(dead_code)]
    pub async fn get_pending_for_client(&self, client_id: &str) -> CmdbResult<Option<CommandTask>> {
        if !self.is_enabled() {
            return Ok(None);
        }
        self.repo.get_pending_task_for_client(client_id).await
    }

    /// Claim one pending task for an authenticated agent. The repository
    /// performs the conditional write, so two pollers cannot receive the same
    /// task at the same time.
    pub async fn claim_pending_for_client(
        &self,
        client_id: &str,
    ) -> CmdbResult<Option<CommandTask>> {
        if !self.is_enabled() {
            return Ok(None);
        }
        let now = Utc::now();
        let lease_expires_at = now + chrono::Duration::seconds(DISPATCH_LEASE_SECS);
        self.repo
            .claim_pending_task_for_client(
                client_id,
                &now.to_rfc3339(),
                &lease_expires_at.to_rfc3339(),
            )
            .await
    }

    // ── agent: mark started ───────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn mark_running(&self, task_id: &str) -> CmdbResult<()> {
        let mut task = self
            .repo
            .get_task(task_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Task {} not found", task_id)))?;

        task.status = CommandStatus::Running;
        task.started_at = Some(Utc::now().to_rfc3339());
        self.repo.update_task(&task).await
    }

    /// Start a task only for the agent that acquired its dispatch lease.
    pub async fn mark_running_for_client(&self, task_id: &str, client_id: &str) -> CmdbResult<()> {
        let now = Utc::now().to_rfc3339();
        match self
            .repo
            .start_claimed_task(task_id, client_id, &now)
            .await?
        {
            Some(_) => Ok(()),
            None => {
                let task =
                    self.repo.get_task(task_id).await?.ok_or_else(|| {
                        CmdbError::NotFound(format!("Task {} not found", task_id))
                    })?;
                if task.client_id != client_id {
                    return Err(CmdbError::Forbidden(
                        "task does not belong to this agent".into(),
                    ));
                }
                Err(CmdbError::Forbidden(
                    "task is no longer available to this agent".into(),
                ))
            }
        }
    }

    // ── agent: push log lines ─────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn push_logs(&self, task_id: &str, req: AgentLogRequest) -> CmdbResult<()> {
        self.push_logs_inner(task_id, req, None).await
    }

    pub async fn push_logs_for_client(
        &self,
        task_id: &str,
        client_id: &str,
        req: AgentLogRequest,
    ) -> CmdbResult<()> {
        self.push_logs_inner(task_id, req, Some(client_id)).await
    }

    async fn push_logs_inner(
        &self,
        task_id: &str,
        req: AgentLogRequest,
        client_id: Option<&str>,
    ) -> CmdbResult<()> {
        let task = self
            .repo
            .get_task(task_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Task {} not found", task_id)))?;
        if let Some(client_id) = client_id {
            ensure_agent_task_owner(&task, client_id)?;
        }
        if client_id.is_some() && task.status != CommandStatus::Running {
            return Err(CmdbError::Forbidden("task is not running".into()));
        }
        if client_id.is_some() && running_task_deadline_elapsed(&task, Utc::now()) {
            return Err(CmdbError::Forbidden(
                "task execution deadline has elapsed".into(),
            ));
        }

        // Enforce 10 MB cap
        let current_bytes = self.repo.count_log_bytes(task_id).await?;
        if current_bytes >= LOG_CAP_BYTES {
            self.repo.mark_task_truncated(task_id).await?;
            return Ok(()); // silently drop additional logs
        }

        // Cap the incoming batch as well as the already-persisted log. A
        // single oversized request must not bypass the per-task limit.
        let mut accepted = Vec::new();
        let mut used_bytes = current_bytes;
        let mut truncated = false;
        for line in &req.lines {
            let line_bytes = serde_json::to_vec(line)
                .map_err(|e| CmdbError::Serialization(format!("serialize command log: {}", e)))?;
            if used_bytes.saturating_add(line_bytes.len()) > LOG_CAP_BYTES {
                truncated = true;
                break;
            }
            used_bytes += line_bytes.len();
            accepted.push(line.clone());
        }
        if truncated {
            self.repo.mark_task_truncated(task_id).await?;
        }
        if accepted.is_empty() {
            return Ok(());
        }

        self.repo.append_logs(task_id, &accepted).await?;
        self.sse.publish(task_id, &accepted).await;

        // Append logs to cast file if available
        if self.cast_recorder.is_some() {
            use crate::service::cast_recorder::CastRecorderInner;
            if CastRecorderInner::cast_file_exists(task_id) {
                for line in &accepted {
                    let timestamp = cast_timestamp_for_task(&task, &line.timestamp);
                    let output = cast_output_for_log_line(&line.line);
                    let _ = CastRecorderInner::append_frame(task_id, timestamp, &output);
                }
            }
        }

        Ok(())
    }

    // ── agent: complete task ──────────────────────────────────────────────────

    #[allow(dead_code)]
    pub async fn complete_task(&self, task_id: &str, req: AgentCompleteRequest) -> CmdbResult<()> {
        self.complete_task_inner(task_id, req, None).await
    }

    pub async fn complete_task_for_client(
        &self,
        task_id: &str,
        client_id: &str,
        req: AgentCompleteRequest,
    ) -> CmdbResult<()> {
        self.complete_task_inner(task_id, req, Some(client_id))
            .await
    }

    async fn complete_task_inner(
        &self,
        task_id: &str,
        req: AgentCompleteRequest,
        client_id: Option<&str>,
    ) -> CmdbResult<()> {
        let mut task = self
            .repo
            .get_task(task_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Task {} not found", task_id)))?;
        if let Some(client_id) = client_id {
            ensure_agent_task_owner(&task, client_id)?;
        }
        if client_id.is_some() && task.status != CommandStatus::Running {
            return Err(CmdbError::Forbidden("task is not running".into()));
        }

        let status = match req.status.as_str() {
            "success" => CommandStatus::Success,
            "failed" => CommandStatus::Failed,
            "timeout" => CommandStatus::Timeout,
            _ => CommandStatus::Failed,
        };
        let completed_at = Utc::now().to_rfc3339();
        if let Some(client_id) = client_id {
            task = self
                .repo
                .complete_running_task(task_id, client_id, status, req.exit_code, &completed_at)
                .await?
                .ok_or_else(|| CmdbError::Forbidden("task is no longer running".into()))?;
        } else {
            task.status = status;
            task.exit_code = Some(req.exit_code);
            task.completed_at = Some(completed_at);
            self.repo.update_task(&task).await?;
        }

        // Audit: task completed
        let audit = AuditLogEntry {
            id: 0,
            action: AuditAction::CommandCompleted,
            operator: task.submitted_by.clone(),
            task_id: Some(task_id.to_string()),
            client_id: Some(task.client_id.clone()),
            detail: format!(
                "Completed: exit_code={}, status={}",
                req.exit_code, req.status
            ),
            created_at: Utc::now().to_rfc3339(),
        };
        let _ = self.repo.append_audit(&audit).await;

        // Update execution session status
        if let Some(ref session_svc) = self.session_svc {
            let status = match task.status {
                CommandStatus::Success => SessionStatus::Success,
                CommandStatus::Failed => SessionStatus::Failed,
                CommandStatus::Timeout => SessionStatus::Failed,
                _ => SessionStatus::Failed,
            };
            let _ = session_svc.complete_session(task_id, status).await;
        }

        // Close the SSE channel and send a done event
        self.sse.close_task(task_id).await;
        Ok(())
    }

    // ── history query ─────────────────────────────────────────────────────────

    pub async fn list_tasks(
        &self,
        client_id: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> CmdbResult<(Vec<CommandTask>, usize)> {
        let all = if let Some(cid) = client_id {
            self.repo.list_tasks_for_client(cid).await?
        } else {
            let mut tasks = self.repo.list_all_tasks().await?;
            // Sort newest first
            tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            tasks
        };

        let total = all.len();
        let offset = page.saturating_sub(1) * page_size;
        let items = all.into_iter().skip(offset).take(page_size).collect();
        Ok((items, total))
    }

    /// List task history restricted to a set of target clients. This is used
    /// for Project/Tag permission scopes; filtering after pagination would
    /// otherwise leak counts and omit eligible tasks from later pages.
    pub async fn list_tasks_for_client_ids(
        &self,
        allowed_client_ids: &HashSet<String>,
        client_id: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> CmdbResult<(Vec<CommandTask>, usize)> {
        let mut tasks = self.repo.list_all_tasks().await?;
        tasks.retain(|task| {
            allowed_client_ids.contains(&task.client_id)
                && client_id.is_none_or(|requested| task.client_id == requested)
        });
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = tasks.len();
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let items = tasks.into_iter().skip(offset).take(page_size).collect();
        Ok((items, total))
    }

    pub async fn list_tasks_for_submitter(
        &self,
        client_id: Option<&str>,
        user_id: &str,
        username: &str,
        page: usize,
        page_size: usize,
    ) -> CmdbResult<(Vec<CommandTask>, usize)> {
        let (mut tasks, _) = self.list_tasks(client_id, 1, usize::MAX).await?;
        tasks.retain(|task| {
            task.submitted_by_id.as_deref() == Some(user_id)
                || (task.submitted_by_id.is_none() && task.submitted_by == username)
        });
        let total = tasks.len();
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let items = tasks.into_iter().skip(offset).take(page_size).collect();
        Ok((items, total))
    }

    pub async fn get_task(&self, task_id: &str) -> CmdbResult<CommandTask> {
        self.repo
            .get_task(task_id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Task {} not found", task_id)))
    }

    pub async fn get_task_logs(
        &self,
        task_id: &str,
    ) -> CmdbResult<Vec<common::command::CommandLogLine>> {
        self.repo.get_logs(task_id).await
    }

    // ── cleanup ───────────────────────────────────────────────────────────────

    /// Delete tasks older than `days` days. Called by the scheduler.
    pub async fn cleanup_old_tasks(&self, days: i64) -> CmdbResult<usize> {
        let cutoff = Utc::now().timestamp() - days * 24 * 3600;
        let old = self.repo.list_tasks_older_than(cutoff).await?;
        let count = old.len();
        for task in old {
            self.repo.delete_task(&task.id).await?;
        }
        Ok(count)
    }

    // ── expire stale pending tasks ────────────────────────────────────────────

    /// Move tasks that have passed their `expires_at` from Pending → Expired.
    pub async fn expire_pending_tasks(&self) -> CmdbResult<usize> {
        let now = Utc::now();
        let all = self.repo.list_all_tasks().await?;
        let mut count = 0;
        let now_str = now.to_rfc3339();
        for task in all {
            if task.status != CommandStatus::Pending {
                continue;
            }
            if self
                .repo
                .expire_pending_task(&task.id, &now_str, &now_str)
                .await?
                .is_some()
            {
                self.sse.close_task(&task.id).await;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Mark commands whose execution deadline elapsed without a completion
    /// callback as timed out. This also prevents a crashed agent from leaving
    /// a task permanently in Running state.
    pub async fn expire_running_tasks(&self) -> CmdbResult<usize> {
        let now = Utc::now();
        let all = self.repo.list_all_tasks().await?;
        let mut count = 0;
        let now_str = now.to_rfc3339();
        for task in all {
            if task.status != CommandStatus::Running {
                continue;
            }
            let Some(task) = self
                .repo
                .timeout_running_task(&task.id, &now_str, &now_str)
                .await?
            else {
                continue;
            };
            if let Some(ref session_svc) = self.session_svc {
                let _ = session_svc
                    .complete_session(&task.id, SessionStatus::Failed)
                    .await;
            }
            self.sse.close_task(&task.id).await;
            count += 1;
        }
        Ok(count)
    }
}

fn ensure_agent_task_owner(task: &CommandTask, client_id: &str) -> CmdbResult<()> {
    if task.client_id != client_id {
        return Err(CmdbError::Forbidden(
            "task does not belong to this agent".into(),
        ));
    }
    if task.claimed_by.as_deref() != Some(client_id) {
        return Err(CmdbError::Forbidden(
            "task has not been claimed by this agent".into(),
        ));
    }
    Ok(())
}

fn running_task_deadline_elapsed(task: &CommandTask, now: chrono::DateTime<Utc>) -> bool {
    let Some(started_at) = task.started_at.as_deref() else {
        return false;
    };
    let Ok(started_at) = DateTime::parse_from_rfc3339(started_at) else {
        return false;
    };
    let timeout_secs = i64::try_from(task.timeout_secs).unwrap_or(i64::MAX);
    started_at.with_timezone(&Utc) + chrono::Duration::seconds(timeout_secs) <= now
}

fn analyse_shell_script(
    danger: &DangerDetectionService,
    script: &str,
) -> crate::service::danger_detection::DetectionResult {
    let mut warning = None;
    for line in script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let (command, args) = split_command(line);
        let result = danger.analyse(&command, &args);
        match result.level {
            common::command::DangerLevel::Blocked => return result,
            common::command::DangerLevel::Warning if warning.is_none() => warning = Some(result),
            _ => {}
        }
    }

    warning.unwrap_or(crate::service::danger_detection::DetectionResult {
        level: common::command::DangerLevel::Safe,
        matched_rule: None,
        message: "Command appears safe.".to_string(),
    })
}

fn cast_timestamp_for_task(task: &CommandTask, timestamp: &str) -> f64 {
    // Older agents sent an already elapsed floating-point timestamp. Keep
    // accepting it, but never allow malformed or negative values into the
    // asciinema stream.
    if let Ok(value) = timestamp.parse::<f64>() {
        return value.max(0.0);
    }

    let base = task
        .started_at
        .as_deref()
        .unwrap_or(task.created_at.as_str());
    let Some(start) = DateTime::parse_from_rfc3339(base).ok() else {
        return 0.0;
    };
    let Some(current) = DateTime::parse_from_rfc3339(timestamp).ok() else {
        return 0.0;
    };
    current
        .signed_duration_since(start)
        .num_microseconds()
        .unwrap_or(0)
        .max(0) as f64
        / 1_000_000.0
}

fn cast_output_for_log_line(line: &str) -> String {
    if line.ends_with('\r') || line.ends_with('\n') {
        line.to_string()
    } else {
        format!("{}\n", line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::repository::approval_repository::ApprovalRepository;
    use crate::repository::exec_policy_repository::ExecPolicyRepository;
    use crate::tests::fixtures::setup_test_db;
    use common::command::{CommandLogLine, CommandStatus, LogStream};
    use common::entity::execution::ExecutionType;
    use common::entity::permission::{
        CommandAction, CommandRules, ExecPolicy, SubjectType, TargetScope,
    };

    #[test]
    fn cast_log_lines_keep_existing_endings_and_add_missing_newlines() {
        assert_eq!(cast_output_for_log_line("first"), "first\n");
        assert_eq!(cast_output_for_log_line("second\n"), "second\n");
        assert_eq!(cast_output_for_log_line("third\r"), "third\r");
    }

    async fn setup_service() -> (Arc<CommandRepository>, Arc<SseHub>, CommandService) {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let repo = Arc::new(CommandRepository::new(db.clone()));
        let danger = Arc::new(DangerDetectionService::new());
        let sse = SseHub::new();
        let svc = CommandService::new(repo.clone(), danger, sse.clone())
            .await
            .unwrap();
        (repo, sse, svc)
    }

    async fn setup_service_with_warning_policy()
    -> (Arc<CommandRepository>, Arc<ApprovalService>, CommandService) {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);
        let repo = Arc::new(CommandRepository::new(db.clone()));
        let approval_repo = Arc::new(ApprovalRepository::new(db.clone()));
        let policy_repo = Arc::new(ExecPolicyRepository::new(db.clone()));
        let approval_svc = Arc::new(ApprovalService::new(approval_repo, repo.clone()));
        let danger = Arc::new(DangerDetectionService::new());
        let sse = SseHub::new();
        let engine = Arc::new(ExecPolicyEngine::new(policy_repo.clone()));
        policy_repo
            .save(&ExecPolicy {
                id: "warn-policy".into(),
                name: "warn-policy".into(),
                description: String::new(),
                subject_type: SubjectType::Role,
                subject_id: "Admin".into(),
                target_scope: TargetScope::All,
                command_rules: CommandRules {
                    default_action: CommandAction::Allow,
                    overrides: vec![common::entity::permission::CommandOverride {
                        pattern: "dangerous".into(),
                        action: CommandAction::Warn,
                    }],
                },
                require_approval: false,
                priority: 10,
            })
            .await
            .unwrap();

        let svc = CommandService::new(repo.clone(), danger, sse)
            .await
            .unwrap()
            .with_policy_engine(engine)
            .with_approval_svc(approval_svc.clone());

        (repo, approval_svc, svc)
    }

    #[tokio::test]
    async fn test_create_task() {
        let (repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-1".to_string(),
            command: "echo".to_string(),
            shell_mode: false,
            args: Some(vec!["hello".to_string()]),
            execution_type: Some(ExecutionType::Terminal),
            timeout_secs: Some(60),
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.expect("task_id should be Some");

        let task = repo.get_task(&task_id).await.unwrap().unwrap();
        assert_eq!(task.client_id, "client-1");
        assert_eq!(task.command, "echo");
        assert_eq!(task.submitted_by, "admin");
        assert_eq!(task.status, CommandStatus::Pending);
    }

    #[tokio::test]
    async fn test_get_task() {
        let (_repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-2".to_string(),
            command: "ls -la".to_string(),
            shell_mode: false,
            args: None,
            execution_type: Some(ExecutionType::Terminal),
            timeout_secs: None,
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.unwrap();

        let task = svc.get_task(&task_id).await.unwrap();
        assert_eq!(task.id, task_id);
        assert_eq!(task.command, "ls");
        assert_eq!(task.args, vec!["-la".to_string()]);
        assert_eq!(task.submitted_by, "admin");
    }

    #[tokio::test]
    async fn test_create_task_preserves_shell_mode_script_body() {
        let (repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-shell".to_string(),
            command: "ls -l\ndf -h".to_string(),
            shell_mode: true,
            args: None,
            execution_type: Some(ExecutionType::Batch),
            timeout_secs: Some(60),
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.unwrap();

        let task = repo.get_task(&task_id).await.unwrap().unwrap();
        assert!(task.shell_mode);
        assert_eq!(task.command, "ls -l\ndf -h");
        assert!(task.args.is_empty());
    }

    #[tokio::test]
    async fn test_warn_policy_creates_approval_request() {
        let (_repo, approval_svc, svc) = setup_service_with_warning_policy().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-warn".to_string(),
            command: "dangerous --check".to_string(),
            shell_mode: false,
            args: None,
            execution_type: Some(ExecutionType::Batch),
            timeout_secs: Some(60),
            force: false,
        };

        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();

        assert!(resp.task_id.is_none());
        let approval_id = resp.approval_id.expect("approval_id should be present");
        assert_eq!(resp.danger_level, "policy_warning");

        let approval = approval_svc
            .get(&approval_id)
            .await
            .unwrap()
            .expect("approval should exist");
        assert_eq!(approval.command, "dangerous --check");
        assert!(matches!(
            approval.payload,
            Some(ApprovalPayload::Command { .. })
        ));
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let (_repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let (tasks, total) = svc.list_tasks(None, 1, 10).await.unwrap();
        assert!(tasks.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_mark_running() {
        let (_repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-3".to_string(),
            command: "sleep 10".to_string(),
            shell_mode: false,
            args: None,
            execution_type: Some(ExecutionType::Terminal),
            timeout_secs: Some(300),
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.unwrap();

        svc.mark_running(&task_id).await.unwrap();
        let task = svc.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, CommandStatus::Running);
        assert!(task.started_at.is_some());
    }

    #[tokio::test]
    async fn test_complete_task() {
        let (_repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-4".to_string(),
            command: "echo done".to_string(),
            shell_mode: false,
            args: None,
            execution_type: Some(ExecutionType::Terminal),
            timeout_secs: None,
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.unwrap();

        svc.mark_running(&task_id).await.unwrap();
        let complete_req = AgentCompleteRequest {
            exit_code: 0,
            status: "success".to_string(),
        };
        svc.complete_task(&task_id, complete_req).await.unwrap();

        let task = svc.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, CommandStatus::Success);
        assert_eq!(task.exit_code, Some(0));
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_push_logs() {
        let (_repo, _sse, svc) = setup_service().await;
        svc.update_config(UpdateRemoteExecConfigRequest { enabled: true }, "admin")
            .await
            .unwrap();

        let req = CreateCommandRequest {
            client_id: "client-5".to_string(),
            command: "echo log-test".to_string(),
            shell_mode: false,
            args: None,
            execution_type: Some(ExecutionType::Terminal),
            timeout_secs: None,
            force: false,
        };
        let resp = svc
            .create_task(&req, "admin", false, "admin-id", &Role::Admin, &[])
            .await
            .unwrap();
        let task_id = resp.task_id.unwrap();

        let log_req = AgentLogRequest {
            lines: vec![CommandLogLine {
                seq: 1,
                line: "stdout line 1".to_string(),
                stream: LogStream::Stdout,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }],
        };
        svc.push_logs(&task_id, log_req).await.unwrap();

        let logs = svc.get_task_logs(&task_id).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].line, "stdout line 1");
    }

    #[tokio::test]
    async fn test_cleanup_old_tasks() {
        let (repo, _sse, svc) = setup_service().await;

        let mut task = CommandTask::new(
            "client-6".to_string(),
            "old".to_string(),
            vec!["command".to_string()],
            "admin".to_string(),
            common::command::DangerLevel::Safe,
            Some(60),
        );
        task.created_at = "2020-01-01T00:00:00Z".to_string();
        repo.save_task(&task).await.unwrap();

        let count = svc.cleanup_old_tasks(0).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_config() {
        let (_repo, _sse, svc) = setup_service().await;

        let config = svc.get_config().await.unwrap();
        assert!(!config.enabled);
    }

    #[tokio::test]
    async fn test_expire_pending() {
        let (repo, _sse, svc) = setup_service().await;

        let mut task = CommandTask::new(
            "client-7".to_string(),
            "stale".to_string(),
            vec!["command".to_string()],
            "admin".to_string(),
            common::command::DangerLevel::Safe,
            Some(60),
        );
        task.expires_at = "2020-01-01T00:00:00Z".to_string();
        repo.save_task(&task).await.unwrap();

        let count = svc.expire_pending_tasks().await.unwrap();
        assert_eq!(count, 1);

        let task = repo.get_task(&task.id).await.unwrap().unwrap();
        assert_eq!(task.status, CommandStatus::Expired);
    }
}
