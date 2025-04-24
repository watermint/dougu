use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::obj::resources::errors::*;
use super::Notation;

#[derive(Debug, Clone)]
pub struct XmlNotation;

impl Notation for XmlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        quick_xml::de::from_reader(input)
            .with_context(|| ERROR_DECODE_FAILED)
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        let mut buf = String::new();
        quick_xml::se::to_writer(&mut buf, value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        let mut buf = String::new();
        quick_xml::se::to_writer(&mut buf, value)
            .with_context(|| ERROR_ENCODE_FAILED)?;
        Ok(buf)
    }
} 