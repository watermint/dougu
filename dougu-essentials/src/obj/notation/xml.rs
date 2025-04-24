use anyhow::{Context, Result};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use std::str;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct XmlNotation;

impl Notation for XmlNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        let value = Self::parse_xml(s)?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let s = Self::format_xml(&value)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        Self::format_xml(&value)
    }
}

impl XmlNotation {
    fn parse_xml(s: &str) -> Result<NotationType> {
        // For now, we'll use quick-xml's deserialization as a base
        // and convert to our NotationType
        let value: quick_xml::de::Value = from_str(s)
            .with_context(|| ERROR_DECODE_FAILED)?;
        Self::xml_value_to_notation(&value)
    }
    
    fn xml_value_to_notation(value: &quick_xml::de::Value) -> Result<NotationType> {
        match value {
            quick_xml::de::Value::String(s) => Ok(NotationType::String(s.clone())),
            quick_xml::de::Value::Number(n) => Ok(NotationType::Number(*n as f64)),
            quick_xml::de::Value::Boolean(b) => Ok(NotationType::Boolean(*b)),
            quick_xml::de::Value::Null => Ok(NotationType::Null),
            quick_xml::de::Value::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::xml_value_to_notation(item)?);
                }
                Ok(NotationType::Array(vec))
            },
            quick_xml::de::Value::Object(obj) => {
                let mut vec = Vec::new();
                for (k, v) in obj {
                    vec.push((k.clone(), Self::xml_value_to_notation(v)?));
                }
                Ok(NotationType::Object(vec))
            },
        }
    }
    
    fn format_xml(value: &NotationType) -> Result<String> {
        let xml_value = Self::notation_to_xml_value(value)?;
        to_string(&xml_value)
            .with_context(|| ERROR_ENCODE_FAILED)
    }
    
    fn notation_to_xml_value(value: &NotationType) -> Result<quick_xml::de::Value> {
        match value {
            NotationType::String(s) => Ok(quick_xml::de::Value::String(s.clone())),
            NotationType::Number(n) => Ok(quick_xml::de::Value::Number(*n as i64)),
            NotationType::Boolean(b) => Ok(quick_xml::de::Value::Boolean(*b)),
            NotationType::Null => Ok(quick_xml::de::Value::Null),
            NotationType::Array(arr) => {
                let mut vec = Vec::new();
                for item in arr {
                    vec.push(Self::notation_to_xml_value(item)?);
                }
                Ok(quick_xml::de::Value::Array(vec))
            },
            NotationType::Object(obj) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Self::notation_to_xml_value(v)?);
                }
                Ok(quick_xml::de::Value::Object(map))
            },
        }
    }
} 