use crate::core::error::{error, Result};
use crate::data::encoding::BinaryTextCodec;

/// Base32 encoding configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base32Engine {
    /// Standard Base32 encoding (RFC 4648)
    Standard,
    /// Base32 with extended hex alphabet (RFC 4648)
    Hex,
}

/// Base32 encoder/decoder
#[derive(Debug, Clone, Copy)]
pub struct Base32 {
    engine: Base32Engine,
}

impl Base32 {
    /// Create a new Base32 encoder/decoder with the given engine
    pub fn new(engine: Base32Engine) -> Self {
        Self { engine }
    }

    /// Create a new Base32 encoder/decoder with standard engine
    pub fn standard() -> Self {
        Self::new(Base32Engine::Standard)
    }

    /// Create a new Base32 encoder/decoder with hex engine
    pub fn hex() -> Self {
        Self::new(Base32Engine::Hex)
    }
}

impl BinaryTextCodec for Base32 {
    fn encode(&self, data: &[u8]) -> Result<String> {
        let mut result = String::new();
        let mut bits = 0;
        let mut buffer = 0u16;

        let alphabet = match self.engine {
            Base32Engine::Standard => "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
            Base32Engine::Hex => "0123456789ABCDEFGHIJKLMNOPQRSTUV",
        };

        for &byte in data {
            buffer = (buffer << 8) | (byte as u16);
            bits += 8;

            while bits >= 5 {
                bits -= 5;
                let index = ((buffer >> bits) & 0x1F) as usize;
                result.push(alphabet.chars().nth(index).unwrap());
            }
        }

        // Handle remaining bits if any
        if bits > 0 {
            let index = ((buffer << (5 - bits)) & 0x1F) as usize;
            result.push(alphabet.chars().nth(index).unwrap());
        }

        Ok(result)
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>> {
        let alphabet = match self.engine {
            Base32Engine::Standard => "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567",
            Base32Engine::Hex => "0123456789ABCDEFGHIJKLMNOPQRSTUV",
        };

        let mut result = Vec::new();
        let mut bits = 0;
        let mut buffer = 0u16;

        for (i, c) in text.chars().enumerate() {
            let value = alphabet.find(c)
                .ok_or_else(|| error(format!("Invalid character {} at position {}", c, i)))?;

            buffer = (buffer << 5) | (value as u16);
            bits += 5;

            if bits >= 8 {
                bits -= 8;
                result.push(((buffer >> bits) & 0xFF) as u8);
            }
        }

        Ok(result)
    }
} 