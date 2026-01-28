use super::Database;
use async_trait::async_trait;
use common::error::{CmdbError, CmdbResult};
use redb::{Database as RedbDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use std::sync::Arc;

// 定义表格，用于存储键值对
const KV_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("kv_store");

/// ReDB implementation of the Database trait
pub struct RedbStore {
    db: Arc<RedbDatabase>,
}

impl RedbStore {
    /// Create a new ReDB store
    pub fn new<P: AsRef<Path>>(path: P) -> CmdbResult<Self> {
        let db = RedbDatabase::create(path)
            .map_err(|e| CmdbError::Database(format!("Failed to open ReDB: {}", e)))?;

        // Initialize the table to ensure it exists (synchronous)
        {
            let write_txn = db.begin_write().map_err(|e| {
                CmdbError::Database(format!("Failed to start init transaction: {}", e))
            })?;

            // Simply open and close the table to ensure it gets created
            {
                let _table = write_txn.open_table(KV_TABLE).map_err(|e| {
                    CmdbError::Database(format!("Failed to initialize table: {}", e))
                })?;
            }

            write_txn.commit().map_err(|e| {
                CmdbError::Database(format!("Failed to commit init transaction: {}", e))
            })?;
        }

        let store = Self { db: Arc::new(db) };

        Ok(store)
    }
}

#[async_trait]
impl Database for RedbStore {
    async fn set(&self, key: &str, value: &[u8]) -> CmdbResult<()> {
        let db = self.db.clone();
        let key = key.to_string();
        let value = value.to_vec();

        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write().map_err(|e| {
                CmdbError::Database(format!("Failed to start write transaction: {}", e))
            })?;

            {
                let mut table = write_txn
                    .open_table(KV_TABLE)
                    .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

                table.insert(key.as_str(), value.as_slice()).map_err(|e| {
                    CmdbError::Database(format!("Failed to set key {}: {}", key, e))
                })?;
            }

            write_txn
                .commit()
                .map_err(|e| CmdbError::Database(format!("Failed to commit transaction: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn get(&self, key: &str) -> CmdbResult<Option<Vec<u8>>> {
        let db = self.db.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(|e| {
                CmdbError::Database(format!("Failed to start read transaction: {}", e))
            })?;

            let table = read_txn
                .open_table(KV_TABLE)
                .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

            match table.get(key.as_str()) {
                Ok(Some(value)) => {
                    let value_vec = value.value().to_vec();
                    Ok(Some(value_vec))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(CmdbError::Database(format!(
                    "Failed to get key {}: {}",
                    key, e
                ))),
            }
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn delete(&self, key: &str) -> CmdbResult<()> {
        let db = self.db.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write().map_err(|e| {
                CmdbError::Database(format!("Failed to start write transaction: {}", e))
            })?;

            {
                let mut table = write_txn
                    .open_table(KV_TABLE)
                    .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

                let _ = table.remove(key.as_str()).map_err(|e| {
                    CmdbError::Database(format!("Failed to delete key {}: {}", key, e))
                })?;
            }

            write_txn
                .commit()
                .map_err(|e| CmdbError::Database(format!("Failed to commit transaction: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_keys(&self, prefix: &str) -> CmdbResult<Vec<String>> {
        let db = self.db.clone();
        let prefix = prefix.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(|e| {
                CmdbError::Database(format!("Failed to start read transaction: {}", e))
            })?;

            let table = read_txn
                .open_table(KV_TABLE)
                .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

            let mut keys = Vec::new();

            {
                let iter = table
                    .iter()
                    .map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;

                for item in iter {
                    let (key, _) =
                        item.map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;
                    let key_str = key.value();
                    if key_str.starts_with(&prefix) {
                        keys.push(key_str.to_string());
                    }
                }
            }

            Ok(keys)
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_values(&self, prefix: &str) -> CmdbResult<Vec<Vec<u8>>> {
        let db = self.db.clone();
        let prefix = prefix.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(|e| {
                CmdbError::Database(format!("Failed to start read transaction: {}", e))
            })?;

            let table = read_txn
                .open_table(KV_TABLE)
                .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

            let mut values = Vec::new();

            {
                let iter = table
                    .iter()
                    .map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;

                for item in iter {
                    let (key, value) =
                        item.map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;
                    let key_str = key.value();
                    if key_str.starts_with(&prefix) {
                        values.push(value.value().to_vec());
                    }
                }
            }

            Ok(values)
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn list_entries(&self, prefix: &str) -> CmdbResult<Vec<(String, Vec<u8>)>> {
        let db = self.db.clone();
        let prefix = prefix.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db.begin_read().map_err(|e| {
                CmdbError::Database(format!("Failed to start read transaction: {}", e))
            })?;

            let table = read_txn
                .open_table(KV_TABLE)
                .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

            let mut entries = Vec::new();

            {
                let iter = table
                    .iter()
                    .map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;

                for item in iter {
                    let (key, value) =
                        item.map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;
                    let key_str = key.value();
                    if key_str.starts_with(&prefix) {
                        entries.push((key_str.to_string(), value.value().to_vec()));
                    }
                }
            }

            Ok(entries)
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }

    async fn exists(&self, key: &str) -> CmdbResult<bool> {
        let result = self.get(key).await?;
        Ok(result.is_some())
    }
}
