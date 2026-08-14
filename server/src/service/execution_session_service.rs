use crate::repository::client_repository::ClientRepository;
use crate::repository::execution_session_repository::ExecutionSessionRepository;
use chrono::Utc;
use common::entity::execution::{ExecutionSession, ExecutionType, SessionStatus};
use common::error::CmdbResult;
use common::models::Client;
use common::models::{CommandQuery, PaginatedResult};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ExecutionSessionService {
    repo: Arc<ExecutionSessionRepository>,
    client_repo: Arc<ClientRepository>,
}

impl ExecutionSessionService {
    pub fn new(repo: Arc<ExecutionSessionRepository>, client_repo: Arc<ClientRepository>) -> Self {
        Self { repo, client_repo }
    }

    pub async fn save_session(&self, session: &ExecutionSession) -> CmdbResult<()> {
        self.repo.save(session).await
    }

    #[allow(dead_code)]
    pub async fn create_session(
        &self,
        user_id: &str,
        username: &str,
        client_ids: Vec<String>,
        command: &str,
        execution_type: ExecutionType,
    ) -> CmdbResult<ExecutionSession> {
        let session = ExecutionSession::new(user_id, username, client_ids, command, execution_type);
        self.repo.save(&session).await?;
        Ok(session)
    }

    pub async fn complete_session(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> CmdbResult<()> {
        let now = Utc::now().to_rfc3339();
        if let Some(session) = self.repo.get(session_id).await? {
            let start =
                chrono::DateTime::parse_from_rfc3339(&session.start_time).unwrap_or_default();
            let end = chrono::DateTime::parse_from_rfc3339(&now).unwrap_or_default();
            let duration = (end - start).num_seconds().max(0) as u64;
            self.repo
                .update_status(session_id, status, &now, duration)
                .await?;
        }
        Ok(())
    }

    pub async fn list_sessions(
        &self,
        query: &CommandQuery,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        let query = self.resolve_client_filter(query).await?;
        self.repo.list_all(&query).await
    }

    pub async fn list_user_sessions(
        &self,
        user_id: &str,
        query: &CommandQuery,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        let query = self.resolve_client_filter(query).await?;
        self.repo.list_by_user(user_id, &query).await
    }

    pub async fn list_user_sessions_for_client_ids(
        &self,
        user_id: &str,
        query: &CommandQuery,
        allowed_client_ids: &HashSet<String>,
    ) -> CmdbResult<PaginatedResult<ExecutionSession>> {
        let query = self.resolve_client_filter(query).await?;
        self.repo
            .list_by_user_scoped(user_id, &query, allowed_client_ids)
            .await
    }

    pub async fn get_session(&self, session_id: &str) -> CmdbResult<Option<ExecutionSession>> {
        self.repo.get(session_id).await
    }

    pub async fn delete_session(&self, session_id: &str) -> CmdbResult<()> {
        self.repo.delete(session_id).await
    }

    pub async fn update_cast_path(&self, session_id: &str, cast_path: &str) -> CmdbResult<()> {
        self.repo.update_cast_path(session_id, cast_path).await
    }

    async fn resolve_client_filter(&self, query: &CommandQuery) -> CmdbResult<CommandQuery> {
        let Some(client_id) = query
            .client_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(query.clone());
        };

        let clients = self.client_repo.list_all().await?;
        if clients.iter().any(|client| client.id == client_id) {
            return Ok(query.clone());
        }

        let matched_ids = matching_client_ids(&clients, client_id);
        let mut next = query.clone();
        next.client_id = matched_ids
            .first()
            .cloned()
            .or_else(|| Some("__no_match__".to_string()));
        Ok(next)
    }
}

fn matching_client_ids(clients: &[Client], search: &str) -> Vec<String> {
    let needle = search.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }

    clients
        .iter()
        .filter(|client| client_matches(client, &needle))
        .map(|client| client.id.clone())
        .collect()
}

fn client_matches(client: &Client, needle: &str) -> bool {
    let primary_ip = client.primary_ip.as_deref().unwrap_or("");
    let serial_number = client.serial_number.as_deref().unwrap_or("");

    client.id.to_lowercase().contains(needle)
        || client.hostname.to_lowercase().contains(needle)
        || client.ip_address.to_lowercase().contains(needle)
        || primary_ip.to_lowercase().contains(needle)
        || serial_number.to_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use super::{client_matches, matching_client_ids};
    use common::models::Client;

    fn client(
        id: &str,
        hostname: &str,
        ip: &str,
        primary_ip: Option<&str>,
        serial: Option<&str>,
    ) -> Client {
        Client {
            id: id.to_string(),
            hostname: hostname.to_string(),
            ip_address: ip.to_string(),
            primary_ip: primary_ip.map(|value| value.to_string()),
            serial_number: serial.map(|value| value.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn client_matches_checks_all_supported_identifier_fields() {
        let client = client(
            "client-77",
            "db-prod-01",
            "10.1.0.8",
            Some("172.16.8.9"),
            Some("SN-7788"),
        );

        assert!(client_matches(&client, "client-77"));
        assert!(client_matches(&client, "db-prod"));
        assert!(client_matches(&client, "10.1.0"));
        assert!(client_matches(&client, "172.16.8"));
        assert!(client_matches(&client, "sn-7788"));
        assert!(!client_matches(&client, "no-match"));
    }

    #[test]
    fn matching_client_ids_returns_all_matches_in_input_order() {
        let clients = vec![
            client("client-1", "web-a", "10.0.0.1", None, Some("SN-1")),
            client(
                "client-2",
                "web-b",
                "10.0.0.2",
                Some("172.16.0.2"),
                Some("SN-2"),
            ),
            client("client-3", "db-a", "10.0.0.3", None, Some("SN-3")),
        ];

        assert_eq!(
            matching_client_ids(&clients, "web"),
            vec!["client-1".to_string(), "client-2".to_string()]
        );
        assert_eq!(
            matching_client_ids(&clients, "SN-3"),
            vec!["client-3".to_string()]
        );
        assert!(matching_client_ids(&clients, "   ").is_empty());
    }
}
