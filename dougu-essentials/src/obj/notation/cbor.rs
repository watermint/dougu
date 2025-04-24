use anyhow::{anyhow, Result};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use ciborium::{from_reader, into_writer, value::Value as CborValue};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use std::convert::TryInto;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CborNotation;

impl Notation for CborNotation {
    fn encode<T>(&self, value: &T) -> anyhow::Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let cbor_value = notation_type_to_cbor_value(&notation_type)?;
        let mut buf = Vec::new();
        into_writer(&cbor_value, &mut buf)?;
        Ok(buf)
    }

    fn decode(&self, data: &[u8]) -> anyhow::Result<NotationType> {
        let cbor_value: CborValue = from_reader(data)?;
        cbor_value_to_notation_type(&cbor_value)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let bytes = self.encode(value)?;
        Ok(hex::encode(bytes))
    }
}

fn cbor_value_to_notation_type(value: &CborValue) -> Result<NotationType> {
    match value {
        CborValue::Null => Ok(NotationType::Null),
        CborValue::Bool(b) => Ok(NotationType::Boolean(*b)),
        CborValue::Integer(i) => {
            if let Ok(ival) = (*i).try_into() {
                 Ok(NotationType::Number(NumberVariant::Int(ival)))
            } else if let Ok(uval) = (*i).try_into() {
                 Ok(NotationType::Number(NumberVariant::Uint(uval)))
            } else {
                Err(anyhow!("CBOR integer {:?} out of range for i64/u64", i))
            }
        }
        CborValue::Float(f) => Ok(NotationType::Number(NumberVariant::Float(*f))),
        CborValue::Bytes(b) => Ok(NotationType::String(BASE64_STANDARD.encode(b))),
        CborValue::Text(s) => Ok(NotationType::String(s.clone())),
        CborValue::Array(arr) => {
            let values: Result<Vec<NotationType>> = arr.iter().map(cbor_value_to_notation_type).collect();
            Ok(NotationType::Array(values?))
        }
        CborValue::Map(map) => {
            let map_values: Result<HashMap<String, NotationType>> = map
                .iter()
                .map(|(k, v)| {
                    let key_str = match k {
                        CborValue::Text(s) => Ok(s.clone()),
                        _ => Err(anyhow!("Unsupported CBOR map key type: {:?}", k)),
                    }?;
                    let value_notation = cbor_value_to_notation_type(v)?;
                    Ok((key_str, value_notation))
                })
                .collect();
            Ok(NotationType::Object(map_values?))
        }
        _ => Err(anyhow!("Unsupported CBOR value type: {:?}", value)),
    }
}

fn notation_type_to_cbor_value(notation_type: &NotationType) -> Result<CborValue> {
    match notation_type {
        NotationType::Null => Ok(CborValue::Null),
        NotationType::Boolean(b) => Ok(CborValue::Bool(*b)),
        NotationType::Number(n) => {
            match n {
                NumberVariant::Int(i) => Ok(CborValue::Integer((*i).into())),
                NumberVariant::Uint(u) => Ok(CborValue::Integer((*u).into())),
                NumberVariant::Float(f) => {
                    if f.trunc() == *f && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                        Ok(CborValue::Integer((*f as i64).into()))
                    } else {
                        Ok(CborValue::Float(*f))
                    }
                },
            }
        }
        NotationType::String(s) => {
            if let Ok(bytes) = BASE64_STANDARD.decode(s) {
                Ok(CborValue::Bytes(bytes))
            } else {
                Ok(CborValue::Text(s.clone()))
            }
        }
        NotationType::Array(arr) => {
            let values: Result<Vec<CborValue>> = arr.iter().map(notation_type_to_cbor_value).collect();
            Ok(CborValue::Array(values?))
        }
        NotationType::Object(map) => {
            let cbor_pairs: Result<Vec<(CborValue, CborValue)>> = map
                .iter()
                .map(|(key, value)| {
                    Ok((
                        CborValue::Text(key.clone()),
                        notation_type_to_cbor_value(value)?,
                    ))
                })
                .collect();
            Ok(CborValue::Map(cbor_pairs?))
        }
        _ => Err(anyhow!("Unsupported notation type for CBOR: {:?}", notation_type)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Needed for NumberVariant tests
    use crate::obj::notation::NumberVariant;
    use std::collections::HashMap;

    #[test]
    fn test_cbor_roundtrip() -> Result<()> {
        let notation = CborNotation;
        // Explicitly type the HashMap value as NotationType
        let mut map: HashMap<String, NotationType> = HashMap::new();
        map.insert("int".to_string(), (123i64).into());
        map.insert("uint".to_string(), (u64::MAX).into());
        map.insert("float".to_string(), (123.45f64).into());
        map.insert("string".to_string(), "hello".into());
        map.insert("bool".to_string(), true.into());
        map.insert("null".to_string(), NotationType::Null);
        // Explicitly type the inner Vec items as NotationType
        map.insert("array".to_string(), NotationType::Array(vec![
            NotationType::Number(NumberVariant::Int(1)), 
            NotationType::String("two".to_string())
        ]));
        
        let input: NotationType = map.into();

        let encoded = notation.encode(&input)?;
        let decoded = notation.decode(&encoded)?;

        // Direct comparison should work if types are preserved
        assert_eq!(input, decoded);
        Ok(())
    }
} 