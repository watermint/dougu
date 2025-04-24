use anyhow::{anyhow, Result};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use bson::{self, Bson, Document};
use hex;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct BsonNotation;

impl BsonNotation {
    pub fn new() -> Self {
        BsonNotation
    }
}

impl Default for BsonNotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Notation for BsonNotation {
    fn encode<T>(&self, value: &T) -> anyhow::Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let bson_value = notation_type_to_bson(&notation_type)?;
        let doc = bson::doc! {"value": bson_value};
        let mut buf = Vec::new();
        doc.to_writer(&mut buf)?;
        Ok(buf)
    }

    fn decode(&self, data: &[u8]) -> anyhow::Result<NotationType> {
        let doc = Document::from_reader(&mut &data[..])?;
        let bson_value = doc.get("value").ok_or_else(|| anyhow!("Missing 'value' key in BSON document"))?;
        bson_to_notation_type(bson_value)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        // Encode to BSON bytes first
        let bytes = self.encode(value)?;
        // Return hex representation of BSON bytes
        Ok(hex::encode(bytes))
    }
}

fn bson_to_notation_type(value: &Bson) -> Result<NotationType> {
    match value {
        Bson::Double(f) => Ok(NotationType::Number(NumberVariant::Float(*f))),
        Bson::String(s) => Ok(NotationType::String(s.clone())),
        Bson::Array(arr) => {
            let values: Result<Vec<NotationType>> = arr.iter().map(bson_to_notation_type).collect();
            Ok(NotationType::Array(values?))
        }
        Bson::Document(doc) => {
            let map: Result<HashMap<String, NotationType>> = doc
                .iter()
                .map(|(k, v)| bson_to_notation_type(v).map(|nt| (k.clone(), nt)))
                .collect();
            Ok(NotationType::Object(map?))
        }
        Bson::Boolean(b) => Ok(NotationType::Boolean(*b)),
        Bson::Null => Ok(NotationType::Null),
        Bson::Int32(i) => Ok(NotationType::Number(NumberVariant::Int(*i as i64))),
        Bson::Int64(i) => Ok(NotationType::Number(NumberVariant::Int(*i))),
        Bson::Binary(bin) => Ok(NotationType::String(BASE64_STANDARD.encode(&bin.bytes))),
        Bson::Timestamp(ts) => Ok(NotationType::String(ts.to_string())),
        _ => Err(anyhow!("Unsupported BSON type encountered: {:?}", value)),
    }
}

fn notation_type_to_bson(notation_type: &NotationType) -> Result<Bson> {
    match notation_type {
        NotationType::Null => Ok(Bson::Null),
        NotationType::Boolean(b) => Ok(Bson::Boolean(*b)),
        NotationType::Number(n) => {
            match n {
                NumberVariant::Int(i) => {
                    if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                         Ok(Bson::Int32(*i as i32))
                    } else {
                         Ok(Bson::Int64(*i))
                    }
                },
                NumberVariant::Uint(u) => {
                    if let Ok(i_val) = (*u).try_into() as Result<i64, _> {
                        if i_val >= i32::MIN as i64 && i_val <= i32::MAX as i64 {
                            Ok(Bson::Int32(i_val.try_into().unwrap_or(i32::MAX)))
                        } else {
                            Ok(Bson::Int64(i_val))
                        }
                    } else {
                        Ok(Bson::Double(*u as f64))
                    }
                },
                NumberVariant::Float(f) => Ok(Bson::Double(*f)),
            }
        }
        NotationType::String(s) => Ok(Bson::String(s.clone())),
        NotationType::Array(arr) => {
            let values: Result<Vec<Bson>> = arr.iter().map(notation_type_to_bson).collect();
            Ok(Bson::Array(values?))
        }
        NotationType::Object(map) => {
            let doc: Result<Document> = map
                .iter()
                .map(|(k, v)| notation_type_to_bson(v).map(|bson_v| (k.clone(), bson_v)))
                .collect();
            Ok(Bson::Document(doc?))
        }
        _ => Err(anyhow!("Unsupported NotationType for BSON conversion: {:?}", notation_type)),
    }
} 