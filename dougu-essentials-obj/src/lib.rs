use anyhow::{anyhow, Context, Result};
use bson::{Document, from_document, to_document};
use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
use jaq_parse::parse;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::io::Cursor;
use serde_yaml;
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
            
            Format::Jsonl => {
                let json = serde_json::to_string(value)
                    .with_context(|| ERROR_ENCODE_FAILED)?;
                Ok(format!("{}\n", json))
            },
            
            _ => {
                let bytes = Self::encode(value, format)?;
                Ok(hex::encode(bytes))
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
        let (_parsed, errors) = parse(query_str, jaq_parse::main());
        if !errors.is_empty() || _parsed.is_none() {
            return Err(anyhow!(ERROR_QUERY_PARSE));
        }
        Ok(Query { filter_str: query_str.to_string() })
    }

    pub fn execute<T>(&self, value: &T) -> Result<Val>
    where
        T: Serialize,
    {
        // Convert Rust value to JSON and Val
        let json_val = serde_json::to_value(value)
            .with_context(|| ERROR_VALUE_CONVERSION)?;
        let input = Val::from(json_val);

        // Set up execution input iterator and context
        let inputs = RcIter::new(std::iter::once(Ok(input.clone())));
        let ctx = Ctx::new(vec![], &inputs);

        // Parse query again for execution
        let (parsed, errors) = parse(&self.filter_str, jaq_parse::main());
        if !errors.is_empty() || parsed.is_none() {
            return Err(anyhow!(ERROR_QUERY_EXECUTION));
        }

        // Compile parsed filter into executable filter
        let mut defs = ParseCtx::new(Vec::new());
        let filter = defs.compile(parsed.unwrap());

        // Execute filter with (ctx, input) tuple
        let mut results = filter.run((ctx, input));

        // Return first result or error
        let first = results
            .next()
            .ok_or_else(|| anyhow!(ERROR_QUERY_EXECUTION))?;
        first.map_err(|e| anyhow!("{}", e))
    }
}

pub struct Converter;

impl Converter {
    pub fn convert<T, U>(input: &[u8], from_format: Format, to_format: Format) -> Result<Vec<u8>>
    where
        T: DeserializeOwned + Serialize,
        U: Serialize,
    {
        let value: T = Decoder::decode(input, from_format)?;
        Encoder::encode(&value, to_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
        
        let json = Encoder::encode(&data, Format::Json).unwrap();
        let decoded: TestData = Decoder::decode(&json, Format::Json).unwrap();
        
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_yaml_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let yaml = Encoder::encode(&data, Format::Yaml).unwrap();
        let decoded: TestData = Decoder::decode(&yaml, Format::Yaml).unwrap();
        
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_toml_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let toml = Encoder::encode(&data, Format::Toml).unwrap();
        let decoded: TestData = Decoder::decode(&toml, Format::Toml).unwrap();
        
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_query() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let query = Query::compile(".name").unwrap();
        let result = query.execute(&data).unwrap();
        
        assert_eq!(result.to_string(), "\"test\"");
    }

    #[test]
    fn test_jsonl_roundtrip() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        
        let jsonl = Encoder::encode_jsonl_all(&[data.clone()]).unwrap();
        let decoded: Vec<TestData> = Decoder::decode(&jsonl, Format::Jsonl).unwrap();
        
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], data);
    }
    
    #[test]
    fn test_jsonl_multiple() {
        let data = vec![
            TestData { name: "test1".to_string(), value: 1 },
            TestData { name: "test2".to_string(), value: 2 },
        ];
        
        let jsonl = Encoder::encode_jsonl_all(&data).unwrap();
        let decoded: Vec<TestData> = Decoder::decode(&jsonl, Format::Jsonl).unwrap();
        
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].name, "test1");
        assert_eq!(decoded[0].value, 1);
        assert_eq!(decoded[1].name, "test2");
        assert_eq!(decoded[1].value, 2);
    }
}

// Add an example to demonstrate functionality
#[cfg(feature = "examples")]
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
        // Create test data
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            hobbies: vec!["reading".to_string(), "coding".to_string()],
        };

        // Convert to different formats
        let json = Encoder::encode(&person, Format::Json)?;
        let bson = Encoder::encode(&person, Format::Bson)?;
        let cbor = Encoder::encode(&person, Format::Cbor)?;
        let xml = Encoder::encode(&person, Format::Xml)?;
        let yaml = Encoder::encode(&person, Format::Yaml)?;
        let toml = Encoder::encode(&person, Format::Toml)?;
        let jsonl = Encoder::encode(&person, Format::Jsonl)?;

        println!("JSON: {}", String::from_utf8_lossy(&json));
        println!("BSON (hex): {}", hex::encode(&bson));
        println!("CBOR (hex): {}", hex::encode(&cbor));
        println!("XML: {}", String::from_utf8_lossy(&xml));
        println!("YAML: {}", String::from_utf8_lossy(&yaml));
        println!("TOML: {}", String::from_utf8_lossy(&toml));
        println!("JSONL: {}", String::from_utf8_lossy(&jsonl));

        // JSONL multiple items example
        let people = vec![
            Person {
                name: "Alice".to_string(),
                age: 30,
                hobbies: vec!["reading".to_string(), "coding".to_string()],
            },
            Person {
                name: "Bob".to_string(),
                age: 25,
                hobbies: vec!["gaming".to_string(), "hiking".to_string()],
            },
        ];

        let jsonl_multi = Encoder::encode_jsonl_all(&people)?;
        println!("JSONL multiple items:\n{}", String::from_utf8_lossy(&jsonl_multi));

        // Query the data
        let query = Query::compile(".hobbies[0]")?;
        let result = query.execute(&person)?;
        println!("Query result (.hobbies[0]): {}", result);

        // Convert between formats
        let xml_from_json = Converter::convert::<serde_json::Value, serde_json::Value>(
            &json, Format::Json, Format::Xml)?;
        println!("XML converted from JSON: {}", String::from_utf8_lossy(&xml_from_json));

        let yaml_from_json = Converter::convert::<serde_json::Value, serde_json::Value>(
            &json, Format::Json, Format::Yaml)?;
        println!("YAML converted from JSON: {}", String::from_utf8_lossy(&yaml_from_json));

        let toml_from_yaml = Converter::convert::<serde_json::Value, serde_json::Value>(
            &yaml, Format::Yaml, Format::Toml)?;
        println!("TOML converted from YAML: {}", String::from_utf8_lossy(&toml_from_yaml));

        Ok(())
    }
} 