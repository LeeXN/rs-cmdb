use crate::db::Database;
use common::entity::permission::WebTerminalPolicy;
use common::error::{CmdbError, CmdbResult};
use std::sync::Arc;

pub struct WebTerminalPolicyRepository {
    db: Arc<dyn Database>,
}

const PREFIX: &str = "web_terminal_policy:";

impl std::fmt::Debug for WebTerminalPolicyRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebTerminalPolicyRepository").finish()
    }
}

impl WebTerminalPolicyRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn save(&self, policy: &WebTerminalPolicy) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, policy.id);
        let value = serde_json::to_vec(policy)
            .map_err(|e| CmdbError::Serialization(e.to_string()))?;
        self.db.set(&key, &value).await
    }

    pub async fn get(&self, id: &str) -> CmdbResult<Option<WebTerminalPolicy>> {
        let key = format!("{}{}", PREFIX, id);
        match self.db.get(&key).await {
            Ok(Some(data)) => {
                let policy: WebTerminalPolicy = serde_json::from_slice(&data)
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

    pub async fn list_all(&self) -> CmdbResult<Vec<WebTerminalPolicy>> {
        let keys = self.db.list_keys(PREFIX).await?;
        let mut policies = Vec::new();
        for key in keys {
            if let Some(data) = self.db.get(&key).await? {
                if let Ok(policy) = serde_json::from_slice::<WebTerminalPolicy>(&data) {
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
