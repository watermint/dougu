use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use super::{Notation, NotationType, NumberVariant};
use super::{
    json::JsonNotation,
    yaml::YamlNotation,
    toml::TomlNotation,
    xml::XmlNotation,
    bson::BsonNotation,
    cbor::CborNotation,
    jsonl::JsonlNotation,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
struct TestData {
    name: String,
    value: f64,
    is_active: bool,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl TestData {
    fn from_notation(notation: NotationType) -> Result<Self> {
        if let NotationType::Object(map) = notation {
            let mut data = TestData {
                name: String::new(),
                value: 0.0,
                is_active: false,
                tags: Vec::new(),
                metadata: HashMap::new(),
            };
            for (k, v) in map {
                match (k.as_str(), v) {
                    ("name", NotationType::String(s)) => data.name = s,
                    ("value", NotationType::Number(n)) => {
                        data.value = match n {
                            NumberVariant::Int(i) => i as f64,
                            NumberVariant::Uint(u) => u as f64,
                            NumberVariant::Float(f) => f,
                        };
                    }
                    ("is_active", NotationType::Boolean(b)) => data.is_active = b,
                    ("tags", NotationType::Array(arr)) => {
                        data.tags = arr.into_iter().filter_map(|t| {
                            if let NotationType::String(s) = t { Some(s) } else { None }
                        }).collect();
                    }
                    ("metadata", NotationType::Object(meta_obj)) => {
                        data.metadata = meta_obj.into_iter().filter_map(|(k, v)| {
                            if let NotationType::String(s) = v { Some((k, s)) } else { None }
                        }).collect();
                    },
                    _ => (),
                }
            }
            Ok(data)
        } else {
            Err(anyhow!("Expected an object"))
        }
    }

    fn to_notation(&self) -> NotationType {
        let mut obj = vec![
            ("name".to_string(), NotationType::String(self.name.clone())),
            ("value".to_string(), NotationType::Number(NumberVariant::Float(self.value))),
            ("is_active".to_string(), NotationType::Boolean(self.is_active)),
            ("tags".to_string(), NotationType::Array(self.tags.iter().map(|s| NotationType::String(s.clone())).collect())),
            ("metadata".to_string(), NotationType::Object(self.metadata.iter().map(|(k, v)| (k.clone(), NotationType::String(v.clone()))).collect())),
        ];
        obj.into()
    }
}

impl From<&TestData> for NotationType {
    fn from(self_val: &TestData) -> NotationType {
        let mut obj = Vec::new();
        obj.push(("name".to_string(), NotationType::String(self_val.name.clone())));
        obj.push(("value".to_string(), NotationType::Number(NumberVariant::Float(self_val.value))));
        obj.push(("is_active".to_string(), NotationType::Boolean(self_val.is_active)));
        obj.push(("tags".to_string(), NotationType::Array(
            self_val.tags.iter()
                .map(|s| NotationType::String(s.clone()))
                .collect()
        )));
        obj.push(("metadata".to_string(), NotationType::Object(
            self_val.metadata.iter()
                .map(|(k, v)| (k.clone(), NotationType::String(v.clone())))
                .collect()
        )));
        obj.into()
    }
}

impl Default for TestData {
    fn default() -> Self {
        TestData {
            name: String::new(),
            value: 0.0,
            is_active: false,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl From<NotationType> for TestData {
    fn from(value: NotationType) -> Self {
        let mut data = TestData::default();
        if let NotationType::Object(obj) = value {
            for (key, value) in obj {
                match (key.as_str(), value) {
                    ("name", NotationType::String(s)) => data.name = s,
                    ("value", NotationType::Number(n)) => {
                        data.value = match n {
                            NumberVariant::Int(i) => i as f64,
                            NumberVariant::Uint(u) => u as f64,
                            NumberVariant::Float(f) => f,
                        };
                    },
                    ("is_active", NotationType::Boolean(b)) => data.is_active = b,
                    ("tags", NotationType::Array(arr)) => {
                        data.tags = arr.into_iter().filter_map(|v| {
                            if let NotationType::String(s) = v { Some(s) } else { None }
                        }).collect();
                    },
                    ("metadata", NotationType::Object(meta_obj)) => {
                        data.metadata = meta_obj.into_iter().filter_map(|(k, v)| {
                            if let NotationType::String(s) = v { Some((k, s)) } else { None }
                        }).collect();
                    },
                    _ => {}
                }
            }
        }
        data
    }
}

impl From<TestData> for NotationType {
    fn from(data: TestData) -> Self {
        let mut obj = Vec::new();
        obj.push(("name".to_string(), NotationType::String(data.name.clone())));
        obj.push(("value".to_string(), NotationType::Number(NumberVariant::Float(data.value))));
        obj.push(("is_active".to_string(), NotationType::Boolean(data.is_active)));
        obj.push(("tags".to_string(), NotationType::Array(
            data.tags.into_iter()
                .map(NotationType::String)
                .collect()
        )));
        obj.push(("metadata".to_string(), NotationType::Object(
            data.metadata.into_iter()
                .map(|(k, v)| (k, NotationType::String(v)))
                .collect()
        )));
        obj.into()
    }
}

// Add From<NotationType> for Vec<TestData>
impl From<NotationType> for Vec<TestData> {
    fn from(value: NotationType) -> Self {
        if let NotationType::Array(arr) = value {
            arr.into_iter().map(TestData::from).collect()
        } else {
            // If a single object was decoded by JsonlNotation, wrap it
            // This might happen if decode was called on a single line
            let single_item = TestData::from(value);
            vec![single_item]
            // Alternatively, return an empty vec or handle error:
            // vec![] 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_data() -> TestData {
        TestData {
            name: "test".to_string(),
            value: 42.0,
            is_active: true,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            metadata: {
                let mut map = HashMap::new();
                map.insert("created".to_string(), "2024-01-01".to_string());
                map.insert("version".to_string(), "1.0".to_string());
                map
            },
        }
    }
    
    #[test]
    fn test_json_notation() -> Result<()> {
        let notation = JsonNotation::new();
        let data = create_test_data();
        
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_yaml_notation() -> Result<()> {
        let notation = YamlNotation::new();
        let data = create_test_data();
        
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_toml_notation() -> Result<()> {
        let notation = TomlNotation::new();
        let data = create_test_data();
        
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("name = \"test\""));
        assert!(encoded_str.contains(&format!("value = {}", data.value)));
        
        Ok(())
    }
    
    #[test]
    fn test_cbor_notation() -> Result<()> {
        let notation = CborNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_bson_notation() -> Result<()> {
        let notation = BsonNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        Ok(())
    }
    
    #[test]
    fn test_xml_notation() -> Result<()> {
        let notation = XmlNotation;
        let data = create_test_data();
        
        // Test encode/decode round trip
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded = TestData::from(decoded_notation);
        assert_eq!(data, decoded);
        
        // Test string encoding
        let encoded_str = notation.encode_to_string(&data)?;
        assert!(encoded_str.contains("<name>test</name>"));
        assert!(encoded_str.contains(&format!("<value>{}</value>", data.value)));
        
        Ok(())
    }
    
    #[test]
    fn test_jsonl_notation() -> Result<()> {
        let notation = JsonlNotation::default();
        
        // Test single item encoding/decoding
        let data = create_test_data();
        let encoded = notation.encode(&data)?;
        let decoded_notation: NotationType = notation.decode(&encoded)?;
        let decoded_vec = Vec::<TestData>::from(decoded_notation);
        assert_eq!(1, decoded_vec.len());
        assert_eq!(data, decoded_vec[0]);
        
        // Test collection encoding
        let data_vec = vec![
            create_test_data(),
            TestData {
                name: "test2".to_string(),
                value: 84.0,
                is_active: false,
                tags: vec!["tag3".to_string()],
                metadata: HashMap::new(),
            }
        ];
        
        let encoded_collection = notation.encode_collection(&data_vec)?;
        let decoded_collection_notation: NotationType = notation.decode(&encoded_collection)?;
        let decoded_collection = Vec::<TestData>::from(decoded_collection_notation);
        assert_eq!(data_vec.len(), decoded_collection.len());
        assert_eq!(data_vec[0], decoded_collection[0]);
        assert_eq!(data_vec[1], decoded_collection[1]);
        
        // Verify the encoded string contains newlines
        let encoded_str = String::from_utf8_lossy(&encoded_collection);
        assert!(encoded_str.contains('\n'));
        
        Ok(())
    }
    
    #[test]
    fn test_notation_types() -> Result<()> {
        let data = create_test_data();
        
        // Test each notation type
        for notation_type_instance in [
            NotationType::Json(JsonNotation),
            NotationType::Yaml(YamlNotation),
            NotationType::Toml(TomlNotation),
            NotationType::Xml(XmlNotation),
            NotationType::Bson(BsonNotation),
            NotationType::Cbor(CborNotation),
        ] {
            let encoded = notation_type_instance.encode(&data)?;
            let decoded_notation: NotationType = notation_type_instance.decode(&encoded)?;
            let decoded = TestData::from(decoded_notation);
            assert_eq!(data, decoded, "Round-trip failed for notation type: {:?}", notation_type_instance);
        }
        
        // Test JSONL separately
        let jsonl_type = NotationType::Jsonl(JsonlNotation::default());
        let encoded = jsonl_type.encode(&data)?;
        let decoded_notation: NotationType = jsonl_type.decode(&encoded)?;
        let decoded_vec = Vec::<TestData>::from(decoded_notation);
        assert_eq!(1, decoded_vec.len());
        assert_eq!(data, decoded_vec[0], "Round-trip failed for JSONL notation");
        
        Ok(())
    }
    
    #[test]
    fn test_format_conversion() -> Result<()> {
        let data = create_test_data();
        
        let json_notation = JsonNotation;
        let yaml_notation = YamlNotation;
        
        let json_encoded = json_notation.encode(&data)?;
        let json_decoded_notation: NotationType = json_notation.decode(&json_encoded)?;
        let json_decoded = TestData::from(json_decoded_notation);
        
        let yaml_encoded = yaml_notation.encode(&json_decoded)?;
        let yaml_decoded_notation: NotationType = yaml_notation.decode(&yaml_encoded)?;
        let yaml_decoded = TestData::from(yaml_decoded_notation);
        
        assert_eq!(data, yaml_decoded);
        
        Ok(())
    }

    #[test]
    fn test_notation_type_conversion() -> Result<()> {
        let data = create_test_data();
        
        let notation_type: NotationType = data.clone().into();
        let decoded: TestData = notation_type.into();
        assert_eq!(data, decoded);
        
        Ok(())
    }
} 