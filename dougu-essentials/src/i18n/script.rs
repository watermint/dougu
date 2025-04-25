// Script related types and implementations
use std::str::FromStr;

/// Script identifier as defined by ISO 15924
///
/// Four-letter codes (e.g., "Latn" for Latin script)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptId(String);

impl ScriptId {
    /// Create a new script identifier following ISO 15924
    pub fn new(code: &str) -> Self {
        // First letter uppercase, rest lowercase according to BCP 47
        let code = code.chars().enumerate().map(|(i, c)| {
            if i == 0 { c.to_uppercase().next().unwrap() } else { c.to_lowercase().next().unwrap() }
        }).collect::<String>();
        Self(code)
    }

    /// Get the script code
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ScriptId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ISO 15924 (four-letter)
        if s.len() != 4 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("Invalid script identifier: must be 4 ASCII letters (ISO 15924)");
        }
        Ok(Self::new(s))
    }
} 