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

    async fn update_all(
        &self,
        prefix: &str,
        callback: Box<dyn Fn(String, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>,
    ) -> CmdbResult<()> {
        let db = self.db.clone();
        let prefix = prefix.to_string();

        tokio::task::spawn_blocking(move || {
            let write_txn = db.begin_write().map_err(|e| {
                CmdbError::Database(format!("Failed to start write transaction: {}", e))
            })?;

            {
                let mut table = write_txn
                    .open_table(KV_TABLE)
                    .map_err(|e| CmdbError::Database(format!("Failed to open table: {}", e)))?;

                // First collecting keys to avoid borrowing issues during iteration/update
                let mut updates = Vec::new();

                {
                    let iter = table
                        .iter()
                        .map_err(|e| CmdbError::Database(format!("Failed to iterate: {}", e)))?;

                    for item in iter {
                        let (key, value) = item.map_err(|e| {
                            CmdbError::Database(format!("Failed to iterate: {}", e))
                        })?;
                        let key_str = key.value();
                        if key_str.starts_with(&prefix) {
                            let value_vec = value.value().to_vec();
                            if let Some(new_value) = callback(key_str.to_string(), value_vec) {
                                updates.push((key_str.to_string(), new_value));
                            }
                        }
                    }
                }

                // Apply updates
                for (key, value) in updates {
                    table.insert(key.as_str(), value.as_slice()).map_err(|e| {
                        CmdbError::Database(format!("Failed to update key {}: {}", key, e))
                    })?;
                }
            }

            write_txn
                .commit()
                .map_err(|e| CmdbError::Database(format!("Failed to commit transaction: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| CmdbError::Database(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn setup_temp_store() -> RedbStore {
        let path = std::env::temp_dir().join(format!(
            "redb_test_{}_{}.db",
            std::process::id(),
            rand::random::<u64>()
        ));
        let _ = std::fs::remove_file(&path);
        RedbStore::new(&path).unwrap()
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let store = setup_temp_store();
        store.set("key1", b"value1").await.unwrap();
        let val = store.get("key1").await.unwrap();
        assert_eq!(val, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_key() {
        let store = setup_temp_store();
        let val = store.get("nonexistent").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_delete_key() {
        let store = setup_temp_store();
        store.set("todelete", b"val").await.unwrap();
        assert!(store.exists("todelete").await.unwrap());
        store.delete("todelete").await.unwrap();
        assert!(!store.exists("todelete").await.unwrap());
        assert_eq!(store.get("todelete").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_overwrite_value() {
        let store = setup_temp_store();
        store.set("k", b"old").await.unwrap();
        store.set("k", b"new").await.unwrap();
        let val = store.get("k").await.unwrap();
        assert_eq!(val, Some(b"new".to_vec()));
    }

    #[tokio::test]
    async fn test_list_keys_by_prefix() {
        let store = setup_temp_store();
        store.set("client:a", b"1").await.unwrap();
        store.set("client:b", b"2").await.unwrap();
        store.set("hardware:a", b"3").await.unwrap();
        store.set("user:x", b"4").await.unwrap();

        let client_keys = store.list_keys("client:").await.unwrap();
        assert_eq!(client_keys.len(), 2);
        assert!(client_keys.contains(&"client:a".to_string()));
        assert!(client_keys.contains(&"client:b".to_string()));

        let all = store.list_keys("").await.unwrap();
        assert_eq!(all.len(), 4);
    }

    #[tokio::test]
    async fn test_list_values_by_prefix() {
        let store = setup_temp_store();
        store.set("pref:a", b"val_a").await.unwrap();
        store.set("pref:b", b"val_b").await.unwrap();

        let vals = store.list_values("pref:").await.unwrap();
        assert_eq!(vals.len(), 2);
    }

    #[tokio::test]
    async fn test_list_entries() {
        let store = setup_temp_store();
        store.set("e:1", b"one").await.unwrap();
        store.set("e:2", b"two").await.unwrap();

        let entries = store.list_entries("e:").await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.contains(&("e:1".to_string(), b"one".to_vec())));
        assert!(entries.contains(&("e:2".to_string(), b"two".to_vec())));
    }

    #[tokio::test]
    async fn test_update_all() {
        let store = Arc::new(setup_temp_store());
        store.set("upd:1", b"hello").await.unwrap();
        store.set("upd:2", b"world").await.unwrap();
        store.set("other:1", b"keep").await.unwrap();

        store
            .update_all(
                "upd:",
                Box::new(|_k, v| {
                    let mut new = b"prefix-".to_vec();
                    new.extend_from_slice(&v);
                    Some(new)
                }),
            )
            .await
            .unwrap();

        let v1 = store.get("upd:1").await.unwrap().unwrap();
        assert_eq!(v1, b"prefix-hello");
        let v2 = store.get("upd:2").await.unwrap().unwrap();
        assert_eq!(v2, b"prefix-world");
        let kept = store.get("other:1").await.unwrap().unwrap();
        assert_eq!(kept, b"keep");
    }

    #[tokio::test]
    async fn test_update_all_noop_callback() {
        let store = setup_temp_store();
        store.set("nop:1", b"data").await.unwrap();
        store
            .update_all("nop:", Box::new(|_k, _v| None))
            .await
            .unwrap();
        let val = store.get("nop:1").await.unwrap().unwrap();
        assert_eq!(val, b"data");
    }

    #[tokio::test]
    async fn test_large_value() {
        let store = setup_temp_store();
        let large = vec![0xABu8; 100_000];
        store.set("large", &large).await.unwrap();
        let retrieved = store.get("large").await.unwrap().unwrap();
        assert_eq!(retrieved.len(), 100_000);
        assert_eq!(retrieved[0], 0xAB);
    }

    #[tokio::test]
    async fn test_empty_value() {
        let store = setup_temp_store();
        store.set("empty", b"").await.unwrap();
        let val = store.get("empty").await.unwrap().unwrap();
        assert!(val.is_empty());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let store = Arc::new(setup_temp_store());
        let mut handles = Vec::new();

        for i in 0..10 {
            let s = store.clone();
            handles.push(tokio::spawn(async move {
                s.set(&format!("con:{}", i), &[i]).await.unwrap();
                let v = s.get(&format!("con:{}", i)).await.unwrap();
                assert_eq!(v, Some(vec![i]));
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let keys = store.list_keys("con:").await.unwrap();
        assert_eq!(keys.len(), 10);
    }
}
