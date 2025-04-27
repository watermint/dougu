mod base32;
mod base64;
mod hex;
mod uuencode;
#[cfg(test)]
mod tests;

pub use base32::{Base32, Base32Engine};
pub use base64::{Base64, Base64Engine};
pub use hex::{Hex, HexEngine};
pub use uuencode::{UUEncode, UUEncodeEngine};

use crate::core::error::Result;

/// Common trait for binary-text encoders/decoders
pub trait BinaryTextCodec {
    /// Encode binary data to text
    fn encode(&self, data: &[u8]) -> Result<String>;

    /// Decode text data to binary
    fn decode(&self, text: &str) -> Result<Vec<u8>>;
} 