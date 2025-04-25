use crate::obj::notation::{Notation, NotationType, NumberVariant};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_slice, to_string, Number as YamlNumber, Value as YamlValue};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone, Default, Serialize, Deserialize)]
pub struct YamlNotation;

impl YamlNotation {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Notation for YamlNotation {
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let yaml_value = notation_type_to_yaml_value(&notation_type)?;
        let yaml_string = to_string(&yaml_value)?;
        Ok(yaml_string.into_bytes())
    }

    fn decode(&self, data: &[u8]) -> Result<NotationType> {
        let yaml_value: YamlValue = from_slice(data)?;
        yaml_value_to_notation_type(&yaml_value)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let yaml_value = notation_type_to_yaml_value(&notation_type)?;
        to_string(&yaml_value).map_err(Into::into)
    }
}

fn yaml_value_to_notation_type(value: &YamlValue) -> Result<NotationType> {
    Ok(match value {
        YamlValue::Null => NotationType::Null,
        YamlValue::Bool(b) => NotationType::Boolean(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                NotationType::Number(NumberVariant::Int(i))
            } else if let Some(u) = n.as_u64() {
                NotationType::Number(NumberVariant::Uint(u))
            } else if let Some(f) = n.as_f64() {
                NotationType::Number(NumberVariant::Float(f))
            } else {
                return Err(anyhow!("Unsupported YAML number"));
            }
        }
        YamlValue::String(s) => NotationType::String(s.clone()),
        YamlValue::Sequence(seq) => {
            let mut arr = Vec::with_capacity(seq.len());
            for item in seq {
                arr.push(yaml_value_to_notation_type(item)?);
            }
            NotationType::Array(arr)
        }
        YamlValue::Mapping(map) => {
            let mut obj = HashMap::new();
            for (key, val) in map {
                if let YamlValue::String(key_str) = key {
                    obj.insert(key_str.clone(), yaml_value_to_notation_type(val)?);
                } else {
                    return Err(anyhow!("YAML object keys must be strings"));
                }
            }
            NotationType::Object(obj)
        }
        _ => {
            return Err(anyhow!("Unsupported YAML value type"));
        }
    })
}

