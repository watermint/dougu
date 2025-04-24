use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

pub mod json;
pub mod bson;
pub mod cbor;
pub mod xml;
pub mod yaml;
pub mod toml;
pub mod jsonl;

#[cfg(test)]
mod tests;

use crate::obj::Format;

/// Notation trait abstracts encoding and decoding operations for different 
/// object notation formats (JSON, BSON, YAML, etc.)
pub trait Notation {
    /// Decode bytes into a value of type T
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned;
    
    /// Encode a value of type T into bytes
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized;
    
    /// Encode a value of type T into a string
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized {
        let bytes = self.encode(value)?;
        Ok(String::from_utf8(bytes)?)
    }
    
    /// Optional method for encoding collections (mainly for JSONL)
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Serialize {
        // Default implementation encodes as a regular array/collection
        self.encode(values)
    }
}

/// An enum representing all available notation types
#[derive(Debug, Clone)]
pub enum NotationType {
    Json(json::JsonNotation),
    Bson(bson::BsonNotation),
    Cbor(cbor::CborNotation),
    Xml(xml::XmlNotation),
    Yaml(yaml::YamlNotation),
    Toml(toml::TomlNotation),
    Jsonl(jsonl::JsonlNotation),
}

impl Notation for NotationType {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        match self {
            NotationType::Json(n) => n.decode(input),
            NotationType::Bson(n) => n.decode(input),
            NotationType::Cbor(n) => n.decode(input),
            NotationType::Xml(n) => n.decode(input),
            NotationType::Yaml(n) => n.decode(input),
            NotationType::Toml(n) => n.decode(input),
            NotationType::Jsonl(n) => n.decode(input),
        }
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize + ?Sized,
    {
        match self {
            NotationType::Json(n) => n.encode(value),
            NotationType::Bson(n) => n.encode(value),
            NotationType::Cbor(n) => n.encode(value),
            NotationType::Xml(n) => n.encode(value),
            NotationType::Yaml(n) => n.encode(value),
            NotationType::Toml(n) => n.encode(value),
            NotationType::Jsonl(n) => n.encode(value),
        }
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Serialize + ?Sized,
    {
        match self {
            NotationType::Json(n) => n.encode_to_string(value),
            NotationType::Bson(n) => n.encode_to_string(value),
            NotationType::Cbor(n) => n.encode_to_string(value),
            NotationType::Xml(n) => n.encode_to_string(value),
            NotationType::Yaml(n) => n.encode_to_string(value),
            NotationType::Toml(n) => n.encode_to_string(value),
            NotationType::Jsonl(n) => n.encode_to_string(value),
        }
    }
    
    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        match self {
            NotationType::Json(n) => n.encode_collection(values),
            NotationType::Bson(n) => n.encode_collection(values),
            NotationType::Cbor(n) => n.encode_collection(values),
            NotationType::Xml(n) => n.encode_collection(values),
            NotationType::Yaml(n) => n.encode_collection(values),
            NotationType::Toml(n) => n.encode_collection(values),
            NotationType::Jsonl(n) => n.encode_collection(values),
        }
    }
}

/// Get a notation implementation for the given format
pub fn get_notation(format: Format) -> NotationType {
    match format {
        Format::Json => NotationType::Json(json::JsonNotation),
        Format::Bson => NotationType::Bson(bson::BsonNotation),
        Format::Cbor => NotationType::Cbor(cbor::CborNotation),
        Format::Xml => NotationType::Xml(xml::XmlNotation),
        Format::Yaml => NotationType::Yaml(yaml::YamlNotation),
        Format::Toml => NotationType::Toml(toml::TomlNotation),
        Format::Jsonl => NotationType::Jsonl(jsonl::JsonlNotation),
    }
} 