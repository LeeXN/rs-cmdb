use crate::db::Database;
use common::error::{CmdbError, CmdbResult};
use common::models::{TerminalInputChunk, TerminalOutputChunk, TerminalResizeInstruction, TerminalSessionSummary};
use std::sync::Arc;

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
        format!("term_session_idx:client:{}:{}:{}", client_id, created_at, session_id)
    }

    fn output_key(session_id: &str, seq: u64) -> String {
        format!("term_output:{}:{:016}", session_id, seq)
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
        self.db.set(&Self::session_key(&session.session_id), &bytes).await?;
        self.db
            .set(
                &Self::client_index_key(&session.client_id, &session.created_at, &session.session_id),
                b"1",
            )
            .await?;
        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> CmdbResult<Option<TerminalSessionSummary>> {
        match self.db.get(&Self::session_key(session_id)).await? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| CmdbError::Serialization(format!("deserialize terminal session: {}", e))),
            None => Ok(None),
        }
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
            self.db.set(&Self::output_key(session_id, chunk.seq), &bytes).await?;
        }
        Ok(())
    }

    pub async fn get_output(&self, session_id: &str) -> CmdbResult<Vec<TerminalOutputChunk>> {
        let mut entries = self.db.list_entries(&Self::output_prefix(session_id)).await?;
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

    pub async fn queue_input(&self, session_id: &str, chunk: &TerminalInputChunk) -> CmdbResult<()> {
        let bytes = serde_json::to_vec(chunk)
            .map_err(|e| CmdbError::Serialization(format!("serialize terminal input: {}", e)))?;
        self.db.set(&Self::input_key(session_id, chunk.seq), &bytes).await
    }

    pub async fn drain_input(&self, session_id: &str) -> CmdbResult<Vec<TerminalInputChunk>> {
        let mut entries = self.db.list_entries(&Self::input_prefix(session_id)).await?;
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
