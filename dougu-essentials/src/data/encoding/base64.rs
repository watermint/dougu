use crate::core::error::{error, Result};
use crate::data::encoding::BinaryTextCodec;

/// Base64 encoding configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64Engine {
    /// Standard Base64 encoding (RFC 4648)
    Standard,
    /// URL-safe Base64 encoding (RFC 4648 Section 5)
    UrlSafe,
}

/// Base64 encoder/decoder
#[derive(Debug, Clone, Copy)]
pub struct Base64 {
    engine: Base64Engine,
}

impl Base64 {
    /// Create a new Base64 encoder/decoder with the given engine
    pub fn new(engine: Base64Engine) -> Self {
        Self { engine }
    }

    /// Create a new Base64 encoder/decoder with standard engine
    pub fn standard() -> Self {
        Self::new(Base64Engine::Standard)
    }

    /// Create a new Base64 encoder/decoder with URL-safe engine
    pub fn url_safe() -> Self {
        Self::new(Base64Engine::UrlSafe)
    }
}

impl BinaryTextCodec for Base64 {
    fn encode(&self, data: &[u8]) -> Result<String> {
        match self.engine {
            Base64Engine::Standard => Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)),
            Base64Engine::UrlSafe => Ok(base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE, data)),
        }
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>> {
        let result = match self.engine {
            Base64Engine::Standard => base64::Engine::decode(&base64::engine::general_purpose::STANDARD, text),
            Base64Engine::UrlSafe => base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, text),
        };

        match result {
            Ok(data) => Ok(data),
            Err(e) => Err(error(format!("Failed to decode Base64: {}", e))),
        }
    }
} 