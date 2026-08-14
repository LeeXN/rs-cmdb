use crate::db::Database;
use chrono::DateTime;
use common::error::{CmdbError, CmdbResult};
use common::models::{
    TerminalInputChunk, TerminalOutputChunk, TerminalResizeInstruction, TerminalSessionSummary,
};
use std::sync::Arc;
use uuid::Uuid;

pub struct TerminalSessionRepository {
    db: Arc<dyn Database>,
}

impl TerminalSessionRepository {
    pub fn new(db: Arc<dyn Database>) -> Self {
        Self { db }
    }

    fn session_key(session_id: &str) -> String {
        format!("term_session:{}", session_id)
    }

    fn client_index_key(client_id: &str, created_at: &str, session_id: &str) -> String {
        format!(
            "term_session_idx:client:{}:{}:{}",
            client_id, created_at, session_id
        )
    }

    fn output_key(session_id: &str, seq: u64, unique_id: &str) -> String {
        // Agent-side sequence numbers restart when a PTY is recreated. Keep a
        // unique suffix so a reconnect cannot overwrite earlier output with
        // the same sequence number.
        format!("term_output:{}:{:016}:{}", session_id, seq, unique_id)
    }

    fn output_prefix(session_id: &str) -> String {
        format!("term_output:{}", session_id)
    }

    fn input_key(session_id: &str, seq: u64) -> String {
        format!("term_input:{}:{:016}", session_id, seq)
    }

    fn input_prefix(session_id: &str) -> String {
        format!("term_input:{}", session_id)
    }

    fn resize_key(session_id: &str) -> String {
        format!("term_resize:{}", session_id)
    }

    fn close_key(session_id: &str) -> String {
        format!("term_close:{}", session_id)
    }

    fn attach_key(session_id: &str) -> String {
        format!("term_attach:{}", session_id)
    }

