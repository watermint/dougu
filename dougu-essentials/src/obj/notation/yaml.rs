use anyhow::{anyhow, Result};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use serde::{Deserialize, Serialize};
use serde_yaml::{from_slice, to_string, Value as YamlValue, Number as YamlNumber};
use std::collections::HashMap;
use num_bigint::BigInt;
use bigdecimal::BigDecimal;
use num_traits::ToPrimitive;

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
            // Try to determine the best NotationType variant
            if n.is_i64() {
                NotationType::Number(NumberVariant::Int(n.as_i64().unwrap()))
            } else if n.is_u64() {
                NotationType::Number(NumberVariant::Uint(n.as_u64().unwrap()))
            } else if n.is_f64() {
                NotationType::Number(NumberVariant::Float(n.as_f64().unwrap()))
            } else {
                // Fallback or error if number type is unusual
                // For now, try converting to f64 as a fallback
                NotationType::Number(NumberVariant::Float(
                    n.as_f64().ok_or_else(|| anyhow!("Unsupported YAML number: {}", n))?
                ))
            }
        }
        YamlValue::String(s) => NotationType::String(s.clone()),
        YamlValue::Sequence(seq) => {
            let values: Result<Vec<NotationType>> = seq.iter().map(yaml_value_to_notation_type).collect();
            NotationType::Array(values?)
        }
        YamlValue::Mapping(map) => {
            let obj_map: Result<HashMap<String, NotationType>> = map
                .iter()
                .map(|(key_yaml, value_yaml)| {
                     let key_str = match key_yaml {
                        YamlValue::String(s) => Ok(s.clone()),
                        YamlValue::Number(n) => Ok(n.to_string()),
                        YamlValue::Bool(b) => Ok(b.to_string()),
                        _ => Err(anyhow!("Unsupported YAML map key type: {:?}", key_yaml)),
                    }?;
                    yaml_value_to_notation_type(value_yaml).map(|nt| (key_str, nt))
                })
                .collect();
            NotationType::Object(obj_map?)
        }
        YamlValue::Tagged(tagged_value) => {
            yaml_value_to_notation_type(&tagged_value.value)?
        }
    })
}

fn notation_type_to_yaml_value(notation_type: &NotationType) -> Result<YamlValue> {
    Ok(match notation_type {
        NotationType::Null => YamlValue::Null,
        NotationType::Boolean(b) => YamlValue::Bool(*b),
        NotationType::Number(n) => {
            // Create the appropriate serde_yaml::Number
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
    use num_bigint::BigInt;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    // Needed for NumberVariant tests
    use crate::obj::notation::NumberVariant;
    use std::collections::HashMap;

    #[test]
    fn test_yaml_roundtrip() -> Result<()> {
        let notation = YamlNotation::new();
        // Explicitly type the HashMap value as NotationType
        let mut map: HashMap<String, NotationType> = HashMap::new();
        map.insert("name".to_string(), "Test".into());
        map.insert("age_int".to_string(), (42i64).into());
        map.insert("count_uint".to_string(), (100u64).into());
        map.insert("price_float".to_string(), (99.99f64).into());
        map.insert("is_active".to_string(), true.into());
        // Explicitly type the inner Vec items as NotationType
        map.insert("tags".to_string(), NotationType::Array(vec![
            NotationType::String("rust".to_string()), 
            NotationType::String("yaml".to_string())
        ]));
        
        // Explicitly type the HashMap value as NotationType
        let mut metadata_map: HashMap<String, NotationType> = HashMap::new();
        metadata_map.insert("created".to_string(), "2024-01-01".into());
        metadata_map.insert("version".to_string(), (1.0f64).into());
        map.insert("metadata".to_string(), metadata_map.into());

        let input_notation: NotationType = map.into();

        let encoded_string = notation.encode_to_string(&input_notation)?;
        println!("Encoded YAML:\n{}", encoded_string); // Print for debugging
        let decoded = notation.decode(encoded_string.as_bytes())?;
        
        // Direct comparison should work if types are preserved
        assert_eq!(input_notation, decoded);

        // Verify specific types after decoding
        if let NotationType::Object(decoded_map) = decoded {
            assert_eq!(decoded_map.get("age_int").unwrap().as_i64(), Some(42));
            assert_eq!(decoded_map.get("count_uint").unwrap().as_u64(), Some(100));
            assert_eq!(decoded_map.get("price_float").unwrap().as_f64(), Some(99.99));
        } else {
             panic!("Decoded result is not an object");
        }

        Ok(())
    }
} 