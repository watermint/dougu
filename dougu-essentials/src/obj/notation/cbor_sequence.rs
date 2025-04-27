use crate::core::error::{error, Result};
use crate::obj::notation::cbor::{cbor_value_to_notation_type, CborNotation};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use ciborium::from_reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// CBOR Sequence notation that follows RFC 8742
/// Concatenates multiple CBOR items without delimiters or additional framing
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct CborSequenceNotation {
    cbor_notation: CborNotation,
}

impl Default for CborSequenceNotation {
    fn default() -> Self {
        Self {
            cbor_notation: CborNotation,
        }
    }
}

impl Notation for CborSequenceNotation {
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let value = value.clone().into();
        let mut result = Vec::new();
        match value {
            NotationType::Array(arr) => {
                for item in arr {
                    let encoded = self.cbor_notation.encode(&item)?;
                    result.extend_from_slice(&encoded);
                }
            }
            _ => {
                let encoded = self.cbor_notation.encode(&value)?;
                result.extend_from_slice(&encoded);
            }
        }
        Ok(result)
    }

    fn decode(&self, data: &[u8]) -> Result<NotationType> {
        if data.is_empty() {
            return Ok(NotationType::Array(Vec::new()));
        }

        let mut result_items = Vec::new();
        let mut cursor = Cursor::new(data);

        // Continue reading CBOR items until we reach the end of the data
        while cursor.position() < data.len() as u64 {
            // Get a reader from the current position
            let reader = cursor.by_ref();

            // Try to parse a CBOR value from the current position
            match from_reader(reader) {
                Ok(cbor_value) => {
                    let decoded_item = cbor_value_to_notation_type(&cbor_value)?;
                    result_items.push(decoded_item);
                }
                Err(e) => {
                    // If we can't parse more items but have read some data, return what we have
                    if !result_items.is_empty() {
                        break;
                    }
                    return Err(error(format!("Failed to decode CBOR sequence: {}", e)));
                }
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
        let bytes = self.encode(value)?;
        Ok(hex::encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cbor_sequence_roundtrip() -> Result<()> {
        let notation = CborSequenceNotation::default();

        // Test single item encoding/decoding
        let mut map: HashMap<String, NotationType> = HashMap::new();
        map.insert("name".to_string(), "test".into());
        map.insert("value".to_string(), (42.5f64).into());
        let input: NotationType = map.into();

        let encoded = notation.encode(&input)?;
        let decoded = notation.decode(&encoded)?;

        // Single items should be returned directly
        assert_eq!(input, decoded);

        // Test array of items
        let mut map1: HashMap<String, NotationType> = HashMap::new();
        map1.insert("name".to_string(), "test1".into());
        map1.insert("value".to_string(), (42.5f64).into());

        let mut map2: HashMap<String, NotationType> = HashMap::new();
        map2.insert("name".to_string(), "test2".into());
        map2.insert("value".to_string(), (84.0f64).into());

        let array_input = NotationType::Array(vec![map1.into(), map2.into()]);

        let encoded_array = notation.encode(&array_input)?;
        let decoded_array = notation.decode(&encoded_array)?;

        // For arrays with multiple items, use custom comparison to handle different map orders
        // and numeric type conversions
        if let (NotationType::Array(input_arr), NotationType::Array(decoded_arr)) = (&array_input, &decoded_array) {
            assert_eq!(input_arr.len(), decoded_arr.len(), "Array lengths differ");

            for (i, (input_item, decoded_item)) in input_arr.iter().zip(decoded_arr.iter()).enumerate() {
                match (input_item, decoded_item) {
                    (NotationType::Object(input_map), NotationType::Object(decoded_map)) => {
                        // Check same number of keys
                        assert_eq!(input_map.len(), decoded_map.len(), "Object at index {} has different key count", i);

                        // Check each key-value pair, allowing for numeric type differences
                        for (key, input_value) in input_map {
                            let decoded_value = decoded_map.get(key).expect(&format!("Missing key '{}' at index {}", key, i));
                            match (input_value, decoded_value) {
                                (NotationType::Number(n1), NotationType::Number(n2)) => {
                                    // For numbers, compare their floating point values
                                    let f1 = match n1 {
                                        NumberVariant::Int(i) => *i as f64,
                                        NumberVariant::Uint(u) => *u as f64,
                                        NumberVariant::Float(f) => *f,
                                    };
                                    let f2 = match n2 {
                                        NumberVariant::Int(i) => *i as f64,
                                        NumberVariant::Uint(u) => *u as f64,
                                        NumberVariant::Float(f) => *f,
                                    };
                                    assert!((f1 - f2).abs() < f64::EPSILON,
                                            "Number values differ for key '{}' at index {}: {} vs {}", key, i, f1, f2);
                                }
                                _ => assert_eq!(input_value, decoded_value,
                                                "Values differ for key '{}' at index {}", key, i),
                            }
                        }
                    }
                    _ => assert_eq!(input_item, decoded_item, "Items at index {} differ", i),
                }
            }
        } else {
            assert_eq!(array_input, decoded_array, "Arrays don't match");
        }

        Ok(())
    }
} 