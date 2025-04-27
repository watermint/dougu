use crate::core::error::{error, Result};
use crate::data::encoding::BinaryTextCodec;

/// UUEncode configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UUEncodeEngine {
    /// Standard UUEncode
    Standard,
}

/// UUEncode encoder/decoder
#[derive(Debug, Clone, Copy)]
pub struct UUEncode {
    engine: UUEncodeEngine,
}

impl UUEncode {
    /// Create a new UUEncode encoder/decoder with the given engine
    pub fn new(engine: UUEncodeEngine) -> Self {
        Self { engine }
    }

    /// Create a new UUEncode encoder/decoder with standard engine
    pub fn standard() -> Self {
        Self::new(UUEncodeEngine::Standard)
    }
}

impl BinaryTextCodec for UUEncode {
    fn encode(&self, data: &[u8]) -> Result<String> {
        let mut result = String::new();

        for chunk in data.chunks(45) {
            // Add length byte
            let length_char = (chunk.len() as u8 + 32) as char;
            result.push(length_char);

            // Encode 3 bytes into 4 characters
            for i in (0..chunk.len()).step_by(3) {
                let mut buffer = [0u8; 3];
                let count = std::cmp::min(3, chunk.len() - i);

                buffer[..count].copy_from_slice(&chunk[i..i + count]);

                // Process triplet
                let b1 = buffer[0];
                let b2 = if count > 1 { buffer[1] } else { 0 };
                let b3 = if count > 2 { buffer[2] } else { 0 };

                // Encode to 4 bytes
                let c1 = 32 + ((b1 >> 2) & 0x3f);
                let c2 = 32 + (((b1 & 0x03) << 4) | ((b2 >> 4) & 0x0f));
                let c3 = 32 + (((b2 & 0x0f) << 2) | ((b3 >> 6) & 0x03));
                let c4 = 32 + (b3 & 0x3f);

                result.push(c1 as char);
                result.push(c2 as char);
                if count > 1 { result.push(c3 as char); } else { result.push(' '); }
                if count > 2 { result.push(c4 as char); } else { result.push(' '); }
            }

            result.push('\n');
        }

        // Add end marker
        result.push('`');
        result.push('\n');

        Ok(result)
    }

    fn decode(&self, text: &str) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            if line == "`" {
                // End marker
                break;
            }

            if line.is_empty() {
                continue;
            }

            let mut chars = line.chars();
            let length_char = chars.next().ok_or_else(|| error("Invalid UUEncode format: missing length character"))?;
            let length = (length_char as u8).wrapping_sub(32) as usize;

            if length > 45 {
                return Err(error(format!("Invalid UUEncode length: {}", length)));
            }

            let line_chars: Vec<char> = chars.collect();
            let encoded_data: Vec<u8> = line_chars.iter().map(|&c| (c as u8).wrapping_sub(32)).collect();

            for i in (0..encoded_data.len()).step_by(4) {
                if i + 3 >= encoded_data.len() {
                    break;
                }

                let b1 = encoded_data[i];
                let b2 = encoded_data[i + 1];
                let b3 = encoded_data[i + 2];
                let b4 = encoded_data[i + 3];

                let d1 = (b1 << 2) | (b2 >> 4);
                let d2 = (b2 << 4) | (b3 >> 2);
                let d3 = (b3 << 6) | b4;

                result.push(d1);
                if result.len() < length {
                    result.push(d2);
                }
                if result.len() < length {
                    result.push(d3);
                }
            }
        }

        Ok(result)
    }
} 