use crate::db::Database;
use common::entity::execution::{ExecutionSession, SessionStatus};
use common::error::{CmdbError, CmdbResult};
use common::models::{CommandQuery, PaginatedResult};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ExecutionSessionRepository {
    db: Arc<dyn Database>,
}

const PREFIX: &str = "exec_session:";
const USER_IDX_PREFIX: &str = "exec_session_idx:user:";

impl std::fmt::Debug for ExecutionSessionRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionSessionRepository").finish()
    }
}

impl ExecutionSessionRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    pub async fn save(&self, session: &ExecutionSession) -> CmdbResult<()> {
        let key = format!("{}{}", PREFIX, session.session_id);
        let value =
            serde_json::to_vec(session).map_err(|e| CmdbError::Serialization(e.to_string()))?;
        self.db.set(&key, &value).await?;

        let idx_key = format!(
            "{}{}:{}:{}",
            USER_IDX_PREFIX, session.user_id, session.start_time, session.session_id
        );
        self.db.set(&idx_key, b"1").await?;

        Ok(())
    }

    pub async fn get(&self, session_id: &str) -> CmdbResult<Option<ExecutionSession>> {
        let key = format!("{}{}", PREFIX, session_id);
        match self.db.get(&key).await {
            Ok(Some(data)) => {
                let session: ExecutionSession = serde_json::from_slice(&data)
                    .map_err(|e| CmdbError::Serialization(e.to_string()))?;
                Ok(Some(session))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn delete(&self, session_id: &str) -> CmdbResult<()> {
        let Some(session) = self.get(session_id).await? else {
            return Ok(());
        };
        self.db.delete(&format!("{}{}", PREFIX, session_id)).await?;
        let index_key = format!(
            "{}{}:{}:{}",
            USER_IDX_PREFIX, session.user_id, session.start_time, session.session_id
        );
        self.db.delete(&index_key).await
    }

    pub async fn list_by_user(
        &self,
        user_id: &str,
        query: &CommandQuery,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        self.list_by_user_filtered(user_id, query, None).await
    }

    pub async fn list_by_user_scoped(
        &self,
        user_id: &str,
        query: &CommandQuery,
        allowed_client_ids: &HashSet<String>,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        self.list_by_user_filtered(user_id, query, Some(allowed_client_ids))
            .await
    }

    async fn list_by_user_filtered(
        &self,
        user_id: &str,
        query: &CommandQuery,
        allowed_client_ids: Option<&HashSet<String>>,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        let prefix = format!("{}{}:", USER_IDX_PREFIX, user_id);
        let idx_keys = self.db.list_keys(&prefix).await?;
        let mut idx_keys = idx_keys;
        idx_keys.sort_by(|a, b| b.cmp(a));

        let mut sessions = Vec::new();
        for idx_key in &idx_keys {
            let parts: Vec<&str> = idx_key.split(':').collect();
            if let Some(session_id) = parts.last() {
                if let Ok(Some(session)) = self.get(session_id).await {
                    let visible = allowed_client_ids
                        .map(|allowed| {
                            session
                                .client_ids
                                .iter()
                                .all(|client_id| allowed.contains(client_id))
                        })
                        .unwrap_or(true);
                    if visible {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(self.filter_and_paginate(sessions, query))
    }

    pub async fn list_all(
        &self,
        query: &CommandQuery,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        let keys = self.db.list_keys(PREFIX).await?;
        let mut keys = keys;
        keys.sort_by(|a, b| b.cmp(a));

        let mut sessions = Vec::new();
        for key in keys {
            if let Some(data) = self.db.get(&key).await? {
                if let Ok(session) = serde_json::from_slice::<ExecutionSession>(&data) {
                    sessions.push(session);
                }
            }
        }

        Ok(self.filter_and_paginate(sessions, query))
    }

    pub async fn update_status(
        &self,
        session_id: &str,
        status: SessionStatus,
        end_time: &str,
        duration_secs: u64,
    ) -> CmdbResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.status = status;
            session.end_time = Some(end_time.to_string());
            session.duration_secs = Some(duration_secs);
            self.save(&session).await?;
        }
        Ok(())
    }

    pub async fn update_cast_path(&self, session_id: &str, cast_file_path: &str) -> CmdbResult<()> {
        if let Some(mut session) = self.get(session_id).await? {
            session.cast_file_path = Some(cast_file_path.to_string());
            self.save(&session).await?;
        }
        Ok(())
    }

    fn filter_and_paginate(
        &self,
        mut sessions: Vec<ExecutionSession>,
        query: &CommandQuery,
    ) -> PaginatedResult<ExecutionSession> {
        if let Some(client_id) = query.client_id.as_deref().filter(|value| !value.is_empty()) {
            sessions.retain(|session| session.client_ids.iter().any(|id| id == client_id));
        }

        if let Some(user_id) = query.user_id.as_deref().filter(|value| !value.is_empty()) {
            sessions.retain(|session| session.user_id == user_id);
        }

        if let Some(status) = query.status.as_deref().filter(|value| !value.is_empty()) {
            sessions.retain(|session| session.status.to_string().eq_ignore_ascii_case(status));
        }

        if let Some(execution_type) = query
            .execution_type
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            sessions.retain(|session| {
                session
                    .execution_type
                    .to_string()
                    .eq_ignore_ascii_case(execution_type)
            });
        }

        if let Some(from) = query.from.as_deref().filter(|value| !value.is_empty()) {
            sessions.retain(|session| session.start_time.as_str() >= from);
        }

        if let Some(to) = query.to.as_deref().filter(|value| !value.is_empty()) {
            sessions.retain(|session| session.start_time.as_str() <= to);
        }

        if let Some(search) = query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let needle = search.to_lowercase();
            sessions.retain(|session| {
                session.session_id.to_lowercase().contains(&needle)
                    || session.username.to_lowercase().contains(&needle)
                    || session.command.to_lowercase().contains(&needle)
                    || session
                        .client_ids
                        .iter()
                        .any(|id| id.to_lowercase().contains(&needle))
            });
        }

        sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let total = sessions.len();
        let total_pages = total.div_ceil(page_size);
        let start = (page - 1) * page_size;
        let items = sessions.into_iter().skip(start).take(page_size).collect();

        PaginatedResult {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use common::entity::execution::ExecutionType;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockDatabase {
        data: Mutex<HashMap<String, Vec<u8>>>,
    }

    #[async_trait]
    impl Database for MockDatabase {
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

    fn repo() -> ExecutionSessionRepository {
        ExecutionSessionRepository::new(Arc::new(MockDatabase::default()))
    }

    fn session(
        session_id: &str,
        user_id: &str,
        client_ids: &[&str],
        execution_type: ExecutionType,
        status: SessionStatus,
        start_time: &str,
    ) -> ExecutionSession {
        ExecutionSession {
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            username: format!("{}-name", user_id),
            client_ids: client_ids.iter().map(|id| id.to_string()).collect(),
            execution_type,
            command: format!("echo {}", session_id),
            status,
            start_time: start_time.to_string(),
            end_time: None,
            duration_secs: None,
            cast_file_path: None,
        }
    }

    #[tokio::test]
    async fn list_all_filters_by_client_status_type_and_date() {
        let repo = repo();
        repo.save(&session(
            "s1",
            "user-a",
            &["client-1"],
            ExecutionType::Terminal,
            SessionStatus::Success,
            "2026-07-08T10:00:00Z",
        ))
        .await
        .unwrap();
        repo.save(&session(
            "s2",
            "user-a",
            &["client-2"],
            ExecutionType::Batch,
            SessionStatus::Failed,
            "2026-07-07T10:00:00Z",
        ))
        .await
        .unwrap();
        repo.save(&session(
            "s3",
            "user-b",
            &["client-1", "client-3"],
            ExecutionType::Batch,
            SessionStatus::Success,
            "2026-07-08T12:00:00Z",
        ))
        .await
        .unwrap();

        let result = repo
            .list_all(&CommandQuery {
                client_id: Some("client-1".to_string()),
                status: Some("success".to_string()),
                execution_type: Some("batch".to_string()),
                from: Some("2026-07-08T00:00:00Z".to_string()),
                to: Some("2026-07-08T23:59:59Z".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].session_id, "s3");
    }

    #[tokio::test]
    async fn list_by_user_paginates_and_sorts_descending() {
        let repo = repo();
        repo.save(&session(
            "s1",
            "user-a",
            &["client-1"],
            ExecutionType::Terminal,
            SessionStatus::Pending,
            "2026-07-08T09:00:00Z",
        ))
        .await
        .unwrap();
        repo.save(&session(
            "s2",
            "user-a",
            &["client-1"],
            ExecutionType::Terminal,
            SessionStatus::Running,
            "2026-07-08T11:00:00Z",
        ))
        .await
        .unwrap();
        repo.save(&session(
            "s3",
            "user-a",
            &["client-2"],
            ExecutionType::Batch,
            SessionStatus::Success,
            "2026-07-08T13:00:00Z",
        ))
        .await
        .unwrap();

        let result = repo
            .list_by_user(
                "user-a",
                &CommandQuery {
                    page: Some(2),
                    page_size: Some(1),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.total_pages, 3);
        assert_eq!(result.page, 2);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].session_id, "s2");
    }

    #[tokio::test]
    async fn list_all_search_matches_command_username_and_session_id() {
        let repo = repo();

        let mut deploy = session(
            "deploy-1",
            "alice",
            &["client-1"],
            ExecutionType::Terminal,
            SessionStatus::Success,
            "2026-07-08T09:00:00Z",
        );
        deploy.username = "alice.ops".into();
        deploy.command = "deploy service-a".into();
        repo.save(&deploy).await.unwrap();

        let mut inspect = session(
            "inspect-2",
            "bob",
            &["client-2"],
            ExecutionType::Batch,
            SessionStatus::Running,
            "2026-07-08T10:00:00Z",
        );
        inspect.username = "bob.viewer".into();
        inspect.command = "journalctl -u nginx".into();
        repo.save(&inspect).await.unwrap();

        let by_command = repo
            .list_all(&CommandQuery {
                search: Some("journalctl".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(by_command.total, 1);
        assert_eq!(by_command.items[0].session_id, "inspect-2");

        let by_user = repo
            .list_all(&CommandQuery {
                search: Some("alice.ops".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(by_user.total, 1);
        assert_eq!(by_user.items[0].session_id, "deploy-1");

        let by_session = repo
            .list_all(&CommandQuery {
                search: Some("inspect-2".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(by_session.total, 1);
        assert_eq!(by_session.items[0].command, "journalctl -u nginx");
    }
}
