use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::str;

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct TomlNotation;

impl Notation for TomlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        toml::from_str(s)
            .with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        let s = toml::to_string(value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        toml::to_string(value)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
} 