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
        };
        self.repo.save(&approval).await?;

        let audit = AuditLogEntry::new(
            AuditAction::CommandCreate,
            username,
            &format!("Approval request created for command '{}' on client {}", command, target_client_id),
        );
        let _ = self.cmd_repo.append_audit(&audit).await;

        Ok(approval)
    }

    pub async fn approve(
        &self,
        id: &str,
        reviewer: &str,
    ) -> CmdbResult<PendingApproval> {
        let mut approval = self.repo.get(id).await?
            .ok_or_else(|| CmdbError::NotFound(format!("Approval {} not found", id)))?;

        if approval.status != ApprovalStatus::Pending {
            return Err(CmdbError::Validation(format!(
                "Approval {} is not in pending status", id
            )));
        }

        approval.status = ApprovalStatus::Approved;
        approval.reviewed_by = Some(reviewer.to_string());
        approval.reviewed_at = Some(Utc::now().to_rfc3339());
        self.repo.save(&approval).await?;

        let audit = AuditLogEntry::new(
            AuditAction::CommandCreate,
            reviewer,
            &format!("Approved command execution request {}", id),
        );
        let _ = self.cmd_repo.append_audit(&audit).await;

        Ok(approval)
    }

    pub async fn reject(
        &self,
        id: &str,
        reviewer: &str,
    ) -> CmdbResult<PendingApproval> {
        let mut approval = self.repo.get(id).await?
            .ok_or_else(|| CmdbError::NotFound(format!("Approval {} not found", id)))?;

        if approval.status != ApprovalStatus::Pending {
            return Err(CmdbError::Validation(format!(
                "Approval {} is not in pending status", id
            )));
        }

        approval.status = ApprovalStatus::Rejected;
        approval.reviewed_by = Some(reviewer.to_string());
        approval.reviewed_at = Some(Utc::now().to_rfc3339());
        self.repo.save(&approval).await?;

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
        let count = expired.len();
        for mut a in expired {
            a.status = ApprovalStatus::Expired;
            self.repo.save(&a).await?;

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
    ) -> CmdbResult<PendingApproval> {
        let mut approval = self
            .repo
            .get(id)
            .await?
            .ok_or_else(|| CmdbError::NotFound(format!("Approval {} not found", id)))?;
        approval.executed_task_id = Some(task_id.to_string());
        self.repo.save(&approval).await?;
        Ok(approval)
    }
}