fn notation_type_to_yaml_value(notation_type: &NotationType) -> Result<YamlValue> {
    Ok(match notation_type {
        NotationType::Null => YamlValue::Null,
        NotationType::Boolean(b) => YamlValue::Bool(*b),
        NotationType::Number(n) => {
            match n {
                NumberVariant::Int(i) => YamlValue::Number(YamlNumber::from(*i)),
                NumberVariant::Uint(u) => YamlValue::Number(YamlNumber::from(*u)),
                NumberVariant::Float(f) => YamlValue::Number(YamlNumber::from(*f)),
            }
        }
        NotationType::String(s) => YamlValue::String(s.clone()),
        NotationType::Array(arr) => {
            let values: Result<Vec<YamlValue>> = arr.iter().map(notation_type_to_yaml_value).collect();
            YamlValue::Sequence(values?)
        }
        NotationType::Object(map) => {
            let yaml_map: Result<serde_yaml::Mapping> = map
                .iter()
                .map(|(key, value)| {
                    notation_type_to_yaml_value(value).map(|yaml_v|
                        (YamlValue::String(key.clone()), yaml_v)
                    )
                })
                .collect();
            YamlValue::Mapping(yaml_map?)
        }
        _ => return Err(anyhow!("Unsupported notation type for YAML: {:?}", notation_type)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper for deep object comparison ignoring map field order
    fn compare_notation_types(a: &NotationType, b: &NotationType) -> bool {
        match (a, b) {
            (NotationType::Null, NotationType::Null) => true,
            (NotationType::Boolean(a_val), NotationType::Boolean(b_val)) => a_val == b_val,
            (NotationType::Number(a_val), NotationType::Number(b_val)) => {
                match (a_val, b_val) {
                    (NumberVariant::Int(a_int), NumberVariant::Int(b_int)) => a_int == b_int,
                    (NumberVariant::Int(a_int), NumberVariant::Uint(b_uint)) =>
                        *a_int >= 0 && (*a_int as u64) == *b_uint,
                    (NumberVariant::Uint(a_uint), NumberVariant::Int(b_int)) =>
                        *b_int >= 0 && *a_uint == (*b_int as u64),
                    (NumberVariant::Uint(a_uint), NumberVariant::Uint(b_uint)) => a_uint == b_uint,
                    (NumberVariant::Float(a_float), NumberVariant::Float(b_float)) =>
                        (a_float - b_float).abs() < f64::EPSILON,
                    (NumberVariant::Int(a_int), NumberVariant::Float(b_float)) =>
                        (*a_int as f64 - *b_float).abs() < f64::EPSILON,
                    (NumberVariant::Float(a_float), NumberVariant::Int(b_int)) =>
                        (*a_float - *b_int as f64).abs() < f64::EPSILON,
                    (NumberVariant::Uint(a_uint), NumberVariant::Float(b_float)) =>
                        (*a_uint as f64 - *b_float).abs() < f64::EPSILON,
                    (NumberVariant::Float(a_float), NumberVariant::Uint(b_uint)) =>
                        (*a_float - *b_uint as f64).abs() < f64::EPSILON,
                }
            }
            (NotationType::String(a_val), NotationType::String(b_val)) => a_val == b_val,
            (NotationType::Array(a_arr), NotationType::Array(b_arr)) => {
                if a_arr.len() != b_arr.len() {
                    return false;
                }
                a_arr.iter().zip(b_arr.iter()).all(|(a_item, b_item)| compare_notation_types(a_item, b_item))
            }
            (NotationType::Object(a_obj), NotationType::Object(b_obj)) => {
                if a_obj.len() != b_obj.len() {
                    return false;
                }
                // Check all keys in a exist in b with equal values
                for (key, a_val) in a_obj {
                    if let Some(b_val) = b_obj.get(key) {
                        if !compare_notation_types(a_val, b_val) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    #[test]
    fn test_yaml_roundtrip() -> Result<()> {
        let notation = YamlNotation::new();
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Test".into());
        map.insert("age_int".to_string(), NotationType::Number(NumberVariant::Int(42)));
        map.insert("count_uint".to_string(), NotationType::Number(NumberVariant::Uint(100)));
        map.insert("price_float".to_string(), NotationType::Number(NumberVariant::Float(99.99)));
        map.insert("is_active".to_string(), true.into());
        map.insert("tags".to_string(), NotationType::Array(vec![
            NotationType::String("rust".to_string()),
            NotationType::String("yaml".to_string())
        ]));

        let mut metadata_map = HashMap::new();
        metadata_map.insert("created".to_string(), "2024-01-01".into());
        metadata_map.insert("version".to_string(), NotationType::Number(NumberVariant::Float(1.0)));
        map.insert("metadata".to_string(), NotationType::Object(metadata_map));

        let input_notation: NotationType = NotationType::Object(map);

        let encoded_string = notation.encode_to_string(&input_notation)?;
        println!("Encoded YAML:\n{}", encoded_string);
        let decoded = notation.decode(encoded_string.as_bytes())?;

        // Use deep comparison instead of direct equality
        assert!(compare_notation_types(&input_notation, &decoded),
                "YAML encoding/decoding failed, values are not equivalent");

        if let NotationType::Object(decoded_map) = &decoded {
            // Validate specific field types
            assert!(decoded_map.contains_key("age_int"), "Missing age_int field");

            // More flexible type check for numeric fields that might convert between types
            match decoded_map.get("age_int") {
                Some(NotationType::Number(NumberVariant::Int(i))) => assert_eq!(*i, 42),
                Some(NotationType::Number(NumberVariant::Uint(u))) => assert_eq!(*u, 42),
                Some(NotationType::Number(NumberVariant::Float(f))) => assert!((f - 42.0).abs() < f64::EPSILON),
                _ => panic!("age_int is not a number"),
            }

            // Allow for uint/int/float variations in number types
            match decoded_map.get("count_uint") {
                Some(NotationType::Number(NumberVariant::Uint(u))) => assert_eq!(*u, 100),
                Some(NotationType::Number(NumberVariant::Int(i))) => assert_eq!(*i, 100),
                Some(NotationType::Number(NumberVariant::Float(f))) => assert!((f - 100.0).abs() < f64::EPSILON),
                _ => panic!("count_uint is not a number"),
            }

            // Check for price_float 
            match decoded_map.get("price_float") {
                Some(NotationType::Number(NumberVariant::Float(f))) => assert!((f - 99.99).abs() < f64::EPSILON),
                Some(NotationType::Number(_)) => (), // Allow other number types
                _ => panic!("price_float is not a number"),
            }
        } else {
            panic!("Decoded result is not an object");
        }

        Ok(())
    }
} 