use anyhow::{Context, Result};
use bson::{from_document, to_document, Document};
use serde::{de::DeserializeOwned, Serialize};
use std::io::Cursor;

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct BsonNotation;

impl Notation for BsonNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let doc = Document::from_reader(Cursor::new(input))
            .with_context(|| ERROR_DECODE_FAILED)?;
        from_document(doc).with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        let doc = to_document(value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        let mut buf = Vec::new();
        doc.to_writer(&mut buf)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf)
    }
} 