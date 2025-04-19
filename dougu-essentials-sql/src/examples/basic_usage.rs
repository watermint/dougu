use crate::{SqlProvider, SqliteProvider, SqlValue};
use anyhow::Result;

/// Example showing basic usage of the SQLite provider.
pub async fn basic_sqlite_example() -> Result<()> {
    // Create an in-memory database
    let db = SqliteProvider::memory()?;
    
    // Create a table
    db.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)",
        &[],
    ).await?;
    
    // Insert data
    db.execute(
        "INSERT INTO users (name, age) VALUES (?, ?)",
        &[
            SqlValue::Text("Alice".to_string()),
            SqlValue::Integer(30),
        ],
    ).await?;
    
    db.execute(
        "INSERT INTO users (name, age) VALUES (?, ?)",
        &[
            SqlValue::Text("Bob".to_string()),
            SqlValue::Integer(25),
        ],
    ).await?;
    
    // Query data
    let rows = db.query(
        "SELECT id, name, age FROM users WHERE age > ?",
        &[SqlValue::Integer(20)],
    ).await?;
    
    // Process results
    for row in rows {
        let id = match &row.values[0] {
            SqlValue::Integer(i) => *i,
            _ => 0,
        };
        
        let name = match &row.values[1] {
            SqlValue::Text(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        
        let age = match &row.values[2] {
            SqlValue::Integer(i) => *i,
            _ => 0,
        };
        
        println!("User {}: {} ({})", id, name, age);
    }
    
    // Transaction example
    db.transaction(|provider| {
        let provider = provider as &dyn SqlProvider;
        
        // These operations will be rolled back if any of them fail
        provider.execute(
            "INSERT INTO users (name, age) VALUES (?, ?)",
            &[
                SqlValue::Text("Charlie".to_string()),
                SqlValue::Integer(40),
            ],
        )?;
        
        provider.execute(
            "UPDATE users SET age = ? WHERE name = ?",
            &[
                SqlValue::Integer(26),
                SqlValue::Text("Bob".to_string()),
            ],
        )?;
        
        Ok(())
    }).await?;
    
    Ok(())
} 