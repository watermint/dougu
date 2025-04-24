use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct JsonNotation;

impl Notation for JsonNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(input)
            .with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        serde_json::to_vec(value)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        serde_json::to_string(value)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
} 