// Key-Value Store operations module

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tokio::task;
use std::sync::Arc;

mod resources;
use resources::Messages;

#[cfg(feature = "examples")]
pub mod examples;

/// A generic key-value pair that associates a key of type K with a value of type V.
#[derive(Debug, Clone)]
pub struct KeyValuePair<K, V> {
    pub key: K,
    pub value: V,
}

/// Trait defining the operations that can be performed on a key-value store.
#[async_trait]
pub trait KvsProvider {
    /// Store a key-value pair in the database.
    async fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
        V: Serialize + Send + Sync + 'static;

    /// Retrieve a value from the database using its key.
    async fn get<K, V>(&self, key: K) -> Result<V>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
        V: DeserializeOwned + Send + Sync + 'static;

    /// Delete a key-value pair from the database.
    async fn delete<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]> + Send + Sync + 'static;

    /// Check if a key exists in the database.
    async fn exists<K>(&self, key: K) -> Result<bool>
    where
        K: AsRef<[u8]> + Send + Sync + 'static;
}

/// An implementation of the KvsProvider trait using the redb embedded database.
pub struct RedbKvsProvider {
    db: Arc<Database>,
}

impl RedbKvsProvider {
    /// Define the table used for storing key-value pairs.
    const KV_TABLE: TableDefinition<'static, &'static [u8], &'static [u8]> = TableDefinition::new("kv_store");

    /// Create a new RedbKvsProvider with the database at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::create(path).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Open an existing RedbKvsProvider with the database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = Database::open(path).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl KvsProvider for RedbKvsProvider {
    async fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
        V: Serialize + Send + Sync + 'static,
    {
        let key_bytes = key.as_ref().to_vec();
        let value_bytes = task::spawn_blocking(move || {
            serde_json::to_vec(&value).map_err(|e| anyhow!(format!("{}: {}", Messages::SERIALIZATION_ERROR, e)))
        })
        .await??;

        task::spawn_blocking({
            let db = Arc::clone(&self.db);
            move || {
                let write_txn = db.begin_write().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                let mut table = write_txn.open_table(Self::KV_TABLE).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_WRITE_ERROR, e)))?;
                table.insert(key_bytes.as_slice(), value_bytes.as_slice()).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_WRITE_ERROR, e)))?;
                drop(table);
                write_txn.commit().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                Ok::<_, anyhow::Error>(())
            }
        })
        .await?
    }

    async fn get<K, V>(&self, key: K) -> Result<V>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
        V: DeserializeOwned + Send + Sync + 'static,
    {
        let key_bytes = key.as_ref().to_vec();

        let value_bytes = task::spawn_blocking({
            let db = Arc::clone(&self.db);
            move || {
                let read_txn = db.begin_read().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                let table = read_txn.open_table(Self::KV_TABLE).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_READ_ERROR, e)))?;
                
                match table.get(key_bytes.as_slice()) {
                    Ok(Some(value)) => Ok(value.value().to_vec()),
                    Ok(None) => Err(anyhow!(Messages::KEY_NOT_FOUND)),
                    Err(e) => Err(anyhow!(format!("{}: {}", Messages::DATABASE_READ_ERROR, e))),
                }
            }
        })
        .await??;

        task::spawn_blocking(move || {
            serde_json::from_slice(&value_bytes).map_err(|e| anyhow!(format!("{}: {}", Messages::DESERIALIZATION_ERROR, e)))
        })
        .await?
    }

    async fn delete<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
    {
        let key_bytes = key.as_ref().to_vec();

        task::spawn_blocking({
            let db = Arc::clone(&self.db);
            move || {
                let write_txn = db.begin_write().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                let mut table = write_txn.open_table(Self::KV_TABLE).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_DELETE_ERROR, e)))?;
                
                table.remove(key_bytes.as_slice()).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_DELETE_ERROR, e)))?;
                drop(table);
                write_txn.commit().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                Ok::<_, anyhow::Error>(())
            }
        })
        .await?
    }

    async fn exists<K>(&self, key: K) -> Result<bool>
    where
        K: AsRef<[u8]> + Send + Sync + 'static,
    {
        let key_bytes = key.as_ref().to_vec();

        task::spawn_blocking({
            let db = Arc::clone(&self.db);
            move || {
                let read_txn = db.begin_read().map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_ERROR, e)))?;
                let table = read_txn.open_table(Self::KV_TABLE).map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_READ_ERROR, e)))?;
                
                match table.get(key_bytes.as_slice()) {
                    Ok(Some(_)) => Ok(true),
                    Ok(None) => Ok(false),
                    Err(e) => Err(anyhow!(format!("{}: {}", Messages::DATABASE_READ_ERROR, e))),
                }
            }
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_creates_key_value_pair() {
        let kv = KeyValuePair {
            key: "test_key",
            value: "test_value",
        };
        assert_eq!(kv.key, "test_key");
        assert_eq!(kv.value, "test_value");
    }
} 