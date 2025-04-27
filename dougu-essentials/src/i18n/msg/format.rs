// Message format handling module
// This module provides utilities for converting between different message formats

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::core::error::{error, Result};
use crate::i18n::locale::LocaleId;
use crate::i18n::msg::MessageBundle;

/// Supported message formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageFormat {
    /// JSON format with simple key-value pairs
    Json,
    /// Fluent format (FTL)
    Fluent,
}

impl MessageFormat {
    /// Load a message bundle from a file in the specified format
    pub fn load_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        locale: LocaleId,
    ) -> Result<MessageBundle> {
        match self {
            MessageFormat::Json => Self::load_json(path, locale),
            MessageFormat::Fluent => Self::load_fluent(path, locale),
        }
    }

    /// Load a message bundle from JSON content
    pub fn load_from_json_string(
        json_content: &str,
        locale: LocaleId,
    ) -> Result<MessageBundle> {
        let value: serde_json::Value = serde_json::from_str(json_content)
            .map_err(|e| error(format!("Failed to parse JSON: {}", e)))?;

        if !value.is_object() {
            return Err(error("JSON content must be an object"));
        }

        let mut bundle = MessageBundle::new(locale);

        if let serde_json::Value::Object(map) = value {
            for (key, value) in map {
                if let Some(message) = value.as_str() {
                    bundle.add_message(key, message);
                } else if let Ok(string_value) = serde_json::to_string(&value) {
                    bundle.add_message(key, string_value);
                }
            }
        }

        Ok(bundle)
    }

    /// Load a message bundle from Fluent content
    pub fn load_from_fluent_string(
        fluent_content: &str,
        locale: LocaleId,
    ) -> Result<MessageBundle> {
        // Parse the Fluent content
        let mut bundle = MessageBundle::new(locale);

        // Use a simple parser to extract message IDs and values
        for line in fluent_content.lines() {
            let line = line.trim();

            // Skip comments and blank lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Look for simple message definitions: key = value
            if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim();
                let value = line[equals_pos + 1..].trim();

                // Skip if key is empty
                if key.is_empty() {
                    continue;
                }

                // Convert Fluent-style placeholders { $name } to {name}
                let converted_value = Self::convert_fluent_placeholders(value);

                bundle.add_message(key, converted_value);
            }
        }

        Ok(bundle)
    }

    /// Load a JSON message bundle from a file
    fn load_json<P: AsRef<Path>>(path: P, locale: LocaleId) -> Result<MessageBundle> {
        let mut file = File::open(path)
            .map_err(|e| error(format!("Failed to open JSON file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| error(format!("Failed to read JSON file: {}", e)))?;

        Self::load_from_json_string(&content, locale)
    }

    /// Load a Fluent message bundle from a file
    fn load_fluent<P: AsRef<Path>>(path: P, locale: LocaleId) -> Result<MessageBundle> {
        let mut file = File::open(path)
            .map_err(|e| error(format!("Failed to open Fluent file: {}", e)))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| error(format!("Failed to read Fluent file: {}", e)))?;

        Self::load_from_fluent_string(&content, locale)
    }

    /// Convert Fluent-style placeholders { $name } to {name}
    fn convert_fluent_placeholders(value: &str) -> String {
        // Regular expression would be better, but we'll use a simple string-based approach
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&' ') {
                // Possibly a Fluent placeholder
                let mut placeholder = String::new();
                placeholder.push(c);

                // Collect the whole placeholder
                let mut is_fluent_placeholder = false;
                while let Some(pc) = chars.next() {
                    placeholder.push(pc);

                    // Check for the dollar sign that indicates a Fluent placeholder
                    if pc == '$' {
                        is_fluent_placeholder = true;
                    }

                    // End of placeholder
                    if pc == '}' {
                        break;
                    }
                }

                if is_fluent_placeholder {
                    // Extract the name without the $ and spaces
                    let name = placeholder
                        .chars()
                        .skip_while(|c| *c != '$')
                        .skip(1)  // Skip the $
                        .take_while(|c| *c != '}')
                        .collect::<String>()
                        .trim()
                        .to_string();

                    // Add the simplified placeholder
                    result.push_str(&format!("{{{}}}", name));
                } else {
                    // Not a Fluent placeholder, add as is
                    result.push_str(&placeholder);
                }
            } else {
                // Regular character
                result.push(c);
            }
        }

        result
    }

    /// Convert simple placeholders {name} to Fluent-style { $name }
    fn convert_to_fluent_placeholders(value: &str) -> String {
        let mut result = String::new();
        let mut start_index = 0;

        while let Some(start) = value[start_index..].find('{') {
            let abs_start = start_index + start;

            // Add everything up to the placeholder
            result.push_str(&value[start_index..abs_start]);

            // Look for the closing brace
            if let Some(end) = value[abs_start..].find('}') {
                let abs_end = abs_start + end + 1;
                let name = &value[abs_start + 1..abs_end - 1];

                // Add the Fluent-style placeholder
                result.push_str(&format!("{{ ${} }}", name));

                // Update the start index for the next search
                start_index = abs_end;
            } else {
                // No closing brace found, add the opening brace and continue
                result.push('{');
                start_index = abs_start + 1;
            }
        }

        // Add any remaining text
        result.push_str(&value[start_index..]);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::locale::{LanguageId, RegionId};
    use crate::i18n::msg::MessageArgs;

    #[test]
    fn test_json_format() {
        let json = r#"{
            "greeting": "Hello, {name}!",
            "farewell": "Goodbye, {name}!"
        }"#;

        let locale = LocaleId::new(LanguageId::new("en"), Some(RegionId::new("US")));
        let bundle = MessageFormat::load_from_json_string(json, locale).unwrap();

        let mut args = MessageArgs::new();
        args.add("name", "World");

        let greeting = bundle.format_message("greeting", Some(&args)).unwrap();
        assert_eq!(greeting, "Hello, World!");
    }

    #[test]
    fn test_fluent_format() {
        let fluent = r#"
# Simple message
greeting = Hello, { $name }!

# Another message
farewell = Goodbye, { $name }!
"#;

        let locale = LocaleId::new(LanguageId::new("en"), Some(RegionId::new("US")));
        let bundle = MessageFormat::load_from_fluent_string(fluent, locale).unwrap();

        let mut args = MessageArgs::new();
        args.add("name", "World");

        let greeting = bundle.format_message("greeting", Some(&args)).unwrap();
        assert_eq!(greeting, "Hello, World!");
    }

    #[test]
    fn test_placeholder_conversion() {
        let fluent_style = "Hello, { $name }, welcome to { $app }!";
        let simple_style = "Hello, {name}, welcome to {app}!";

        let converted = MessageFormat::convert_fluent_placeholders(fluent_style);
        assert_eq!(converted, simple_style);

        let back_to_fluent = MessageFormat::convert_to_fluent_placeholders(&converted);
        assert_eq!(back_to_fluent, "Hello, { $name }, welcome to { $app }!");
    }
} 