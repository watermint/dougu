use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::str;

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct JsonlNotation;

impl Notation for JsonlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        let values: Vec<Value> = s
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line))
            .collect::<Result<Vec<Value>, _>>()
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        // Convert the Vec<Value> into a single Value::Array
        let jsonl_array = Value::Array(values);
        
        // Attempt to deserialize the Value::Array into the target type T
        serde_json::from_value(jsonl_array)
            .with_context(|| format!("Failed to deserialize JSONL array into target type: {}", std::any::type_name::<T>()))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        // For single object, encode as JSON with a newline
        let mut json = serde_json::to_vec(value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        json.push(b'\n');
        Ok(json)
    }
    
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        let mut result = Vec::new();
        
        for value in values {
            let mut json = serde_json::to_vec(value)
                .with_context(|| ERROR_ENCODE_FAILED)?;
            json.push(b'\n');
            result.extend_from_slice(&json);
        }
        
        Ok(result)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        let encoded = self.encode(value)?;
        Ok(String::from_utf8(encoded)?)
    }
} 