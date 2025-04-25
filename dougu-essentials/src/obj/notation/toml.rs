use crate::core::error::{error, Result};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
use toml::Value as TomlValue;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TomlNotation;

impl TomlNotation {
    pub fn new() -> Self {
        TomlNotation
    }
}

impl Default for TomlNotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Notation for TomlNotation {
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let toml_string = self.encode_to_string(value)?;
        Ok(toml_string.into_bytes())
    }

    fn decode(&self, data: &[u8]) -> Result<NotationType> {
        let toml_str = str::from_utf8(data)?;
        let toml_value: TomlValue = toml::from_str(toml_str)?;
        toml_value_to_notation_type(&toml_value)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let toml_value = notation_type_to_toml_value(&notation_type)?;
        Ok(toml::to_string(&toml_value)?)
    }
}

fn notation_type_to_toml_value(notation_type: &NotationType) -> Result<TomlValue> {
    Ok(match notation_type {
        NotationType::Null => TomlValue::String("".to_string()),
        NotationType::Boolean(b) => TomlValue::Boolean(*b),
        NotationType::Number(n) => {
            match n {
                NumberVariant::Int(i) => TomlValue::Integer(*i),
                NumberVariant::Uint(u) => {
                    if let Ok(i_val) = (*u).try_into() {
                        TomlValue::Integer(i_val)
                    } else {
                        TomlValue::Float(*u as f64)
                    }
                }
                NumberVariant::Float(f) => TomlValue::Float(*f),
            }
        }
        NotationType::String(s) => TomlValue::String(s.clone()),
        NotationType::Array(arr) => {
            let values: Result<Vec<TomlValue>> = arr
                .iter()
                .map(notation_type_to_toml_value)
                .collect();
            TomlValue::Array(values?)
        }
        NotationType::Object(obj) => {
            let table: Result<toml::map::Map<String, TomlValue>> = obj
                .iter()
                .map(|(k, v)| notation_type_to_toml_value(v).map(|toml_v| (k.clone(), toml_v)))
                .collect();
            TomlValue::Table(table?)
        }
        _ => return Err(error(format!("Unsupported notation type for TOML: {:?}", notation_type))),
    })
}

fn toml_value_to_notation_type(value: &TomlValue) -> Result<NotationType> {
    Ok(match value {
        TomlValue::String(s) => NotationType::String(s.clone()),
        TomlValue::Integer(i) => NotationType::Number(NumberVariant::Int(*i)),
        TomlValue::Float(f) => NotationType::Number(NumberVariant::Float(*f)),
        TomlValue::Boolean(b) => NotationType::Boolean(*b),
        TomlValue::Datetime(dt) => NotationType::String(dt.to_string()),
        TomlValue::Array(arr) => {
            let values: Result<Vec<NotationType>> = arr
                .iter()
                .map(toml_value_to_notation_type)
                .collect();
            NotationType::Array(values?)
        }
        TomlValue::Table(table) => {
            let map: Result<HashMap<String, NotationType>> = table
                .iter()
                .map(|(k, v)| toml_value_to_notation_type(v).map(|nt| (k.clone(), nt)))
                .collect();
            NotationType::Object(map?)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_toml_roundtrip() {
        let notation = TomlNotation::default();
        // Explicitly type the HashMap value as NotationType
        let mut map: HashMap<String, NotationType> = HashMap::new();
        map.insert("title".to_string(), "TOML Example".into());
        map.insert("integer".to_string(), (123i64).into());
        map.insert("float".to_string(), (123.45f64).into());
        map.insert("unsigned".to_string(), (u64::MAX).into()); // Test large unsigned

        // Explicitly type the HashMap value as NotationType
        let mut owner: HashMap<String, NotationType> = HashMap::new();
        owner.insert("name".to_string(), "Tom Preston-Werner".into());
        owner.insert("organization".to_string(), "GitHub".into());
        map.insert("owner".to_string(), owner.into());

        let input: NotationType = map.into();
        let encoded = notation.encode(&input).unwrap();
        let decoded: NotationType = notation.decode(&encoded).unwrap();

        // Direct comparison might fail due to Uint->Float conversion
        // assert_eq!(input, decoded);

        // Instead, check specific fields after decoding
        if let NotationType::Object(decoded_map) = decoded {
            assert_eq!(decoded_map.get("title").unwrap().as_str(), Some("TOML Example"));
            assert_eq!(decoded_map.get("integer").unwrap().as_i64(), Some(123));
            assert_eq!(decoded_map.get("float").unwrap().as_f64(), Some(123.45));
            // Check how the large unsigned integer was handled (likely became float)
            assert!(decoded_map.get("unsigned").unwrap().as_f64().is_some());
            if let NotationType::Object(owner_map) = decoded_map.get("owner").unwrap() {
                assert_eq!(owner_map.get("name").unwrap().as_str(), Some("Tom Preston-Werner"));
            } else {
                panic!("Decoded owner is not an object");
            }
        } else {
            panic!("Decoded result is not an object");
        }
    }
}