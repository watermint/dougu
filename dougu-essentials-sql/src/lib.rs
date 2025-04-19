use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rusqlite::{Connection, params, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use tokio::task;

mod resources;
use resources::Messages;

#[cfg(feature = "examples")]
pub mod examples;

/// A generic SQL result row.
#[derive(Debug, Clone)]
pub struct SqlRow {
    pub values: Vec<SqlValue>,
}

/// Represents a value that can be stored in or retrieved from a database.
#[derive(Debug, Clone)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// Trait defining SQL database operations.
#[async_trait]
pub trait SqlProvider {
    /// Execute a SQL query without returning any rows.
    async fn execute(&self, query: &str, params: &[SqlValue]) -> Result<usize>;
    
    /// Execute a SQL query and return the resulting rows.
    async fn query(&self, query: &str, params: &[SqlValue]) -> Result<Vec<SqlRow>>;
    
    /// Execute a SQL query and return a single row.
    async fn query_row(&self, query: &str, params: &[SqlValue]) -> Result<SqlRow>;
    
    /// Begin a transaction.
    async fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&dyn SqlProvider) -> Result<T> + Send + 'static,
        T: Send + 'static;
}

/// SQLite implementation of the SqlProvider trait.
pub struct SqliteProvider {
    path: String,
}

impl SqliteProvider {
    /// Create a new SQLite database connection.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid path"))?
            .to_string();
        
        // Test connection to ensure the database is accessible
        let _ = Connection::open(&path_str)
            .map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
        
        Ok(Self { path: path_str })
    }
    
    /// Create or open an in-memory SQLite database.
    pub fn memory() -> Result<Self> {
        // Test connection to ensure in-memory database works
        let _ = Connection::open_in_memory()
            .map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
        
        Ok(Self { path: ":memory:".to_string() })
    }
    
    /// Convert Rust types to SqlValues.
    fn to_sql_value<T: Serialize>(value: T) -> Result<SqlValue> {
        let json = serde_json::to_string(&value)
            .map_err(|e| anyhow!(format!("{}: {}", Messages::SERIALIZATION_ERROR, e)))?;
        Ok(SqlValue::Text(json))
    }
    
    /// Convert SqlValues back to Rust types.
    fn from_sql_value<T: DeserializeOwned>(value: &SqlValue) -> Result<T> {
        match value {
            SqlValue::Text(text) => {
                serde_json::from_str(text)
                    .map_err(|e| anyhow!(format!("{}: {}", Messages::DESERIALIZATION_ERROR, e)))
            },
            _ => Err(anyhow!(format!("{}: Unexpected value type", Messages::DESERIALIZATION_ERROR))),
        }
    }
    
    /// Convert SqlValue to rusqlite::Value for parameter binding.
    fn to_rusqlite_param(value: &SqlValue) -> rusqlite::types::Value {
        match value {
            SqlValue::Null => rusqlite::types::Value::Null,
            SqlValue::Integer(i) => rusqlite::types::Value::Integer(*i),
            SqlValue::Real(r) => rusqlite::types::Value::Real(*r),
            SqlValue::Text(t) => rusqlite::types::Value::Text(t.clone()),
            SqlValue::Blob(b) => rusqlite::types::Value::Blob(b.clone()),
        }
    }
    
    /// Convert rusqlite::Value to SqlValue.
    fn from_rusqlite_value(value: rusqlite::types::Value) -> SqlValue {
        match value {
            rusqlite::types::Value::Null => SqlValue::Null,
            rusqlite::types::Value::Integer(i) => SqlValue::Integer(i),
            rusqlite::types::Value::Real(r) => SqlValue::Real(r),
            rusqlite::types::Value::Text(t) => SqlValue::Text(t),
            rusqlite::types::Value::Blob(b) => SqlValue::Blob(b),
        }
    }
}

#[async_trait]
impl SqlProvider for SqliteProvider {
    async fn execute(&self, query: &str, params: &[SqlValue]) -> Result<usize> {
        let query = query.to_string();
        let params: Vec<SqlValue> = params.to_vec();
        let path = self.path.clone();
        
        task::spawn_blocking(move || {
            let conn = Connection::open(&path)
                .map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
            
            let params: Vec<rusqlite::types::Value> = params.iter()
                .map(Self::to_rusqlite_param)
                .collect();
            
            let result = conn.execute(&query, rusqlite::params_from_iter(params.iter()))
                .map_err(|e| anyhow!(format!("{}: {}", Messages::QUERY_EXECUTION_ERROR, e)))?;
            
            Ok(result)
        }).await?
    }
    
    async fn query(&self, query: &str, params: &[SqlValue]) -> Result<Vec<SqlRow>> {
        let query = query.to_string();
        let params: Vec<SqlValue> = params.to_vec();
        let path = self.path.clone();
        
        task::spawn_blocking(move || {
            let conn = Connection::open(&path)
                .map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
            
            let params: Vec<rusqlite::types::Value> = params.iter()
                .map(Self::to_rusqlite_param)
                .collect();
            
            let mut stmt = conn.prepare(&query)
                .map_err(|e| anyhow!(format!("{}: {}", Messages::QUERY_EXECUTION_ERROR, e)))?;
            
            let column_count = stmt.column_count();
            
            let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
                let mut values = Vec::with_capacity(column_count);
                
                for i in 0..column_count {
                    let value = row.get_ref(i)
                        .map_err(|e| anyhow!(format!("{}: {}", Messages::COLUMN_FETCH_ERROR, e)))?;
                    
                    let sql_value = match value {
                        rusqlite::types::ValueRef::Null => SqlValue::Null,
                        rusqlite::types::ValueRef::Integer(i) => SqlValue::Integer(i),
                        rusqlite::types::ValueRef::Real(r) => SqlValue::Real(r),
                        rusqlite::types::ValueRef::Text(t) => SqlValue::Text(String::from_utf8_lossy(t).to_string()),
                        rusqlite::types::ValueRef::Blob(b) => SqlValue::Blob(b.to_vec()),
                    };
                    
                    values.push(sql_value);
                }
                
                Ok(SqlRow { values })
            })
            .map_err(|e| anyhow!(format!("{}: {}", Messages::QUERY_EXECUTION_ERROR, e)))?;
            
            let mut result = Vec::new();
            for row in rows {
                result.push(row.map_err(|e| anyhow!(format!("{}: {}", Messages::ROW_FETCH_ERROR, e)))?);
            }
            
            Ok(result)
        }).await?
    }
    
    async fn query_row(&self, query: &str, params: &[SqlValue]) -> Result<SqlRow> {
        let rows = self.query(query, params).await?;
        rows.into_iter().next().ok_or_else(|| anyhow!("No rows returned"))
    }
    
    async fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&dyn SqlProvider) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let path = self.path.clone();
        
        task::spawn_blocking(move || {
            let conn = Connection::open(&path)
                .map_err(|e| anyhow!(format!("{}: {}", Messages::DATABASE_OPEN_ERROR, e)))?;
            
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_BEGIN_ERROR, e)))?;
            
            // Create a temporary provider for the transaction
            let provider = SqliteProvider { path: path.clone() };
            
            let result = match f(&provider) {
                Ok(result) => {
                    tx.commit()
                        .map_err(|e| anyhow!(format!("{}: {}", Messages::TRANSACTION_COMMIT_ERROR, e)))?;
                    Ok(result)
                },
                Err(e) => {
                    let _ = tx.rollback();
                    Err(e)
                }
            };
            
            result
        }).await?
    }
} 