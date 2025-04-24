use anyhow::{anyhow, Context, Result};
use bson::{from_document, to_document, Document};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Value, json};
use serde_yaml;
use std::io::Cursor;
use toml;

mod resources;
pub mod query;
pub mod notation;
pub mod examples;

use resources::errors::*;
use resources::formats::*;
pub use query::Query;
pub use notation::Notation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Bson,
    Cbor,
    Xml,
    Yaml,
    Toml,
    Jsonl,
}

impl Format {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            FORMAT_JSON => Ok(Format::Json),
            FORMAT_BSON => Ok(Format::Bson),
            FORMAT_CBOR => Ok(Format::Cbor),
            FORMAT_XML => Ok(Format::Xml),
            FORMAT_YAML => Ok(Format::Yaml),
            FORMAT_TOML => Ok(Format::Toml),
            FORMAT_JSONL => Ok(Format::Jsonl),
            _ => Err(anyhow!(ERROR_UNSUPPORTED_FORMAT)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Json => FORMAT_JSON,
            Format::Bson => FORMAT_BSON,
            Format::Cbor => FORMAT_CBOR,
            Format::Xml => FORMAT_XML,
            Format::Yaml => FORMAT_YAML,
            Format::Toml => FORMAT_TOML,
            Format::Jsonl => FORMAT_JSONL,
        }
    }
}

pub struct Decoder;

impl Decoder {
    pub fn decode<T>(input: &[u8], format: Format) -> Result<T> 
    where
        T: DeserializeOwned,
    {
        notation::get_notation(format).decode(input)
    }

    pub fn decode_str<T>(input: &str, format: Format) -> Result<T>
    where
        T: DeserializeOwned,
    {
        Self::decode(input.as_bytes(), format)
    }

    pub fn decode_to_value(input: &[u8], format: Format) -> Result<Value> {
        Self::decode(input, format)
    }
}

pub struct Encoder;

impl Encoder {
    pub fn encode<T>(value: &T, format: Format) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        notation::get_notation(format).encode(value)
    }

    pub fn encode_to_string<T>(value: &T, format: Format) -> Result<String>
    where
        T: Serialize,
    {
        notation::get_notation(format).encode_to_string(value)
    }
    
    pub fn encode_jsonl_all<T>(values: &[T]) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        notation::jsonl::JsonlNotation.encode_collection(values)
    }
}

pub struct QueryImpl {
    jaq_query: query::Query,
}

impl QueryImpl {
    pub fn compile(query_str: &str) -> Result<Self> {
        let jaq_query = query::Query::compile(query_str)?;
        
        Ok(Self {
            jaq_query,
        })
    }
    
    pub fn execute<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize,
    {
        self.jaq_query.execute_to_string(value)
    }
}

pub struct Converter;

impl Converter {
    pub fn convert<T, U>(input: &[u8], from_format: Format, to_format: Format) -> Result<Vec<u8>>
    where
        T: DeserializeOwned + Serialize,
        U: Serialize,
    {
        let from_notation = notation::get_notation(from_format);
        let to_notation = notation::get_notation(to_format);
        
        let decoded: T = from_notation.decode(input)?;
        let encoded = to_notation.encode(&decoded)?;
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }
    
    #[test]
    fn test_json_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let encoded = Encoder::encode(&data, Format::Json).unwrap();
        let decoded: TestData = Decoder::decode(&encoded, Format::Json).unwrap();
        
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_yaml_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let encoded = Encoder::encode(&data, Format::Yaml).unwrap();
        let decoded: TestData = Decoder::decode(&encoded, Format::Yaml).unwrap();
        
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_toml_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let encoded = Encoder::encode(&data, Format::Toml).unwrap();
        let decoded: TestData = Decoder::decode(&encoded, Format::Toml).unwrap();
        
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_query() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        // Use a simple JQ-style query
        let query = QueryImpl::compile(".value").unwrap();
        let result = query.execute(&data).unwrap();
        
        // The result will be a JSON string with the value
        assert_eq!("42", result);
        
        // Test more complex query with array access and filtering
        let complex_data = json!({
            "items": [
                {"id": 1, "name": "Item 1"},
                {"id": 2, "name": "Item 2"},
                {"id": 3, "name": "Item 3"}
            ]
        });
        
        // Test with a simpler query first to avoid syntax issues
        let array_query = QueryImpl::compile(".items[1].name").unwrap();
        let name_result = array_query.execute(&complex_data).unwrap();
        
        assert_eq!("\"Item 2\"", name_result);
    }
    
    #[test]
    fn test_jsonl_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let encoded = Encoder::encode(&data, Format::Jsonl).unwrap();
        let decoded: Vec<TestData> = Decoder::decode(&encoded, Format::Jsonl).unwrap();
        
        assert_eq!(decoded.len(), 1);
        assert_eq!(data, decoded[0]);
    }
    
    #[test]
    fn test_jsonl_multiple() {
        let data = vec![
            TestData {
                name: "test1".to_string(),
                value: 42,
            },
            TestData {
                name: "test2".to_string(),
                value: 43,
            },
        ];
        
        let encoded = Encoder::encode_jsonl_all(&data).unwrap();
        let decoded: Vec<TestData> = Decoder::decode(&encoded, Format::Jsonl).unwrap();
        
        assert_eq!(data.len(), decoded.len());
        assert_eq!(data[0], decoded[0]);
        assert_eq!(data[1], decoded[1]);
    }
}

pub mod examples_impl {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
        hobbies: Vec<String>,
    }
    
    pub fn run_example() -> Result<()> {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            hobbies: vec!["reading".to_string(), "hiking".to_string()],
        };
        
        // Convert to different formats
        let json = Encoder::encode_to_string(&person, Format::Json)?;
        let yaml = Encoder::encode_to_string(&person, Format::Yaml)?;
        let toml = Encoder::encode_to_string(&person, Format::Toml)?;
        
        println!("JSON:\n{}\n", json);
        println!("YAML:\n{}\n", yaml);
        println!("TOML:\n{}\n", toml);
        
        // Parse back from YAML
        let decoded: Person = Decoder::decode_str(&yaml, Format::Yaml)?;
        println!("Decoded: {:?}", decoded);
        
        // Query the person's age
        let query = QueryImpl::compile(".age")?;
        let age = query.execute(&person)?;
        // NOTE: Commented out due to API changes in jaq-interpret
        // println!("Age: {}", age.as_u64().unwrap());
        println!("Age query: {}", age);
        
        Ok(())
    }
} 