use anyhow::{Context, Result};
use toml::Value as TomlValue;
use std::str;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct TomlNotation;

impl Notation for TomlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        let value = Self::parse_toml(s)?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let s = Self::format_toml(&value)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        Self::format_toml(&value)
    }
}

impl TomlNotation {
    fn parse_toml(s: &str) -> Result<NotationType> {
        let value: TomlValue = s.parse()
            .with_context(|| ERROR_DECODE_FAILED)?;
        Self::toml_to_notation(&value)
    }
    
    fn toml_to_notation(toml: &TomlValue) -> Result<NotationType> {
        match toml {
            TomlValue::String(s) => Ok(NotationType::String(s.clone())),
            TomlValue::Integer(i) => Ok(NotationType::Number(*i as f64)),
            TomlValue::Float(f) => Ok(NotationType::Number(*f)),
            TomlValue::Boolean(b) => Ok(NotationType::Boolean(*b)),
            TomlValue::Datetime(dt) => Ok(NotationType::String(dt.to_string())),
            TomlValue::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::toml_to_notation(item)?);
                }
                Ok(NotationType::Array(vec))
            },
            TomlValue::Table(table) => {
                let mut vec = Vec::new();
                for (k, v) in table {
                    vec.push((k.clone(), Self::toml_to_notation(v)?));
                }
                Ok(NotationType::Object(vec))
            },
        }
    }
    
    fn format_toml(value: &NotationType) -> Result<String> {
        let toml = Self::notation_to_toml(value)?;
        Ok(toml.to_string())
    }
    
    fn notation_to_toml(value: &NotationType) -> Result<TomlValue> {
        match value {
            NotationType::String(s) => Ok(TomlValue::String(s.clone())),
            NotationType::Number(n) => {
                if n.fract() == 0.0 {
                    Ok(TomlValue::Integer(*n as i64))
                } else {
                    Ok(TomlValue::Float(*n))
                }
            },
            NotationType::Boolean(b) => Ok(TomlValue::Boolean(*b)),
            NotationType::Null => Ok(TomlValue::String("null".to_string())),
            NotationType::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::notation_to_toml(item)?);
                }
                Ok(TomlValue::Array(vec))
            },
            NotationType::Object(obj) => {
                let mut table = toml::Table::new();
                for (k, v) in obj {
                    table.insert(k.clone(), Self::notation_to_toml(v)?);
                }
                Ok(TomlValue::Table(table))
            },
        }
    }
} 