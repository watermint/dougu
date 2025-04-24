use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;
use std::str;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct YamlNotation;

impl Notation for YamlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        let value = Self::parse_yaml(s)?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let s = Self::format_yaml(&value)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        Self::format_yaml(&value)
    }
}

impl YamlNotation {
    fn parse_yaml(s: &str) -> Result<NotationType> {
        let value: YamlValue = serde_yaml::from_str(s)
            .with_context(|| ERROR_DECODE_FAILED)?;
        Self::yaml_to_notation(&value)
    }
    
    fn yaml_to_notation(yaml: &YamlValue) -> Result<NotationType> {
        match yaml {
            YamlValue::Null => Ok(NotationType::Null),
            YamlValue::Bool(b) => Ok(NotationType::Boolean(*b)),
            YamlValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(NotationType::Number(i as f64))
                } else if let Some(f) = n.as_f64() {
                    Ok(NotationType::Number(f))
                } else {
                    Err(anyhow!("{}: Invalid YAML number", ERROR_DECODE_FAILED))
                }
            },
            YamlValue::String(s) => Ok(NotationType::String(s.clone())),
            YamlValue::Sequence(seq) => {
                let mut vec = Vec::new();
                for item in seq {
                    vec.push(Self::yaml_to_notation(item)?);
                }
                Ok(NotationType::Array(vec))
            },
            YamlValue::Mapping(map) => {
                let mut vec = Vec::new();
                for (k, v) in map {
                    let key = match k {
                        YamlValue::String(s) => s.clone(),
                        YamlValue::Number(n) => n.to_string(),
                        YamlValue::Bool(b) => b.to_string(),
                        _ => return Err(anyhow!("{}: Invalid YAML map key", ERROR_DECODE_FAILED)),
                    };
                    vec.push((key, Self::yaml_to_notation(v)?));
                }
                Ok(NotationType::Object(vec))
            },
        }
    }
    
    fn format_yaml(value: &NotationType) -> Result<String> {
        let yaml = Self::notation_to_yaml(value)?;
        serde_yaml::to_string(&yaml)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
    
    fn notation_to_yaml(value: &NotationType) -> Result<YamlValue> {
        match value {
            NotationType::Null => Ok(YamlValue::Null),
            NotationType::Boolean(b) => Ok(YamlValue::Bool(*b)),
            NotationType::Number(n) => Ok(YamlValue::Number(serde_yaml::Number::from_f64(*n)
                .ok_or_else(|| anyhow!("{}: Invalid number", ERROR_ENCODE_FAILED))?)),
            NotationType::String(s) => Ok(YamlValue::String(s.clone())),
            NotationType::Array(arr) => {
                let mut seq = Vec::new();
                for item in arr {
                    seq.push(Self::notation_to_yaml(item)?);
                }
                Ok(YamlValue::Sequence(seq))
            },
            NotationType::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (k, v) in obj {
                    map.insert(
                        YamlValue::String(k.clone()),
                        Self::notation_to_yaml(v)?,
                    );
                }
                Ok(YamlValue::Mapping(map))
            },
        }
    }
} 