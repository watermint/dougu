use anyhow::Result;
use std::str;

pub mod json;
pub mod bson;
pub mod cbor;
pub mod xml;
pub mod yaml;
pub mod toml;
pub mod jsonl;

#[cfg(test)]
mod tests;

/// Notation trait abstracts encoding and decoding operations for different 
/// object notation formats (JSON, BSON, YAML, etc.)
pub trait Notation {
    /// Decode bytes into a value of type T
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>;
    
    /// Encode a value of type T into bytes
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>;
    
    /// Encode a value of type T into a string
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> {
        let bytes = self.encode(value)?;
        Ok(String::from_utf8(bytes)?)
    }
    
    /// Optional method for encoding collections (mainly for JSONL)
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Into<NotationType> {
        // Default implementation encodes as a regular array/collection
        self.encode(values)
    }
}

/// An enum representing all available notation types
#[derive(Debug, Clone)]
pub enum NotationType {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<NotationType>),
    Object(Vec<(String, NotationType)>),
}

impl From<String> for NotationType {
    fn from(s: String) -> Self {
        NotationType::String(s)
    }
}

impl From<f64> for NotationType {
    fn from(n: f64) -> Self {
        NotationType::Number(n)
    }
}

impl From<bool> for NotationType {
    fn from(b: bool) -> Self {
        NotationType::Boolean(b)
    }
}

impl<T: Into<NotationType>> From<Vec<T>> for NotationType {
    fn from(v: Vec<T>) -> Self {
        NotationType::Array(v.into_iter().map(|x| x.into()).collect())
    }
}

impl<T: Into<NotationType>> From<Vec<(String, T)>> for NotationType {
    fn from(v: Vec<(String, T)>) -> Self {
        NotationType::Object(v.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl From<()> for NotationType {
    fn from(_: ()) -> Self {
        NotationType::Null
    }
}

impl std::fmt::Display for NotationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotationType::String(s) => write!(f, "{}", s),
            NotationType::Number(n) => write!(f, "{}", n),
            NotationType::Boolean(b) => write!(f, "{}", b),
            NotationType::Null => write!(f, "null"),
            NotationType::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            NotationType::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            },
        }
    }
}
