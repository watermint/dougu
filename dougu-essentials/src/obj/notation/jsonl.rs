use anyhow::{Context, Result};
use std::str;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct JsonlNotation;

impl Notation for JsonlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        let value = Self::parse_jsonl(s)?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let s = Self::format_jsonl(&value)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        Self::format_jsonl(&value)
    }
    
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let mut buf = Vec::new();
        for value in values {
            let value = value.into();
            let s = Self::format_jsonl(&value)?;
            buf.extend_from_slice(s.as_bytes());
            buf.push(b'\n');
        }
        Ok(buf)
    }
}

impl JsonlNotation {
    fn parse_jsonl(s: &str) -> Result<NotationType> {
        let mut lines = s.lines();
        let first_line = lines.next()
            .ok_or_else(|| anyhow!("{}: Empty JSONL input", ERROR_DECODE_FAILED))?;
        
        let first_value = super::json::JsonNotation::parse_json(first_line)?;
        
        if lines.next().is_none() {
            // Single line - treat as regular JSON
            return Ok(first_value);
        }
        
        // Multiple lines - treat as array of JSON objects
        let mut arr = vec![first_value];
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let value = super::json::JsonNotation::parse_json(line)?;
            arr.push(value);
        }
        Ok(NotationType::Array(arr))
    }
    
    fn format_jsonl(value: &NotationType) -> Result<String> {
        super::json::JsonNotation::format_json(value)
    }
} 