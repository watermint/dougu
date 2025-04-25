use crate::core::error::{error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str;

pub mod bson;
pub mod cbor;
pub mod error_messages;
pub mod json;
pub mod jsonl;
pub mod toml;
pub mod xml;
pub mod yaml;

pub use bson::BsonNotation;
pub use cbor::CborNotation;
pub use json::JsonNotation;
pub use jsonl::JsonlNotation;
pub use toml::TomlNotation;
pub use xml::XmlNotation;
pub use yaml::YamlNotation;

#[cfg(test)]
mod tests;

/// Notation trait abstracts encoding and decoding operations for different 
/// object notation formats (JSON, BSON, YAML, etc.)
pub trait Notation {
    /// Decode bytes into a NotationType value
    fn decode(&self, input: &[u8]) -> Result<NotationType>;

    /// Encode a value of type T into bytes
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone;

    /// Encode a value of type T into a string
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone;

    /// Optional method for encoding collections (mainly for JSONL)
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let vec: Vec<T> = values.to_vec();
        let notation_type = NotationType::Array(vec.into_iter().map(|v| v.into()).collect());
        self.encode(&notation_type)
    }

    fn to_notation_type<T>(&self, value: &T) -> Result<NotationType>
    where
        T: Into<NotationType> + Clone,
    {
        Ok(value.clone().into())
    }

    fn from_notation_type<T>(&self, notation: NotationType) -> Result<T>
    where
        T: From<NotationType>,
    {
        Ok(notation.into())
    }
}

/// Represents different kinds of numbers within NotationType
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum NumberVariant {
    Int(i64),
    Uint(u64),
    Float(f64),
}

/// An enum representing all available notation types
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum NotationType {
    Null,
    Boolean(bool),
    Number(NumberVariant),
    String(String),
    Array(Vec<NotationType>),
    Object(HashMap<String, NotationType>),
    Json(JsonNotation),
    Yaml(YamlNotation),
    Toml(TomlNotation),
    Xml(XmlNotation),
    Bson(BsonNotation),
    Cbor(CborNotation),
    Jsonl(JsonlNotation),
}

impl NotationType {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NotationType::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&NumberVariant> {
        match self {
            NotationType::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            NotationType::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<NotationType>> {
        match self {
            NotationType::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, NotationType>> {
        match self {
            NotationType::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&NotationType> {
        match self {
            NotationType::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            NotationType::Number(NumberVariant::Int(i)) => Some(*i),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            NotationType::Number(NumberVariant::Uint(u)) => Some(*u),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            NotationType::Number(NumberVariant::Float(f)) => Some(*f),
            _ => None,
        }
    }
}

impl From<bool> for NotationType {
    fn from(value: bool) -> Self {
        NotationType::Boolean(value)
    }
}

impl From<f64> for NotationType {
    fn from(value: f64) -> Self {
        NotationType::Number(NumberVariant::Float(value))
    }
}

impl From<i32> for NotationType {
    fn from(value: i32) -> Self {
        NotationType::Number(NumberVariant::Int(value as i64))
    }
}

impl From<i64> for NotationType {
    fn from(value: i64) -> Self {
        NotationType::Number(NumberVariant::Int(value))
    }
}

impl From<u32> for NotationType {
    fn from(value: u32) -> Self {
        NotationType::Number(NumberVariant::Uint(value as u64))
    }
}

impl From<u64> for NotationType {
    fn from(value: u64) -> Self {
        NotationType::Number(NumberVariant::Uint(value))
    }
}

impl From<String> for NotationType {
    fn from(value: String) -> Self {
        NotationType::String(value)
    }
}

impl From<&str> for NotationType {
    fn from(value: &str) -> Self {
        NotationType::String(value.to_string())
    }
}

impl<T> From<Vec<T>> for NotationType
where
    T: Into<NotationType>,
{
    fn from(value: Vec<T>) -> Self {
        NotationType::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T> From<HashMap<String, T>> for NotationType
where
    T: Into<NotationType>,
{
    fn from(value: HashMap<String, T>) -> Self {
        NotationType::Object(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl From<Vec<(String, NotationType)>> for NotationType {
    fn from(value: Vec<(String, NotationType)>) -> Self {
        NotationType::Object(value.into_iter().collect())
    }
}

impl fmt::Display for NumberVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberVariant::Int(i) => write!(f, "{}", i),
            NumberVariant::Uint(u) => write!(f, "{}", u),
            NumberVariant::Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl fmt::Display for NotationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotationType::Null => write!(f, "null"),
            NotationType::Boolean(b) => write!(f, "{}", b),
            NotationType::Number(n) => write!(f, "{}", n),
            NotationType::String(s) => write!(f, "\"{}\"", s),
            NotationType::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            NotationType::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            }
            _ => write!(f, "<notation>"),
        }
    }
}

// Implement Notation trait for NotationType
impl Notation for NotationType {
    fn decode(&self, input: &[u8]) -> Result<NotationType>
    {
        match self {
            NotationType::Json(notation) => notation.decode(input),
            NotationType::Yaml(notation) => notation.decode(input),
            NotationType::Toml(notation) => notation.decode(input),
            NotationType::Xml(notation) => notation.decode(input),
            NotationType::Bson(notation) => notation.decode(input),
            NotationType::Cbor(notation) => notation.decode(input),
            NotationType::Jsonl(notation) => notation.decode(input),
            _ => Err(error("Notation type does not support decoding")),
        }
    }

    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        match self {
            NotationType::Json(notation) => notation.encode(value),
            NotationType::Yaml(notation) => notation.encode(value),
            NotationType::Toml(notation) => notation.encode(value),
            NotationType::Xml(notation) => notation.encode(value),
            NotationType::Bson(notation) => notation.encode(value),
            NotationType::Cbor(notation) => notation.encode(value),
            NotationType::Jsonl(notation) => notation.encode(value),
            _ => Err(error("Notation type does not support encoding")),
        }
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        match self {
            NotationType::Json(notation) => notation.encode_to_string(value),
            NotationType::Yaml(notation) => notation.encode_to_string(value),
            NotationType::Toml(notation) => notation.encode_to_string(value),
            NotationType::Xml(notation) => notation.encode_to_string(value),
            _ => Err(error("Notation type does not support encoding to string")),
        }
    }
}
