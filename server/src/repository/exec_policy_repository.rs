use crate::db::Database;
use common::entity::permission::ExecPolicy;
use common::error::{CmdbError, CmdbResult};
use std::sync::Arc;

pub struct ExecPolicyRepository {
    db: Arc<dyn Database>,
}

const PREFIX: &str = "exec_policy:";

impl std::fmt::Debug for ExecPolicyRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecPolicyRepository").finish()
    }
}

impl ExecPolicyRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn save(&self, policy: &ExecPolicy) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, policy.id);
        let value =
            serde_json::to_vec(policy).map_err(|e| CmdbError::Serialization(e.to_string()))?;
        self.db.set(&key, &value).await
    }

    pub async fn get(&self, id: &str) -> CmdbResult<Option<ExecPolicy>> {
        let key = format!("{}{}", PREFIX, id);
        match self.db.get(&key).await {
            Ok(Some(data)) => {
                let policy: ExecPolicy = serde_json::from_slice(&data)
                    .map_err(|e| CmdbError::Serialization(e.to_string()))?;
                Ok(Some(policy))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, id);
        self.db.delete(&key).await
    }

    pub async fn list_all(&self) -> CmdbResult<Vec<ExecPolicy>> {
        let keys = self.db.list_keys(PREFIX).await?;
        let mut policies = Vec::new();
        for key in keys {
            if let Some(data) = self.db.get(&key).await? {
                if let Ok(policy) = serde_json::from_slice::<ExecPolicy>(&data) {
                    policies.push(policy);
                }
            }
        }
        Ok(policies)
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> CmdbResult<usize> {
        let keys = self.db.list_keys(PREFIX).await?;
        Ok(keys.len())
    }
}
