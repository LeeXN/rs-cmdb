use crate::repository::approval_repository::ApprovalRepository;
use crate::repository::command_repository::CommandRepository;
use chrono::Utc;
use common::command::{AuditAction, AuditLogEntry};
use common::entity::permission::{ApprovalPayload, ApprovalStatus, PendingApproval};
use common::error::{CmdbError, CmdbResult};
use std::sync::Arc;
use uuid::Uuid;

pub struct ApprovalService {
    repo: Arc<ApprovalRepository>,
    cmd_repo: Arc<CommandRepository>,
}

impl ApprovalService {
    pub fn new(repo: Arc<ApprovalRepository>, cmd_repo: Arc<CommandRepository>) -> Self {
        Self { repo, cmd_repo }
    }

    pub async fn create_approval(
        &self,
        policy_type: &str,
        policy_id: &str,
        user_id: &str,
        username: &str,
        target_client_id: &str,
        command: &str,
        payload: Option<ApprovalPayload>,
        ttl_secs: i64,
    ) -> CmdbResult<PendingApproval> {
        let now = Utc::now();
        let approval = PendingApproval {
            id: Uuid::new_v4().to_string(),
            policy_type: policy_type.to_string(),
            policy_id: policy_id.to_string(),
            user_id: user_id.to_string(),
            username: username.to_string(),
            target_client_id: target_client_id.to_string(),
            command: command.to_string(),
            payload,
            status: ApprovalStatus::Pending,
            created_at: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::seconds(ttl_secs)).to_rfc3339(),
            reviewed_by: None,
            reviewed_at: None,
            executed_task_id: None,
            execution_started_at: None,
            execution_claim_id: None,
        };
        self.repo.save(&approval).await?;

        let audit = AuditLogEntry::new(
            AuditAction::CommandCreate,
            username,
            &format!(
                "Approval request created for command '{}' on client {}",
                command, target_client_id
            ),
        );
        let _ = self.cmd_repo.append_audit(&audit).await;

        Ok(approval)
    }

    pub async fn approve(&self, id: &str, reviewer: &str) -> CmdbResult<PendingApproval> {
        let now = Utc::now().to_rfc3339();
        let approval = match self
            .repo
            .review_pending(id, reviewer, ApprovalStatus::Approved, &now)
            .await?
        {
            Some(approval) if approval.status == ApprovalStatus::Approved => approval,
            Some(_) => {
                return Err(CmdbError::Validation(format!(
                    "Approval {} has expired",
                    id
                )));
            }
            None => {
                let existing = self
                    .repo
                    .get(id)
                    .await?
                    .ok_or_else(|| CmdbError::NotFound(format!("Approval {} not found", id)))?;
                if existing.status == ApprovalStatus::Approved
                    && existing.executed_task_id.is_none()
                {
                    // Approval is idempotent while task materialization is
                    // pending. This allows a retry after a transient task or
                    // attach failure without creating a second approval.
                    return Ok(existing);
                }
                return Err(CmdbError::Validation(format!(
                    "Approval {} is not in pending status",
                    existing.id
                )));
            }
        };

        let audit = AuditLogEntry::new(
            AuditAction::CommandCreate,
            reviewer,
            &format!("Approved command execution request {}", id),
        );
        let _ = self.cmd_repo.append_audit(&audit).await;

        Ok(approval)
    }

    pub async fn reject(&self, id: &str, reviewer: &str) -> CmdbResult<PendingApproval> {
        let now = Utc::now().to_rfc3339();
        let approval = match self
            .repo
            .review_pending(id, reviewer, ApprovalStatus::Rejected, &now)
            .await?
        {
            Some(approval) if approval.status == ApprovalStatus::Rejected => approval,
            Some(_) => {
                return Err(CmdbError::Validation(format!(
                    "Approval {} has expired",
                    id
                )));
            }
            None => {
                let existing = self
                    .repo
                    .get(id)
                    .await?
                    .ok_or_else(|| CmdbError::NotFound(format!("Approval {} not found", id)))?;
                return Err(CmdbError::Validation(format!(
                    "Approval {} is not in pending status",
                    existing.id
                )));
            }
        };

        let audit = AuditLogEntry::new(
            AuditAction::CommandCreate,
            reviewer,
            &format!("Rejected command execution request {}", id),
        );
        let _ = self.cmd_repo.append_audit(&audit).await;

        Ok(approval)
    }

    pub async fn expire_old(&self) -> CmdbResult<usize> {
        let expired = self.repo.list_expired().await?;
        let now = Utc::now().to_rfc3339();
        let mut count = 0;
        for candidate in expired {
            let Some(a) = self.repo.expire_pending(&candidate.id, &now).await? else {
                continue;
            };
            count += 1;

            let audit = AuditLogEntry::new(
                AuditAction::CommandCreate,
                "system",
                &format!("Approval request {} expired", a.id),
            );
            let _ = self.cmd_repo.append_audit(&audit).await;
        }
        Ok(count)
    }

    pub async fn list_all(&self) -> CmdbResult<Vec<PendingApproval>> {
        self.repo.list_all().await
    }

    pub async fn list_by_user(&self, user_id: &str) -> CmdbResult<Vec<PendingApproval>> {
        self.repo.list_by_user(user_id).await
    }

    #[allow(dead_code)]
    pub async fn list_pending(&self) -> CmdbResult<Vec<PendingApproval>> {
        self.repo.list_pending().await
    }

    pub async fn get(&self, id: &str) -> CmdbResult<Option<PendingApproval>> {
        self.repo.get(id).await
    }

    pub async fn attach_executed_task(
        &self,
        id: &str,
        task_id: &str,
        claim_id: &str,
    ) -> CmdbResult<PendingApproval> {
        self.repo
            .attach_executed_task(id, task_id, claim_id)
            .await?
            .ok_or_else(|| {
                CmdbError::Validation(format!(
                    "Approval {} is not approved or already has an executed task",
                    id
                ))
            })
    }

    /// Atomically claim an approved request for task materialization. A
    /// previous worker that crashed after claiming is allowed to retry after
    /// the bounded claim TTL.
    pub async fn begin_task_materialization(
        &self,
        id: &str,
        now: &str,
        claim_ttl_secs: i64,
    ) -> CmdbResult<PendingApproval> {
        self.repo
            .begin_task_materialization(id, now, claim_ttl_secs)
            .await?
            .ok_or_else(|| {
                CmdbError::Validation(format!(
                    "Approval {} is not ready for task materialization",
                    id
                ))
            })
    }

    /// Roll an approved materialization claim back when task creation fails.
    /// This preserves the approval so an administrator can retry safely.
    pub async fn rollback_task_materialization(&self, id: &str, claim_id: &str) -> CmdbResult<()> {
        self.repo.rollback_task_materialization(id, claim_id).await
    }
}
