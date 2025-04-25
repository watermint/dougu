use std::collections::HashMap;

/// Error messages for various operations
pub struct ErrorMessages {
    messages: HashMap<String, String>,
}

impl ErrorMessages {
    pub fn new() -> Self {
        let mut messages = HashMap::new();
        messages.insert(
            "xml_parse_error".to_string(),
            "Failed to parse XML document".to_string(),
        );
        messages.insert(
            "xml_encode_error".to_string(),
            "Failed to encode data to XML".to_string(),
        );
        messages.insert(
            "xml_decode_error".to_string(),
            "Failed to decode XML data".to_string(),
        );
        Self { messages }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.messages.get(key)
    }
}

impl Default for ErrorMessages {
    fn default() -> Self {
        Self::new()
    }
} 