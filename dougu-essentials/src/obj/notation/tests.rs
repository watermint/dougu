use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::obj::{Format, notation, Notation};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    name: String,
    value: i32,
    tags: Vec<String>,
    nested: Option<Box<NestedData>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NestedData {
    id: u64,
    active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_data() -> TestData {
        TestData {
            name: "test".to_string(),
            value: 42,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            nested: Some(Box::new(NestedData {
                id: 123,
                active: true,
            })),
        }
    }
    
    #[test]
    fn test_json_notation() -> Result<()> {
        let notation = notation::json::JsonNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("\"name\":\"test\""));
        assert!(encoded_str.contains("\"value\":42"));
        
        Ok(())
    }
    
    #[test]
    fn test_yaml_notation() -> Result<()> {
        let notation = notation::yaml::YamlNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("name: test"));
        assert!(encoded_str.contains("value: 42"));
        
        Ok(())
    }
    
    #[test]
    fn test_toml_notation() -> Result<()> {
        let notation = notation::toml::TomlNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("name = \"test\""));
        assert!(encoded_str.contains("value = 42"));
        
        Ok(())
    }
    
    #[test]
    fn test_cbor_notation() -> Result<()> {
        let notation = notation::cbor::CborNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_bson_notation() -> Result<()> {
        let notation = notation::bson::BsonNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_xml_notation() -> Result<()> {
        let notation = notation::xml::XmlNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded: TestData = notation.decode(&encoded)?;
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("<name>test</name>"));
        assert!(encoded_str.contains("<value>42</value>"));
        
        Ok(())
    }
    
    #[test]
    fn test_jsonl_notation() -> Result<()> {
        let notation = notation::jsonl::JsonlNotation;
        
        // Test single item encoding/decoding
        let data = create_test_data();
        let encoded = notation.encode(&data)?;
        let decoded_vec: Vec<TestData> = notation.decode(&encoded)?;
        assert_eq!(1, decoded_vec.len());
        assert_eq!(data, decoded_vec[0]);
        
        // Test collection encoding
        let data_vec = vec![
            create_test_data(),
            TestData {
                name: "test2".to_string(),
                value: 84,
                tags: vec!["tag3".to_string()],
                nested: None,
            }
        ];
        
        let encoded_collection = notation.encode_collection(&data_vec)?;
        let decoded_collection: Vec<TestData> = notation.decode(&encoded_collection)?;
        assert_eq!(data_vec.len(), decoded_collection.len());
        assert_eq!(data_vec[0], decoded_collection[0]);
        assert_eq!(data_vec[1], decoded_collection[1]);
        
        // Verify the encoded string contains newlines
        let encoded_str = String::from_utf8_lossy(&encoded_collection);
        assert!(encoded_str.contains('\n'));
        
        Ok(())
    }
    
    #[test]
    fn test_get_notation() -> Result<()> {
        let data = create_test_data();
        
        // Test each format using the get_notation function (excluding JSONL which needs special handling)
        for format in [
            Format::Json,
            Format::Yaml,
            Format::Toml,
            Format::Xml,
            Format::Bson,
            Format::Cbor,
        ] {
            let notation = notation::get_notation(format);
            let encoded = notation.encode(&data)?;
            let decoded: TestData = notation.decode(&encoded)?;
            assert_eq!(data, decoded, "Round-trip failed for format: {:?}", format);
        }
        
        // Test JSONL separately with proper array handling
        let jsonl_notation = notation::get_notation(Format::Jsonl);
        let encoded = jsonl_notation.encode(&data)?;
        let decoded_vec: Vec<TestData> = jsonl_notation.decode(&encoded)?;
        assert_eq!(1, decoded_vec.len());
        assert_eq!(data, decoded_vec[0], "Round-trip failed for JSONL format");
        
        Ok(())
    }
    
    #[test]
    fn test_format_conversion() -> Result<()> {
        let data = create_test_data();
        
        // Convert from JSON to YAML
        let json_notation = notation::json::JsonNotation;
        let yaml_notation = notation::yaml::YamlNotation;
        
        let json_encoded = json_notation.encode(&data)?;
        let json_decoded: TestData = json_notation.decode(&json_encoded)?;
        
        let yaml_encoded = yaml_notation.encode(&json_decoded)?;
        let yaml_decoded: TestData = yaml_notation.decode(&yaml_encoded)?;
        
        assert_eq!(data, yaml_decoded);
        
        Ok(())
    }
} 