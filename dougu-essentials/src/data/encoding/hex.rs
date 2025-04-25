use crate::core::error::{error, Result};
use crate::data::encoding::BinaryTextCodec;

/// Hex encoding configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HexEngine {
    /// Lowercase hex (default)
    Lower,
    /// Uppercase hex
    Upper,
}

/// Hex encoder/decoder
#[derive(Debug, Clone, Copy)]
pub struct Hex {
    engine: HexEngine,
}

impl Hex {
    /// Create a new Hex encoder/decoder with the given engine
    pub fn new(engine: HexEngine) -> Self {
        Self { engine }
    }

    /// Create a new Hex encoder/decoder with lowercase engine (default)
    pub fn lower() -> Self {
        Self::new(HexEngine::Lower)
    }

    /// Create a new Hex encoder/decoder with uppercase engine
    pub fn upper() -> Self {
        Self::new(HexEngine::Upper)
    }
}

impl BinaryTextCodec for Hex {
    fn encode(&self, data: &[u8]) -> Result<String> {
        match self.engine {
            HexEngine::Lower => Ok(hex::encode(data)),
            HexEngine::Upper => Ok(hex::encode_upper(data)),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>> {
        match hex::decode(text) {
            Ok(data) => Ok(data),
            Err(e) => Err(error(format!("Failed to decode Hex: {}", e))),
        }
    }
} 