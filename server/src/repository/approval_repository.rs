use crate::db::Database;
use common::entity::permission::{ApprovalStatus, PendingApproval};
use common::error::{CmdbError, CmdbResult};
use std::sync::Arc;
use uuid::Uuid;

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
        let value =
            serde_json::to_vec(approval).map_err(|e| CmdbError::Serialization(e.to_string()))?;
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

    /// Atomically review a pending approval. Expiry is checked in the same
    /// write transaction as the status transition.
    pub async fn review_pending(
        &self,
        id: &str,
        reviewer: &str,
        requested_status: ApprovalStatus,
        now: &str,
    ) -> CmdbResult<Option<PendingApproval>> {
        let key = format!("{}{}", PREFIX, id);
        let expected_key = key.clone();
        let result = Arc::new(std::sync::Mutex::new(None::<PendingApproval>));
        let result_slot = result.clone();
        let id = id.to_string();
        let reviewer = reviewer.to_string();
        let now = now.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut approval) = serde_json::from_slice::<PendingApproval>(&value) else {
                        return None;
                    };
                    if approval.id != id || approval.status != ApprovalStatus::Pending {
                        return None;
                    }
                    let expired = chrono::DateTime::parse_from_rfc3339(&approval.expires_at)
                        .ok()
                        .zip(chrono::DateTime::parse_from_rfc3339(&now).ok())
                        .is_some_and(|(expires, now)| expires <= now);
                    approval.status = if expired {
                        ApprovalStatus::Expired
                    } else {
                        requested_status.clone()
                    };
                    approval.reviewed_by = Some(reviewer.clone());
                    approval.reviewed_at = Some(now.clone());
                    let Ok(bytes) = serde_json::to_vec(&approval) else {
                        return None;
                    };
                    *result_slot.lock().expect("approval review lock") = Some(approval);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(result.lock().expect("approval review lock").clone())
    }

    /// Atomically mark an expired pending approval as Expired. A scheduler
    /// scan may be stale, so the status check must happen in the same write
    /// transaction as the transition.
    pub async fn expire_pending(&self, id: &str, now: &str) -> CmdbResult<Option<PendingApproval>> {
        let key = format!("{}{}", PREFIX, id);
        let expected_key = key.clone();
        let result = Arc::new(std::sync::Mutex::new(None::<PendingApproval>));
        let result_slot = result.clone();
        let id = id.to_string();
        let now = now.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut approval) = serde_json::from_slice::<PendingApproval>(&value) else {
                        return None;
                    };
                    let expired = chrono::DateTime::parse_from_rfc3339(&approval.expires_at)
                        .ok()
                        .zip(chrono::DateTime::parse_from_rfc3339(&now).ok())
                        .is_some_and(|(expires, now)| expires <= now);
                    if approval.id != id || approval.status != ApprovalStatus::Pending || !expired {
                        return None;
                    }
                    approval.status = ApprovalStatus::Expired;
                    let Ok(bytes) = serde_json::to_vec(&approval) else {
                        return None;
                    };
                    *result_slot.lock().expect("approval expiry lock") = Some(approval);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(result.lock().expect("approval expiry lock").clone())
    }

    /// Attach exactly one executed task to an approved approval request.
    pub async fn attach_executed_task(
        &self,
        id: &str,
        task_id: &str,
        claim_id: &str,
    ) -> CmdbResult<Option<PendingApproval>> {
        let key = format!("{}{}", PREFIX, id);
        let expected_key = key.clone();
        let result = Arc::new(std::sync::Mutex::new(None::<PendingApproval>));
        let result_slot = result.clone();
        let id = id.to_string();
        let task_id = task_id.to_string();
        let claim_id = claim_id.to_string();
        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut approval) = serde_json::from_slice::<PendingApproval>(&value) else {
                        return None;
                    };
                    if approval.id != id
                        || approval.status != ApprovalStatus::Approved
                        || approval.executed_task_id.is_some()
                        || approval.execution_claim_id.as_deref() != Some(claim_id.as_str())
                    {
                        return None;
                    }
                    approval.executed_task_id = Some(task_id.clone());
                    approval.execution_started_at = None;
                    approval.execution_claim_id = None;
                    let Ok(bytes) = serde_json::to_vec(&approval) else {
                        return None;
                    };
                    *result_slot.lock().expect("approval task lock") = Some(approval);
                    Some(bytes)
                }),
            )
            .await?;
        Ok(result.lock().expect("approval task lock").clone())
    }

    pub async fn begin_task_materialization(
        &self,
        id: &str,
        now: &str,
        claim_ttl_secs: i64,
    ) -> CmdbResult<Option<PendingApproval>> {
        let key = format!("{}{}", PREFIX, id);
        let expected_key = key.clone();
        let result = Arc::new(std::sync::Mutex::new(None::<PendingApproval>));
        let result_slot = result.clone();
        let id = id.to_string();
        let now = now.to_string();
        let claim_id = Uuid::new_v4().to_string();
        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut approval) = serde_json::from_slice::<PendingApproval>(&value) else {
                        return None;
                    };
                    if approval.id != id
                        || approval.status != ApprovalStatus::Approved
                        || approval.executed_task_id.is_some()
                    {
                        return None;
                    }
                    if let Some(started_at) = approval.execution_started_at.as_deref()
                        && chrono::DateTime::parse_from_rfc3339(started_at)
                            .ok()
                            .zip(chrono::DateTime::parse_from_rfc3339(&now).ok())
                            .is_some_and(|(started, current)| {
                                started + chrono::Duration::seconds(claim_ttl_secs) > current
                            })
                    {
                        return None;
                    }
                    approval.execution_started_at = Some(now.clone());
                    approval.execution_claim_id = Some(claim_id.clone());
                    let Ok(bytes) = serde_json::to_vec(&approval) else {
                        return None;
                    };
                    *result_slot.lock().expect("approval materialization lock") = Some(approval);
                    Some(bytes)
                }),
            )
            .await?;
        Ok(result
            .lock()
            .expect("approval materialization lock")
            .clone())
    }

    pub async fn rollback_task_materialization(&self, id: &str, claim_id: &str) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, id);
        let expected_key = key.clone();
        let claim_id = claim_id.to_string();
        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut approval) = serde_json::from_slice::<PendingApproval>(&value) else {
                        return None;
                    };
                    if approval.status != ApprovalStatus::Approved
                        || approval.executed_task_id.is_some()
                        || approval.execution_claim_id.as_deref() != Some(claim_id.as_str())
                    {
                        return None;
                    }
                    approval.execution_started_at = None;
                    approval.execution_claim_id = None;
                    serde_json::to_vec(&approval).ok()
                }),
            )
            .await
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
        Ok(all
            .into_iter()
            .filter(|a| {
                matches!(
                    a.status,
                    common::entity::permission::ApprovalStatus::Pending
                )
            })
            .collect())
    }

    pub async fn list_expired(&self) -> CmdbResult<Vec<PendingApproval>> {
        let all = self.list_all().await?;
        let now = chrono::Utc::now();
        Ok(all
            .into_iter()
            .filter(|a| {
                matches!(
                    a.status,
                    common::entity::permission::ApprovalStatus::Pending
                ) && chrono::DateTime::parse_from_rfc3339(&a.expires_at)
                    .map(|expires| expires.timestamp_micros() <= now.timestamp_micros())
                    .unwrap_or(false)
            })
            .collect())
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> CmdbResult<usize> {
        let keys = self.db.list_keys(PREFIX).await?;
        Ok(keys.len())
    }
}
