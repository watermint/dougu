use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct CborNotation;

impl Notation for CborNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        ciborium::de::from_reader(input)
            .with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(value, &mut buf)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf)
    }
} 