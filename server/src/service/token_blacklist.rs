use crate::db::Database;
use chrono::Utc;
use std::sync::Arc;

const KEY_PREFIX: &str = "revoked_jti:";

pub struct TokenBlacklist {
    db: Arc<dyn Database>,
}

impl TokenBlacklist {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn revoke(&self, jti: &str, exp: usize) {
        let key = format!("{}{}", KEY_PREFIX, jti);
        let value = exp.to_string();
        let _ = self.db.set(&key, value.as_bytes()).await;
    }

    pub async fn is_revoked(&self, jti: &str) -> bool {
        let key = format!("{}{}", KEY_PREFIX, jti);
        self.db.exists(&key).await.unwrap_or(false)
    }

    pub async fn cleanup_expired(&self) -> usize {
        let now = Utc::now().timestamp() as usize;
        let keys = match self.db.list_keys(KEY_PREFIX).await {
            Ok(k) => k,
            Err(_) => return 0,
        };
        let mut removed = 0;
        for key in keys {
            if let Some(value) = self.db.get(&key).await.unwrap_or(None) {
                let exp_str = String::from_utf8_lossy(&value);
                if let Ok(exp) = exp_str.parse::<usize>() {
                    if exp <= now && self.db.delete(&key).await.is_ok() {
                        removed += 1;
                    }
                }
            }
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::setup_test_db;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_revoke_and_check() {
        let db = Arc::new(setup_test_db().unwrap());
        let bl = TokenBlacklist::new(db);
        let exp = (Utc::now().timestamp() + 3600) as usize;
        bl.revoke("test-jti-1", exp).await;
        assert!(bl.is_revoked("test-jti-1").await);
        assert!(!bl.is_revoked("test-jti-2").await);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let db = Arc::new(setup_test_db().unwrap());
        let bl = TokenBlacklist::new(db);
        let past = (Utc::now().timestamp() - 3600) as usize;
        let future = (Utc::now().timestamp() + 3600) as usize;
        bl.revoke("expired-jti", past).await;
        bl.revoke("valid-jti", future).await;
        let removed = bl.cleanup_expired().await;
        assert_eq!(removed, 1);
        assert!(!bl.is_revoked("expired-jti").await);
        assert!(bl.is_revoked("valid-jti").await);
    }
}
