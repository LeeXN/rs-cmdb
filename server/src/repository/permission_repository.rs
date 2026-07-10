use crate::db::Database;
use common::entity::permission::{Group, PermissionRule};
use common::error::{CmdbError, CmdbResult};
use serde_json;
use std::sync::Arc;

pub struct PermissionRepository {
    db: Arc<dyn Database>,
    rule_prefix: String,
    group_prefix: String,
}

impl std::fmt::Debug for PermissionRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionRepository")
            .field("rule_prefix", &self.rule_prefix)
            .field("group_prefix", &self.group_prefix)
            .finish()
    }
}

impl PermissionRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self {
            db,
            rule_prefix: "perm:rule:".to_string(),
            group_prefix: "perm:group:".to_string(),
        }
    }

    // ── PermissionRule CRUD ──────────────────────────────────────────────────

    fn rule_key(&self, id: &str) -> String {
        format!("{}{}", self.rule_prefix, id)
    }

    pub async fn save_rule(&self, rule: &PermissionRule) -> CmdbResult<()> {
        let json = serde_json::to_vec(rule)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize rule: {}", e)))?;
        self.db.set(&self.rule_key(&rule.id), &json).await
    }

    pub async fn get_rule(&self, id: &str) -> CmdbResult<Option<PermissionRule>> {
        let data = match self.db.get(&self.rule_key(id)).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        let rule = serde_json::from_slice(&data)
            .map_err(|e| CmdbError::Serialization(format!("Failed to deserialize rule: {}", e)))?;
        Ok(Some(rule))
    }

    pub async fn delete_rule(&self, id: &str) -> CmdbResult<()> {
        self.db.delete(&self.rule_key(id)).await
    }

    pub async fn list_rules(&self) -> CmdbResult<Vec<PermissionRule>> {
        let values = self.db.list_values(&self.rule_prefix).await?;
        let mut rules = Vec::with_capacity(values.len());
        for data in values {
            match serde_json::from_slice::<PermissionRule>(&data) {
                Ok(rule) => rules.push(rule),
                Err(e) => {
                    return Err(CmdbError::Serialization(format!(
                        "Failed to deserialize rule: {}",
                        e
                    )));
                }
            }
        }
        Ok(rules)
    }

    // ── Group CRUD ───────────────────────────────────────────────────────────

    fn group_key(&self, id: &str) -> String {
        format!("{}{}", self.group_prefix, id)
    }

    pub async fn save_group(&self, group: &Group) -> CmdbResult<()> {
        let json = serde_json::to_vec(group)
            .map_err(|e| CmdbError::Serialization(format!("Failed to serialize group: {}", e)))?;
        self.db.set(&self.group_key(&group.id), &json).await
    }

    pub async fn get_group(&self, id: &str) -> CmdbResult<Option<Group>> {
        let data = match self.db.get(&self.group_key(id)).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        let group = serde_json::from_slice(&data).map_err(|e| {
            CmdbError::Serialization(format!("Failed to deserialize group: {}", e))
        })?;
        Ok(Some(group))
    }

    pub async fn delete_group(&self, id: &str) -> CmdbResult<()> {
        self.db.delete(&self.group_key(id)).await
    }

    pub async fn list_groups(&self) -> CmdbResult<Vec<Group>> {
        let values = self.db.list_values(&self.group_prefix).await?;
        let mut groups = Vec::with_capacity(values.len());
        for data in values {
            match serde_json::from_slice::<Group>(&data) {
                Ok(group) => groups.push(group),
                Err(e) => {
                    return Err(CmdbError::Serialization(format!(
                        "Failed to deserialize group: {}",
                        e
                    )));
                }
            }
        }
        Ok(groups)
    }

    // ── Count operations ────────────────────────────────────────────────────

    pub async fn rule_count(&self) -> CmdbResult<usize> {
        let keys = self.db.list_keys(&self.rule_prefix).await?;
        Ok(keys.len())
    }

    pub async fn group_count(&self) -> CmdbResult<usize> {
        let keys = self.db.list_keys(&self.group_prefix).await?;
        Ok(keys.len())
    }
}
