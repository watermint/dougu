use crate::core::error::{error, Result};
use crate::obj::notation::{JsonNotation, Notation, NotationType};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct JsonlNotation {
    json_notation: JsonNotation,
}

impl Default for JsonlNotation {
    fn default() -> Self {
        Self {
            json_notation: JsonNotation::default(),
        }
    }
}

impl Notation for JsonlNotation {
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let value = value.clone().into();
        let mut result = Vec::new();
        match value {
            NotationType::Array(arr) => {
                for item in arr {
                    let encoded = self.json_notation.encode_to_string(&item)?;
                    result.extend_from_slice(encoded.as_bytes());
                    result.push(b'\n');
                }
            }
            _ => {
                let encoded = self.json_notation.encode_to_string(&value)?;
                result.extend_from_slice(encoded.as_bytes());
                result.push(b'\n');
            }
        }
        Ok(result)
    }

    fn decode(&self, data: &[u8]) -> Result<NotationType> {
        let content = String::from_utf8(data.to_vec())?;
        let lines: Vec<&str> = content.lines().collect();
        let mut result_items = Vec::new();

        for line in lines {
            if !line.trim().is_empty() {
                let decoded_item: NotationType = self.json_notation.decode(line.as_bytes())?;
                result_items.push(decoded_item);
            }
        }

        if result_items.len() == 1 {
            Ok(result_items[0].clone())
        } else {
            Ok(NotationType::Array(result_items))
        }
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let encoded = self.encode(value)?;
        String::from_utf8(encoded).map_err(|e| error(format!("{}", e)))
    }

    fn encode_collection<T>(&self, values: &[T]) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let mut result = Vec::new();
        for value in values {
            let encoded = self.json_notation.encode_to_string(value)?;
            result.extend_from_slice(encoded.as_bytes());
            result.push(b'\n');
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::notation::NumberVariant;
    use std::collections::HashMap;

    #[test]
    fn test_jsonl_roundtrip() -> Result<()> {
        let notation = JsonlNotation::default();
        let mut map1: HashMap<String, NotationType> = HashMap::new();
        map1.insert("key1".to_string(), NotationType::String("value1".to_string()));
        map1.insert("key2".to_string(), NotationType::Number(NumberVariant::Float(42.0)));
        let obj1 = NotationType::Object(map1);

        let mut map2: HashMap<String, NotationType> = HashMap::new();
        map2.insert("alpha".to_string(), NotationType::Boolean(true));
        map2.insert("beta".to_string(), NotationType::Null);
        let obj2 = NotationType::Object(map2);

        let input_collection = vec![obj1.clone(), obj2.clone()];

        let encoded_bytes = notation.encode_collection(&input_collection)?;
        let encoded_string = String::from_utf8(encoded_bytes)?;

        assert_eq!(encoded_string.lines().count(), 2);

        let mut decoded_collection = Vec::new();
        for line in encoded_string.lines() {
            if !line.trim().is_empty() {
                let decoded_obj = notation.decode(line.as_bytes())?;
                decoded_collection.push(decoded_obj);
            }
        }

        assert_eq!(input_collection.len(), decoded_collection.len());
        assert_eq!(input_collection, decoded_collection);

        Ok(())
    }
} 