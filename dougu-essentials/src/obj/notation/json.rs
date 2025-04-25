use crate::obj::notation::{Notation, NotationType, NumberVariant};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct JsonNotation;

impl JsonNotation {
    pub fn new() -> Self {
        JsonNotation
    }
}

impl Default for JsonNotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Notation for JsonNotation {
    fn encode<T>(&self, value: &T) -> anyhow::Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let json_value = notation_type_to_json_value(&notation_type)?;
        Ok(serde_json::to_vec(&json_value)?)
    }

    fn decode(&self, data: &[u8]) -> anyhow::Result<NotationType> {
        let json_value = serde_json::from_slice(data)?;
        json_value_to_notation_type(&json_value)
    }

    fn encode_to_string<T>(&self, value: &T) -> anyhow::Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let json_value = notation_type_to_json_value(&notation_type)?;
        Ok(serde_json::to_string(&json_value)?)
    }
}

/// Converts a NotationType enum into a serde_json::Value
pub fn notation_type_to_json_value(notation_type: &NotationType) -> Result<Value> {
    match notation_type {
        NotationType::Null => Ok(Value::Null),
        NotationType::Boolean(b) => Ok(Value::Bool(*b)),
        NotationType::Number(n) => {
            let f_val = match n {
                NumberVariant::Int(i) => *i as f64,
                NumberVariant::Uint(u) => *u as f64,
                NumberVariant::Float(f) => *f,
            };
            serde_json::Number::from_f64(f_val)
                .map(Value::Number)
                .ok_or_else(|| anyhow!("Invalid number for JSON: {}", f_val))
        }
        NotationType::String(s) => Ok(Value::String(s.clone())),
        NotationType::Array(arr) => {
            let values: Result<Vec<Value>> = arr.iter().map(notation_type_to_json_value).collect();
            Ok(Value::Array(values?))
        }
        NotationType::Object(obj) => {
            let map: Result<serde_json::Map<String, Value>> = obj
                .iter()
                .map(|(k, v)| notation_type_to_json_value(v).map(|json_v| (k.clone(), json_v)))
                .collect();
            Ok(Value::Object(map?))
        }
        _ => Err(anyhow!("Unsupported notation type for JSON conversion: {:?}", notation_type)),
    }
}

/// Converts a serde_json::Value into a NotationType enum
pub fn json_value_to_notation_type(value: &Value) -> Result<NotationType> {
    Ok(match value {
        Value::Null => NotationType::Null,
        Value::Bool(b) => NotationType::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                NotationType::Number(NumberVariant::Int(i))
            } else if let Some(u) = n.as_u64() {
                NotationType::Number(NumberVariant::Uint(u))
            } else if let Some(f) = n.as_f64() {
                NotationType::Number(NumberVariant::Float(f))
            } else {
                return Err(anyhow!("Invalid JSON number format: {}", n));
            }
        }
        Value::String(s) => NotationType::String(s.clone()),
        Value::Array(arr) => {
            let values: Result<Vec<NotationType>> = arr
                .iter()
                .map(json_value_to_notation_type)
                .collect();
            NotationType::Array(values?)
        }
        Value::Object(obj) => {
            let map: Result<HashMap<String, NotationType>> = obj
                .iter()
                .map(|(k, v)| json_value_to_notation_type(v).map(|nt| (k.clone(), nt)))
                .collect();
            NotationType::Object(map?)
        }
    })
}

#[cfg(test)]
mod tests {
    // ... tests using serde_json directly ...
} 