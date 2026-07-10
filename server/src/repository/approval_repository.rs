use crate::db::Database;
use common::entity::permission::PendingApproval;
use common::error::{CmdbError, CmdbResult};
use std::sync::Arc;

pub struct ApprovalRepository {
    db: Arc<dyn Database>,
}

const PREFIX: &str = "pending_approval:";

impl std::fmt::Debug for ApprovalRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApprovalRepository").finish()
    }
}

impl ApprovalRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn save(&self, approval: &PendingApproval) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, approval.id);
        let value = serde_json::to_vec(approval)
            .map_err(|e| CmdbError::Serialization(e.to_string()))?;
        self.db.set(&key, &value).await
    }

    pub async fn get(&self, id: &str) -> CmdbResult<Option<PendingApproval>> {
        let key = format!("{}{}", PREFIX, id);
        match self.db.get(&key).await {
            Ok(Some(data)) => {
                let approval: PendingApproval = serde_json::from_slice(&data)
                    .map_err(|e| CmdbError::Serialization(e.to_string()))?;
                Ok(Some(approval))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, id);
        self.db.delete(&key).await
    }

    pub async fn list_all(&self) -> CmdbResult<Vec<PendingApproval>> {
        let keys = self.db.list_keys(PREFIX).await?;
        let mut approvals = Vec::new();
        for key in keys {
            if let Some(data) = self.db.get(&key).await? {
                if let Ok(approval) = serde_json::from_slice::<PendingApproval>(&data) {
                    approvals.push(approval);
                }
            }
        }
        Ok(approvals)
    }

    pub async fn list_by_user(&self, user_id: &str) -> CmdbResult<Vec<PendingApproval>> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|a| a.user_id == user_id).collect())
    }

    #[allow(dead_code)]
    pub async fn list_pending(&self) -> CmdbResult<Vec<PendingApproval>> {
        let all = self.list_all().await?;
        Ok(all.into_iter().filter(|a| matches!(a.status, common::entity::permission::ApprovalStatus::Pending)).collect())
    }

    pub async fn list_expired(&self) -> CmdbResult<Vec<PendingApproval>> {
        let all = self.list_all().await?;
        let now = chrono::Utc::now().to_rfc3339();
        Ok(all.into_iter().filter(|a| {
            matches!(a.status, common::entity::permission::ApprovalStatus::Pending) && a.expires_at <= now
        }).collect())
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> CmdbResult<usize> {
        let keys = self.db.list_keys(PREFIX).await?;
        Ok(keys.len())
    }
}
