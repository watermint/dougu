# dougu-essentials-kvs

An embedded key-value store library wrapper for the dougu ecosystem.

## Features

- Persistent key-value storage
- ACID transactions
- Type-safe data storage and retrieval
- Asynchronous API

## Usage

```rust
use dougu_essentials_kvs::{KvsProvider, RedbKvsProvider};

async fn example() -> anyhow::Result<()> {
    // Initialize the KVS provider
    let kvs = RedbKvsProvider::new("path/to/database.redb")?;
    
    // Store a value
    kvs.set("key1", "value1").await?;
    
    // Retrieve a value
    let value: String = kvs.get("key1").await?;
    
    // Delete a value
    kvs.delete("key1").await?;
    
    Ok(())
}
``` 