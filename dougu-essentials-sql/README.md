# dougu-essentials-sql

A Rust library providing a simple abstraction layer for embedded SQL databases, with a focus on SQLite.

## Features

- Asynchronous API for database operations
- Straightforward interface for executing SQL queries
- Transaction support
- Error handling with descriptive messages
- Type-safe conversion between Rust and SQL types
- Bundled SQLite with no external dependencies required

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
dougu-essentials-sql = { path = "../dougu-essentials-sql" }
```

### Basic Example

```rust
use dougu_essentials_sql::{SqliteProvider, SqlValue, SqlProvider};
use anyhow::Result;

async fn example() -> Result<()> {
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
    
    // Query data
    let rows = db.query(
        "SELECT * FROM users WHERE age > ?",
        &[SqlValue::Integer(20)],
    ).await?;
    
    // Process results
    for row in rows {
        // Process row data...
    }
    
    Ok(())
}
```

### Transactions

```rust
// Transaction example
db.transaction(|provider| {
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
```

## License

This project is part of the dougu toolkit. 