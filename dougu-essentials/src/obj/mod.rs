use anyhow::{anyhow, Context, Result};
use bson::{from_document, to_document, Document};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use serde_yaml;
use std::io::Cursor;
use toml;

mod resources;
use resources::errors::*;
use resources::formats::*;

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
        match format {
            Format::Json => serde_json::from_slice(input)
                .with_context(|| ERROR_DECODE_FAILED),
            
            Format::Bson => {
                let doc = Document::from_reader(Cursor::new(input))
                    .with_context(|| ERROR_DECODE_FAILED)?;
                from_document(doc).with_context(|| ERROR_DECODE_FAILED)
            },
            
            Format::Cbor => ciborium::de::from_reader(input)
                .with_context(|| ERROR_DECODE_FAILED),
            
            Format::Xml => {
                quick_xml::de::from_reader(input)
                    .with_context(|| ERROR_DECODE_FAILED)
            },

            Format::Yaml => {
                serde_yaml::from_slice(input)
                    .with_context(|| ERROR_DECODE_FAILED)
            },

            Format::Toml => {
                let s = std::str::from_utf8(input)
                    .with_context(|| ERROR_DECODE_FAILED)?;
                toml::from_str(s)
                    .with_context(|| ERROR_DECODE_FAILED)
            },
            
            Format::Jsonl => {
                let s = std::str::from_utf8(input)
                    .with_context(|| ERROR_DECODE_FAILED)?;
                
                let values: Vec<Value> = s
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| serde_json::from_str(line))
                    .collect::<Result<Vec<Value>, _>>()
                    .with_context(|| ERROR_DECODE_FAILED)?;
                
                // Convert the Vec<Value> into a single Value::Array
                let jsonl_array = Value::Array(values);
                
                // Attempt to deserialize the Value::Array into the target type T
                serde_json::from_value(jsonl_array)
                    .with_context(|| format!("Failed to deserialize JSONL array into target type: {}", std::any::type_name::<T>()))
            },
        }
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
        match format {
            Format::Json => serde_json::to_vec(value)
                .with_context(|| ERROR_ENCODE_FAILED),
            
            Format::Bson => {
                let doc = to_document(value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                let mut buf = Vec::new();
                doc.to_writer(&mut buf)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(buf)
            },
            
            Format::Cbor => {
                let mut buf = Vec::new();
                ciborium::ser::into_writer(value, &mut buf)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(buf)
            },
            
            Format::Xml => {
                let mut buf = String::new();
                quick_xml::se::to_writer(&mut buf, value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(buf.into_bytes())
            },

            Format::Yaml => {
                let s = serde_yaml::to_string(value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(s.into_bytes())
            },

            Format::Toml => {
                let s = toml::to_string(value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(s.into_bytes())
            },
            
            Format::Jsonl => {
                // For single object, encode as JSON with a newline
                let mut json = serde_json::to_vec(value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                json.push(b'\n');
                Ok(json)
            },
        }
    }

    pub fn encode_to_string<T>(value: &T, format: Format) -> Result<String>
    where
        T: Serialize,
    {
        match format {
            Format::Json => serde_json::to_string(value)
                .with_context(|| ERROR_ENCODE_FAILED),
            
            Format::Xml => {
                let mut buf = String::new();
                quick_xml::se::to_writer(&mut buf, value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(buf)
            },

            Format::Yaml => {
                serde_yaml::to_string(value)
                    .with_context(|| ERROR_ENCODE_FAILED)
            },

            Format::Toml => {
                toml::to_string(value)
                    .with_context(|| ERROR_ENCODE_FAILED)
            },
            
            _ => {
                let bytes = Self::encode(value, format)?;
                String::from_utf8(bytes)
                    .with_context(|| ERROR_ENCODE_FAILED)
            }
        }
    }
    
    pub fn encode_jsonl_all<T>(values: &[T]) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        let mut result = Vec::new();
        
        for value in values {
            let mut json = serde_json::to_vec(value)
                .with_context(|| ERROR_ENCODE_FAILED)?;
            json.push(b'\n');
            result.extend_from_slice(&json);
        }
        
        Ok(result)
    }
}

pub struct Query {
    filter_str: String,
}

impl Query {
    pub fn compile(query_str: &str) -> Result<Self> {
        // Basic validation - should start with .
        if !query_str.starts_with('.') {
            return Err(anyhow!("{}: Query must start with '.'", ERROR_QUERY_PARSE));
        }
        
        Ok(Self {
            filter_str: query_str.to_string(),
        })
    }
    
    pub fn execute<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .with_context(|| ERROR_VALUE_CONVERSION)?;
            
        // Very simple implementation that just supports basic field access
        // For example: ".name" or ".address.street"
        let path = self.filter_str.trim_start_matches('.');
        let parts: Vec<&str> = path.split('.').collect();
        
        let mut current = &json_value;
        
        for part in parts {
            if let Value::Object(map) = current {
                match map.get(part) {
                    Some(val) => current = val,
                    None => return Err(anyhow!("Field '{}' not found in object", part)),
                }
            } else if let Value::Array(arr) = current {
                if let Ok(index) = part.parse::<usize>() {
                    if index < arr.len() {
                        current = &arr[index];
                    } else {
                        return Err(anyhow!("Array index {} out of bounds", index));
                    }
                } else {
                    return Err(anyhow!("Invalid array index: {}", part));
                }
            } else {
                return Err(anyhow!("Cannot access field '{}' on non-object value", part));
            }
        }
        
        Ok(serde_json::to_string(current)
            .map_err(|e| anyhow!("{}: {}", ERROR_VALUE_CONVERSION, e))?)
    }
}

pub struct Converter;

impl Converter {
    pub fn convert<T, U>(input: &[u8], from_format: Format, to_format: Format) -> Result<Vec<u8>>
    where
        T: DeserializeOwned + Serialize,
        U: Serialize,
    {
        let decoded: T = Decoder::decode(input, from_format)?;
        let encoded = Encoder::encode(&decoded, to_format)?;
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
        
        let query = Query::compile(".value").unwrap();
        let result = query.execute(&data).unwrap();
        
        // Parse the result back to get the number
        let value: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(value.as_i64(), Some(42));
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

pub mod examples {
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
        let query = Query::compile(".age")?;
        let age = query.execute(&person)?;
        // NOTE: Commented out due to API changes in jaq-interpret
        // println!("Age: {}", age.as_u64().unwrap());
        println!("Age query: {}", age);
        
        Ok(())
    }
} 