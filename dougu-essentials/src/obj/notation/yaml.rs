use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct YamlNotation;

impl Notation for YamlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        serde_yaml::from_slice(input)
            .with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        let s = serde_yaml::to_string(value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        serde_yaml::to_string(value)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
} 