use crate::data::uuid::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid as RawUuid;

/// Represents different UUID versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UuidVersion {
    /// Version 1: Time-based UUID
    V1,
    /// Version 2: DCE Security UUID
    V2,
    /// Version 3: Name-based UUID using MD5 hashing
    V3,
    /// Version 4: Random UUID
    V4,
    /// Version 5: Name-based UUID using SHA-1 hashing
    V5,
    /// Version 6: Reordered time-based UUID
    V6,
    /// Version 7: Time-ordered UUID with Unix timestamp
    V7,
    /// Version 8: Custom UUID
    V8,
    /// ULID (Universally Unique Lexicographically Sortable Identifier)
    Ulid,
    /// Unknown or unsupported version
    Unknown,
}

impl UuidVersion {
    /// Determines if this UUID version contains timestamp information
    pub fn has_timestamp(&self) -> bool {
        matches!(self, Self::V1 | Self::V2 | Self::V6 | Self::V7 | Self::Ulid)
    }
}

/// UUID variant as defined in RFC 4122
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UuidVariant {
    /// Reserved by the NCS for backward compatibility
    NCS,
    /// RFC 4122 variant
    RFC4122,
    /// Reserved by Microsoft for backward compatibility
    Microsoft,
    /// Reserved for future definition
    Future,
    /// Unknown or unsupported variant
    Unknown,
}

/// Wrapper for different UUID implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Uuid {
    /// The raw bytes of the UUID
    bytes: [u8; 16],
    /// The UUID version
    version: UuidVersion,
    /// The UUID variant
    variant: UuidVariant,
}

impl Uuid {
    /// Create a new UUID from raw bytes
    pub fn new(bytes: [u8; 16], version: UuidVersion, variant: UuidVariant) -> Self {
        Self {
            bytes,
            version,
            variant,
        }
    }

    /// Create a new random UUID (v4)
    pub fn new_v4() -> Self {
        let raw = RawUuid::new_v4();
        Self {
            bytes: raw.into_bytes(),
            version: UuidVersion::V4,
            variant: UuidVariant::RFC4122,
        }
    }

    /// Returns the raw bytes of the UUID
    pub fn bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Returns the UUID version
    pub fn version(&self) -> UuidVersion {
        self.version
    }

    /// Returns the UUID variant
    pub fn variant(&self) -> UuidVariant {
        self.variant
    }

    /// Returns whether this UUID version contains timestamp information
    pub fn has_timestamp(&self) -> bool {
        self.version.has_timestamp()
    }

    /// Convert to a simple string representation
    pub fn to_simple(&self) -> String {
        self.to_string().replace('-', "")
    }

    /// Convert to a hyphenated string representation
    pub fn to_hyphenated(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert bytes to standard UUID format
        let uuid = RawUuid::from_bytes(self.bytes);
        write!(f, "{}", uuid.hyphenated())
    }
}

impl FromStr for Uuid {
    type Err = crate::data::uuid::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        let raw = RawUuid::parse_str(s)?;
        
        // Determine version
        let version = match raw.get_version_num() {
            1 => UuidVersion::V1,
            2 => UuidVersion::V2,
            3 => UuidVersion::V3,
            4 => UuidVersion::V4,
            5 => UuidVersion::V5,
            6 => UuidVersion::V6,
            7 => UuidVersion::V7,
            8 => UuidVersion::V8,
            _ => UuidVersion::Unknown,
        };
        
        // Determine variant
        let variant = match raw.get_variant() {
            uuid::Variant::RFC4122 => UuidVariant::RFC4122,
            uuid::Variant::NCS => UuidVariant::NCS,
            uuid::Variant::Microsoft => UuidVariant::Microsoft,
            uuid::Variant::Future => UuidVariant::Future,
            _ => UuidVariant::Unknown,
        };
        
        Ok(Self {
            bytes: raw.into_bytes(),
            version,
            variant,
        })
    }
} 