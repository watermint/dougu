use anyhow::Result;
use dougu_essentials_obj::{Encoder, Decoder, Format, Query};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    created_at: String,
    tags: Vec<String>,
}

fn main() -> Result<()> {
    // Create sample data
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
        metadata: Metadata {
            created_at: "2023-01-01T00:00:00Z".to_string(),
            tags: vec!["customer".to_string(), "premium".to_string()],
        },
    };

    // Encode to JSON
    let json_bytes = Encoder::encode(&user, Format::Json)?;
    let json_str = String::from_utf8_lossy(&json_bytes);
    println!("JSON:\n{}", json_str);

    // Encode to XML
    let xml_bytes = Encoder::encode(&user, Format::Xml)?;
    let xml_str = String::from_utf8_lossy(&xml_bytes);
    println!("\nXML:\n{}", xml_str);

    // Encode to CBOR and BSON (binary formats)
    let cbor_bytes = Encoder::encode(&user, Format::Cbor)?;
    println!("\nCBOR (hex):\n{}", hex::encode(&cbor_bytes));

    let bson_bytes = Encoder::encode(&user, Format::Bson)?;
    println!("\nBSON (hex):\n{}", hex::encode(&bson_bytes));

    // Decode from JSON
    let decoded_user: User = Decoder::decode(&json_bytes, Format::Json)?;
    println!("\nDecoded user from JSON:\n{:#?}", decoded_user);

    // Use query language (similar to jq)
    let query = Query::compile(".metadata.tags[]")?;
    let result = query.execute(&user)?;
    println!("\nQuery result (.metadata.tags[]):\n{}", result);

    // Convert between formats (JSON to XML)
    let json_to_xml = Encoder::encode_to_string(&decoded_user, Format::Xml)?;
    println!("\nJSON converted to XML:\n{}", json_to_xml);

    Ok(())
} 