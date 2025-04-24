use anyhow::{Context, Result};
use ciborium::value::Value as CborValue;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct CborNotation;

impl Notation for CborNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let value = ciborium::de::from_reader::<CborValue, _>(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        let notation = Self::cbor_to_notation(&value)?;
        Ok(T::from(notation))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let cbor = Self::notation_to_cbor(&value)?;
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&cbor, &mut buf)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf)
    }
}

impl CborNotation {
    fn cbor_to_notation(cbor: &CborValue) -> Result<NotationType> {
        match cbor {
            CborValue::Integer(n) => Ok(NotationType::Number(*n as f64)),
            CborValue::Bytes(b) => Ok(NotationType::String(hex::encode(b))),
            CborValue::Text(s) => Ok(NotationType::String(s.clone())),
            CborValue::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::cbor_to_notation(item)?);
                }
                Ok(NotationType::Array(vec))
            },
            CborValue::Map(map) => {
                let mut vec = Vec::new();
                for (k, v) in map {
                    let key = match k {
                        CborValue::Text(s) => s.clone(),
                        CborValue::Integer(n) => n.to_string(),
                        _ => return Err(anyhow!("{}: Invalid CBOR map key", ERROR_DECODE_FAILED)),
                    };
                    vec.push((key, Self::cbor_to_notation(v)?));
                }
                Ok(NotationType::Object(vec))
            },
            CborValue::Tag(_, v) => Self::cbor_to_notation(v),
            CborValue::Simple(s) => match s {
                20 => Ok(NotationType::Boolean(false)),
                21 => Ok(NotationType::Boolean(true)),
                22 => Ok(NotationType::Null),
                _ => Ok(NotationType::Number(*s as f64)),
            },
            CborValue::Float(n) => Ok(NotationType::Number(*n)),
            CborValue::Null => Ok(NotationType::Null),
        }
    }
    
    fn notation_to_cbor(value: &NotationType) -> Result<CborValue> {
        match value {
            NotationType::String(s) => Ok(CborValue::Text(s.clone())),
            NotationType::Number(n) => Ok(CborValue::Float(*n)),
            NotationType::Boolean(b) => Ok(CborValue::Simple(if *b { 21 } else { 20 })),
            NotationType::Null => Ok(CborValue::Null),
            NotationType::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::notation_to_cbor(item)?);
                }
                Ok(CborValue::Array(vec))
            },
            NotationType::Object(obj) => {
                let mut map = Vec::new();
                for (k, v) in obj {
                    map.push((CborValue::Text(k.clone()), Self::notation_to_cbor(v)?));
                }
                Ok(CborValue::Map(map))
            },
        }
    }
} 