    pub async fn save_session(&self, session: &TerminalSessionSummary) -> CmdbResult<()> {
        let bytes = serde_json::to_vec(session)
            .map_err(|e| CmdbError::Serialization(format!("serialize terminal session: {}", e)))?;
        self.db
            .set(&Self::session_key(&session.session_id), &bytes)
            .await?;
        self.db
            .set(
                &Self::client_index_key(
                    &session.client_id,
                    &session.created_at,
                    &session.session_id,
                ),
                b"1",
            )
            .await?;
        Ok(())
    }

    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        match self.db.get(&Self::session_key(session_id)).await? {
            Some(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|e| {
                CmdbError::Serialization(format!("deserialize terminal session: {}", e))
            }),
            None => Ok(None),
        }
    }

    /// Atomically acquire or renew a terminal session lease for an agent.
    /// The caller may have selected a stale candidate, so the ownership check
    /// must happen inside the database write transaction rather than in the
    /// service's read/modify/save sequence.
    pub async fn claim_session(
        &self,
        session_id: &str,
        client_id: &str,
        claim_id: &str,
        now: &str,
        lease_expires_at: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let key = Self::session_key(session_id);
        let expected_key = key.clone();
        let claimed = Arc::new(std::sync::Mutex::new(None::<TerminalSessionSummary>));
        let claimed_result = claimed.clone();
        let session_id = session_id.to_string();
        let client_id = client_id.to_string();
        let claim_id = claim_id.to_string();
        let now = now.to_string();
        let lease_expires_at = lease_expires_at.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut session) = serde_json::from_slice::<TerminalSessionSummary>(&value)
                    else {
                        return None;
                    };
                    let now_dt = DateTime::parse_from_rfc3339(&now).ok();
                    let lease_expired = session
                        .lease_expires_at
                        .as_deref()
                        .and_then(|deadline| DateTime::parse_from_rfc3339(deadline).ok())
                        .zip(now_dt.as_ref())
                        .is_some_and(|(deadline, now)| {
                            deadline.timestamp_micros() <= now.timestamp_micros()
                        });
                    let available = session.session_id == session_id
                        && session.client_id == client_id
                        && matches!(
                            session.state,
                            common::models::TerminalSessionState::Pending
                                | common::models::TerminalSessionState::Active
                        )
                        && (session.lease_id.is_none()
                            || session.lease_id.as_deref() == Some(claim_id.as_str())
                            || lease_expired);
                    if !available {
                        return None;
                    }

                    session.lease_id = Some(claim_id.clone());
                    session.last_heartbeat_at = Some(now.clone());
                    session.lease_expires_at = Some(lease_expires_at.clone());
                    let Ok(bytes) = serde_json::to_vec(&session) else {
                        return None;
                    };
                    *claimed_result.lock().expect("terminal claim lock") = Some(session);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(claimed.lock().expect("terminal claim lock").clone())
    }

    /// Refresh a lease only while the session is still Active and owned by the
    /// same claim. Keeping the state/claim check inside the write transaction
    /// prevents a late heartbeat from overwriting a concurrent close or a
    /// newer agent claim.
    pub async fn heartbeat_session(
        &self,
        session_id: &str,
        client_id: &str,
        claim_id: &str,
        now: &str,
        lease_expires_at: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let key = Self::session_key(session_id);
        let expected_key = key.clone();
        let refreshed = Arc::new(std::sync::Mutex::new(None::<TerminalSessionSummary>));
        let refreshed_result = refreshed.clone();
        let session_id = session_id.to_string();
        let client_id = client_id.to_string();
        let claim_id = claim_id.to_string();
        let now = now.to_string();
        let lease_expires_at = lease_expires_at.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut session) = serde_json::from_slice::<TerminalSessionSummary>(&value)
                    else {
                        return None;
                    };
                    if session.session_id != session_id
                        || session.client_id != client_id
                        || session.state != common::models::TerminalSessionState::Active
                        || session.lease_id.as_deref() != Some(claim_id.as_str())
                    {
                        return None;
                    }

                    session.last_heartbeat_at = Some(now.clone());
                    session.lease_expires_at = Some(lease_expires_at.clone());
                    let Ok(bytes) = serde_json::to_vec(&session) else {
                        return None;
                    };
                    *refreshed_result.lock().expect("terminal heartbeat lock") = Some(session);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(refreshed.lock().expect("terminal heartbeat lock").clone())
    }

    /// Append output only while the authenticated agent still owns a live
    /// lease. The session metadata is updated with the same conditional
    /// write, so a late output packet cannot revive a closed session or
    /// overwrite a newer agent claim.
    pub async fn append_output_if_claimed(
        &self,
        session_id: &str,
        client_id: &str,
        claim_id: &str,
        chunks: &[TerminalOutputChunk],
        now: &str,
        lease_expires_at: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let key = Self::session_key(session_id);
        let expected_key = key.clone();
        let updated = Arc::new(std::sync::Mutex::new(None::<TerminalSessionSummary>));
        let updated_result = updated.clone();
        let session_id = session_id.to_string();
        let output_session_id = session_id.clone();
        let client_id = client_id.to_string();
        let claim_id = claim_id.to_string();
        let now = now.to_string();
        let lease_expires_at = lease_expires_at.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut session) = serde_json::from_slice::<TerminalSessionSummary>(&value)
                    else {
                        return None;
                    };
                    let now_dt = DateTime::parse_from_rfc3339(&now).ok();
                    let lease_valid = session
                        .lease_expires_at
                        .as_deref()
                        .and_then(|deadline| DateTime::parse_from_rfc3339(deadline).ok())
                        .zip(now_dt.as_ref())
                        .is_some_and(|(deadline, now)| {
                            deadline.timestamp_micros() > now.timestamp_micros()
                        });
                    if session.session_id != session_id
                        || session.client_id != client_id
                        || !matches!(
                            session.state,
                            common::models::TerminalSessionState::Pending
                                | common::models::TerminalSessionState::Active
                        )
                        || session.lease_id.as_deref() != Some(claim_id.as_str())
                        || !lease_valid
                    {
                        return None;
                    }

                    session.last_activity_at = Some(now.clone());
                    session.last_heartbeat_at = Some(now.clone());
                    session.lease_expires_at = Some(lease_expires_at.clone());
                    if session.state == common::models::TerminalSessionState::Pending {
                        session.state = common::models::TerminalSessionState::Active;
                        session.activated_at = Some(now.clone());
                    }
                    let Ok(bytes) = serde_json::to_vec(&session) else {
                        return None;
                    };
                    *updated_result.lock().expect("terminal output update lock") = Some(session);
                    Some(bytes)
                }),
            )
            .await?;

        let session = updated.lock().expect("terminal output update lock").clone();
        if session.is_some() {
            for chunk in chunks {
                let bytes = serde_json::to_vec(chunk).map_err(|e| {
                    CmdbError::Serialization(format!("serialize terminal output: {}", e))
                })?;
                self.db
                    .set(
                        &Self::output_key(
                            &output_session_id,
                            chunk.seq,
                            &Uuid::new_v4().to_string(),
                        ),
                        &bytes,
                    )
                    .await?;
            }
        }
        Ok(session)
    }

    /// Transition a terminal session only while the same agent claim is live.
    /// This is used for Active/Closed/Failed reports from agents.
    pub async fn update_state_if_claimed(
        &self,
        session_id: &str,
        client_id: &str,
        claim_id: &str,
        state: common::models::TerminalSessionState,
        message: Option<String>,
        now: &str,
        lease_expires_at: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let key = Self::session_key(session_id);
        let expected_key = key.clone();
        let updated = Arc::new(std::sync::Mutex::new(None::<TerminalSessionSummary>));
        let updated_result = updated.clone();
        let session_id = session_id.to_string();
        let client_id = client_id.to_string();
        let claim_id = claim_id.to_string();
        let now = now.to_string();
        let lease_expires_at = lease_expires_at.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut session) = serde_json::from_slice::<TerminalSessionSummary>(&value)
                    else {
                        return None;
                    };
                    let now_dt = DateTime::parse_from_rfc3339(&now).ok();
                    let lease_valid = session
                        .lease_expires_at
                        .as_deref()
                        .and_then(|deadline| DateTime::parse_from_rfc3339(deadline).ok())
                        .zip(now_dt.as_ref())
                        .is_some_and(|(deadline, now)| {
                            deadline.timestamp_micros() > now.timestamp_micros()
                        });
                    if session.session_id != session_id
                        || session.client_id != client_id
                        || !matches!(
                            session.state,
                            common::models::TerminalSessionState::Pending
                                | common::models::TerminalSessionState::Active
                        )
                        || session.lease_id.as_deref() != Some(claim_id.as_str())
                        || !lease_valid
                    {
                        return None;
                    }

                    session.state = state.clone();
                    session.last_activity_at = Some(now.clone());
                    session.last_heartbeat_at = Some(now.clone());
                    if state == common::models::TerminalSessionState::Active
                        && session.activated_at.is_none()
                    {
                        session.activated_at = Some(now.clone());
                    }
                    if matches!(
                        state,
                        common::models::TerminalSessionState::Closed
                            | common::models::TerminalSessionState::Failed
                    ) {
                        session.closed_at = Some(now.clone());
                        session.close_reason = message.clone();
                        session.lease_id = None;
                        session.lease_expires_at = None;
                    } else {
                        session.lease_expires_at = Some(lease_expires_at.clone());
                    }
                    let Ok(bytes) = serde_json::to_vec(&session) else {
                        return None;
                    };
                    *updated_result.lock().expect("terminal state update lock") = Some(session);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(updated.lock().expect("terminal state update lock").clone())
    }

    /// Close an active session only if the heartbeat observed by the caller
    /// is still current and is older than the supplied cutoff. This prevents
    /// stale-session cleanup from overwriting a concurrent heartbeat or claim.
    pub async fn close_stale_session(
        &self,
        session_id: &str,
        client_id: &str,
        expected_last_seen: &str,
        stale_before: &str,
        closed_at: &str,
    ) -> CmdbResult<Option<TerminalSessionSummary>> {
        let key = Self::session_key(session_id);
        let expected_key = key.clone();
        let closed = Arc::new(std::sync::Mutex::new(None::<TerminalSessionSummary>));
        let closed_result = closed.clone();
        let session_id = session_id.to_string();
        let client_id = client_id.to_string();
        let expected_last_seen = expected_last_seen.to_string();
        let stale_before = stale_before.to_string();
        let closed_at = closed_at.to_string();

        self.db
            .update_all(
                &key,
                Box::new(move |candidate_key, value| {
                    if candidate_key != expected_key {
                        return None;
                    }
                    let Ok(mut session) = serde_json::from_slice::<TerminalSessionSummary>(&value)
                    else {
                        return None;
                    };
                    let current_last_seen =
                        if session.state == common::models::TerminalSessionState::Pending {
                            session
                                .last_activity_at
                                .as_deref()
                                .or(Some(session.created_at.as_str()))
                        } else {
                            session
                                .last_heartbeat_at
                                .as_deref()
                                .or(session.activated_at.as_deref())
                        };
                    let stale = current_last_seen
                        .filter(|value| *value == expected_last_seen)
                        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                        .zip(chrono::DateTime::parse_from_rfc3339(&stale_before).ok())
                        .is_some_and(|(last_seen, cutoff)| last_seen <= cutoff);
                    if session.session_id != session_id
                        || session.client_id != client_id
                        || !matches!(
                            session.state,
                            common::models::TerminalSessionState::Pending
                                | common::models::TerminalSessionState::Active
                        )
                        || !stale
                    {
                        return None;
                    }

                    session.state = common::models::TerminalSessionState::Closed;
                    session.closed_at = Some(closed_at.clone());
                    session.last_activity_at = session.closed_at.clone();
                    session.close_reason = Some("agent heartbeat lost".to_string());
                    session.lease_id = None;
                    session.lease_expires_at = None;
                    let Ok(bytes) = serde_json::to_vec(&session) else {
                        return None;
                    };
                    *closed_result.lock().expect("stale session close lock") = Some(session);
                    Some(bytes)
                }),
            )
            .await?;

        Ok(closed.lock().expect("stale session close lock").clone())
    }

    pub async fn append_output(
        &self,
        session_id: &str,
        chunks: &[TerminalOutputChunk],
    ) -> CmdbResult<()> {
        for chunk in chunks {
            let bytes = serde_json::to_vec(chunk).map_err(|e| {
                CmdbError::Serialization(format!("serialize terminal output: {}", e))
            })?;
            self.db
                .set(
                    &Self::output_key(session_id, chunk.seq, &Uuid::new_v4().to_string()),
                    &bytes,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn get_output(&self, session_id: &str) -> CmdbResult<Vec<TerminalOutputChunk>> {
        let mut entries = self
            .db
            .list_entries(&Self::output_prefix(session_id))
            .await?;
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        entries
            .into_iter()
            .map(|(_, value)| {
                serde_json::from_slice(&value).map_err(|e| {
                    CmdbError::Serialization(format!("deserialize terminal output: {}", e))
                })
            })
            .collect()
    }

    pub async fn queue_input(
        &self,
        session_id: &str,
        chunk: &TerminalInputChunk,
    ) -> CmdbResult<()> {
        let bytes = serde_json::to_vec(chunk)
            .map_err(|e| CmdbError::Serialization(format!("serialize terminal input: {}", e)))?;
        self.db
            .set(&Self::input_key(session_id, chunk.seq), &bytes)
            .await
    }

    pub async fn drain_input(&self, session_id: &str) -> CmdbResult<Vec<TerminalInputChunk>> {
        let mut entries = self
            .db
            .list_entries(&Self::input_prefix(session_id))
            .await?;
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut chunks = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let chunk = serde_json::from_slice(&value).map_err(|e| {
                CmdbError::Serialization(format!("deserialize terminal input: {}", e))
            })?;
            chunks.push(chunk);
            let _ = self.db.delete(&key).await;
        }
        Ok(chunks)
    }

    pub async fn set_resize(
        &self,
        session_id: &str,
        resize: &TerminalResizeInstruction,
    ) -> CmdbResult<()> {
        let bytes = serde_json::to_vec(resize)
            .map_err(|e| CmdbError::Serialization(format!("serialize terminal resize: {}", e)))?;
        self.db.set(&Self::resize_key(session_id), &bytes).await
    }

    pub async fn take_resize(
        &self,
        session_id: &str,
    ) -> CmdbResult<Option<TerminalResizeInstruction>> {
        match self.db.get(&Self::resize_key(session_id)).await? {
            Some(bytes) => {
                let resize = serde_json::from_slice(&bytes).map_err(|e| {
                    CmdbError::Serialization(format!("deserialize terminal resize: {}", e))
                })?;
                let _ = self.db.delete(&Self::resize_key(session_id)).await;
                Ok(Some(resize))
            }
            None => Ok(None),
        }
    }

    pub async fn request_close(&self, session_id: &str) -> CmdbResult<()> {
        self.db.set(&Self::close_key(session_id), b"1").await
    }

    pub async fn take_close_request(&self, session_id: &str) -> CmdbResult<bool> {
        let key = Self::close_key(session_id);
        let exists = self.db.exists(&key).await?;
        if exists {
            let _ = self.db.delete(&key).await;
        }
        Ok(exists)
    }

    pub async fn take_attach_request(&self, session_id: &str) -> CmdbResult<bool> {
        let key = Self::attach_key(session_id);
        let exists = self.db.exists(&key).await?;
        if exists {
            let _ = self.db.delete(&key).await;
        }
        Ok(exists)
    }

    pub async fn list_open_sessions_for_client(
        &self,
        client_id: &str,
    ) -> CmdbResult<Vec<TerminalSessionSummary>> {
        let prefix = format!("term_session_idx:client:{}:", client_id);
        let mut keys = self.db.list_keys(&prefix).await?;
        keys.sort();
        let mut sessions = Vec::new();
        for key in keys {
            if let Some(session_id) = key.rsplit(':').next() {
                if let Some(session) = self.get_session(session_id).await? {
                    sessions.push(session);
                }
            }
        }
        Ok(sessions)
    }

    pub async fn list_all_sessions(&self) -> CmdbResult<Vec<TerminalSessionSummary>> {
        let mut keys = self.db.list_keys("term_session:").await?;
        keys.sort();
        let mut sessions = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(data) = self.db.get(&key).await? {
                let session = serde_json::from_slice(&data).map_err(|e| {
                    CmdbError::Serialization(format!("deserialize terminal session: {}", e))
                })?;
                sessions.push(session);
            }
        }
        Ok(sessions)
    }
}
