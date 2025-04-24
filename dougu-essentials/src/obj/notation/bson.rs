use anyhow::{Context, Result};
use bson::{Bson, Document};
use std::io::Cursor;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct BsonNotation;

impl Notation for BsonNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let doc = Document::from_reader(Cursor::new(input))
            .with_context(|| ERROR_DECODE_FAILED)?;
        let value = Self::bson_to_notation(&Bson::Document(doc))?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let bson = Self::notation_to_bson(&value)?;
        let doc = match bson {
            Bson::Document(doc) => doc,
            _ => return Err(anyhow!("{}: Expected BSON document", ERROR_ENCODE_FAILED)),
        };
        let mut buf = Vec::new();
        doc.to_writer(&mut buf)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf)
    }
}

impl BsonNotation {
    fn bson_to_notation(bson: &Bson) -> Result<NotationType> {
        match bson {
            Bson::Double(n) => Ok(NotationType::Number(*n)),
            Bson::String(s) => Ok(NotationType::String(s.clone())),
            Bson::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::bson_to_notation(item)?);
                }
                Ok(NotationType::Array(vec))
            },
            Bson::Document(doc) => {
                let mut vec = Vec::new();
                for (k, v) in doc {
                    vec.push((k.clone(), Self::bson_to_notation(v)?));
                }
                Ok(NotationType::Object(vec))
            },
            Bson::Boolean(b) => Ok(NotationType::Boolean(*b)),
            Bson::Null => Ok(NotationType::Null),
            Bson::Int32(n) => Ok(NotationType::Number(*n as f64)),
            Bson::Int64(n) => Ok(NotationType::Number(*n as f64)),
            Bson::Decimal128(n) => Ok(NotationType::Number(n.to_string().parse()?)),
            Bson::Timestamp(_) | Bson::DateTime(_) => {
                Ok(NotationType::String(bson.to_string()))
            },
            Bson::ObjectId(oid) => Ok(NotationType::String(oid.to_hex())),
            Bson::Binary(bin) => Ok(NotationType::String(hex::encode(bin))),
            Bson::RegularExpression(regex) => {
                Ok(NotationType::String(format!("/{}/{}", regex.pattern, regex.options)))
            },
            Bson::JavaScriptCode(code) => Ok(NotationType::String(code.clone())),
            Bson::JavaScriptCodeWithScope(scope) => {
                Ok(NotationType::String(scope.code.clone()))
            },
            Bson::Symbol(sym) => Ok(NotationType::String(sym.clone())),
            Bson::Undefined => Ok(NotationType::Null),
            Bson::MinKey => Ok(NotationType::String("MinKey".to_string())),
            Bson::MaxKey => Ok(NotationType::String("MaxKey".to_string())),
            Bson::DbPointer(ptr) => {
                Ok(NotationType::String(format!("{}:{}", ptr.namespace, ptr.id.to_hex())))
            },
        }
    }
    
    fn notation_to_bson(value: &NotationType) -> Result<Bson> {
        match value {
            NotationType::String(s) => Ok(Bson::String(s.clone())),
            NotationType::Number(n) => Ok(Bson::Double(*n)),
            NotationType::Boolean(b) => Ok(Bson::Boolean(*b)),
            NotationType::Null => Ok(Bson::Null),
            NotationType::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::notation_to_bson(item)?);
                }
                Ok(Bson::Array(vec))
            },
            NotationType::Object(obj) => {
                let mut doc = Document::new();
                for (k, v) in obj {
                    doc.insert(k.clone(), Self::notation_to_bson(v)?);
                }
                Ok(Bson::Document(doc))
            },
        }
    }
} 