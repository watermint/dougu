use anyhow::Result;
use dougu_essentials_kvs::{KvsProvider, RedbKvsProvider};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new database or open an existing one
    let kvs = RedbKvsProvider::new("example.redb")?;
    
    // Store a string value
    kvs.set("string_key", "Hello, KVS!").await?;
    
    // Store a numeric value
    kvs.set("numeric_key", 42i32).await?;
    
    // Store a complex type using serde
    let complex_value = vec!["item1", "item2", "item3"];
    kvs.set("complex_key", &complex_value).await?;
    
    // Retrieve values
    let string_value: String = kvs.get("string_key").await?;
    println!("String value: {}", string_value);
    
    let numeric_value: i32 = kvs.get("numeric_key").await?;
    println!("Numeric value: {}", numeric_value);
    
    let complex_value: Vec<String> = kvs.get("complex_key").await?;
    println!("Complex value: {:?}", complex_value);
    
    // Check if a key exists
    let exists = kvs.exists("string_key").await?;
    println!("Key 'string_key' exists: {}", exists);
    
    // Delete a key
    kvs.delete("string_key").await?;
    
    // Check that it no longer exists
    let exists = kvs.exists("string_key").await?;
    println!("Key 'string_key' exists after deletion: {}", exists);
    
    Ok(())
} 