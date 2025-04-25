use crate::data::uniqueid::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid as RawUuid;

/// Represents different identifier versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdVersion {
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
    /// Unknown or unsupported version
    Unknown,
}

impl IdVersion {
    /// Determines if this identifier version contains timestamp information
    pub fn has_timestamp(&self) -> bool {
        matches!(self, Self::V1 | Self::V2 | Self::V6 | Self::V7)
    }
}

/// Identifier variant as defined in RFC 4122
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdVariant {
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

/// Represents the type of unique identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdType {
    /// UUID (Universally Unique Identifier)
    Uuid(IdVersion, IdVariant),
    /// ULID (Universally Unique Lexicographically Sortable Identifier)
    Ulid,
}

impl IdType {
    /// Determines if this ID type contains timestamp information
    pub fn has_timestamp(&self) -> bool {
        match self {
            Self::Uuid(version, _) => version.has_timestamp(),
            Self::Ulid => true,
        }
    }

    /// Returns the identifier version if this is a UUID type
    pub fn uuid_version(&self) -> Option<IdVersion> {
        match self {
            Self::Uuid(version, _) => Some(*version),
            Self::Ulid => None,
        }
    }

    /// Returns the identifier variant if this is a UUID type
    pub fn uuid_variant(&self) -> Option<IdVariant> {
        match self {
            Self::Uuid(_, variant) => Some(*variant),
            Self::Ulid => None,
        }
    }
}

/// Wrapper for different unique identifier implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueId {
    /// The raw bytes of the identifier
    bytes: [u8; 16],
    /// The identifier type
    id_type: IdType,
}

impl UniqueId {
    /// Create a new UniqueId from raw bytes
    pub fn new(bytes: [u8; 16], id_type: IdType) -> Self {
        Self {
            bytes,
            id_type,
        }
    }

    /// Create a new random UUID (v4)
    pub fn new_v4() -> Self {
        let raw = RawUuid::new_v4();
        Self {
            bytes: raw.into_bytes(),
            id_type: IdType::Uuid(IdVersion::V4, IdVariant::RFC4122),
        }
    }

    /// Create a new ULID
    pub fn new_ulid() -> Self {
        let ulid = ulid::Ulid::new();
        let bytes = ulid.to_bytes();
        Self {
            bytes,
            id_type: IdType::Ulid,
        }
    }

    /// Create a new ULID with a specific timestamp
    pub fn new_ulid_with_timestamp(timestamp_ms: u64) -> Self {
        // Create a ULID with a specific timestamp
        let system_time = std::time::SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::from_millis(timestamp_ms))
            .unwrap_or_else(|| std::time::SystemTime::now());

        let ulid = ulid::Ulid::from_datetime(system_time);
        let bytes = ulid.to_bytes();
        Self {
            bytes,
            id_type: IdType::Ulid,
        }
    }

    /// Returns the raw bytes of the identifier
    pub fn bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Returns the identifier type
    pub fn id_type(&self) -> IdType {
        self.id_type
    }

    /// Returns whether this identifier is a UUID
    pub fn is_uuid(&self) -> bool {
        matches!(self.id_type, IdType::Uuid(_, _))
    }

    /// Returns whether this identifier is a ULID
    pub fn is_ulid(&self) -> bool {
        matches!(self.id_type, IdType::Ulid)
    }

    /// Returns the identifier version if this is a UUID type
    pub fn uuid_version(&self) -> Option<IdVersion> {
        self.id_type.uuid_version()
    }

    /// Returns the identifier variant if this is a UUID type
    pub fn uuid_variant(&self) -> Option<IdVariant> {
        self.id_type.uuid_variant()
    }

    /// Returns whether this identifier type contains timestamp information
    pub fn has_timestamp(&self) -> bool {
        self.id_type.has_timestamp()
    }

    /// Convert to a simple string representation (without hyphens for UUID, base32 for ULID)
    pub fn to_simple(&self) -> String {
        match self.id_type {
            IdType::Uuid(_, _) => {
                let uuid = RawUuid::from_bytes(self.bytes);
                uuid.simple().to_string()
            }
            IdType::Ulid => {
                let ulid = ulid::Ulid::from_bytes(self.bytes);
                ulid.to_string()
            }
        }
    }

    /// Convert to a hyphenated string representation (for UUID)
    /// For ULID, returns the standard ULID format
    pub fn to_hyphenated(&self) -> String {
        match self.id_type {
            IdType::Uuid(_, _) => {
                let uuid = RawUuid::from_bytes(self.bytes);
                uuid.hyphenated().to_string()
            }
            IdType::Ulid => self.to_simple(),
        }
    }

    /// Convert to a ULID string representation if possible
    pub fn to_ulid_string(&self) -> Option<String> {
        if self.is_ulid() {
            let ulid = ulid::Ulid::from_bytes(self.bytes);
            Some(ulid.to_string())
        } else {
            None
        }
    }

    /// Convert to a UUID string representation if possible
    pub fn to_uuid_string(&self) -> Option<String> {
        if self.is_uuid() {
            let uuid = RawUuid::from_bytes(self.bytes);
            Some(uuid.hyphenated().to_string())
        } else {
            None
        }
    }
}

impl fmt::Display for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id_type {
            IdType::Uuid(_, _) => {
                let uuid = RawUuid::from_bytes(self.bytes);
                write!(f, "{}", uuid.hyphenated())
            }
            IdType::Ulid => {
                let ulid = ulid::Ulid::from_bytes(self.bytes);
                write!(f, "{}", ulid)
            }
        }
    }
}

impl FromStr for UniqueId {
    type Err = crate::data::uniqueid::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Try to parse as ULID first (26 characters)
        if s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            match ulid::Ulid::from_string(s) {
                Ok(ulid) => {
                    let bytes = ulid.to_bytes();
                    return Ok(Self {
                        bytes,
                        id_type: IdType::Ulid,
                    });
                }
                Err(_) => {}  // Fall through to UUID parsing
            }
        }

        // Try to parse as UUID
        let raw = RawUuid::parse_str(s)?;

        // Determine version
        let version = match raw.get_version_num() {
            1 => IdVersion::V1,
            2 => IdVersion::V2,
            3 => IdVersion::V3,
            4 => IdVersion::V4,
            5 => IdVersion::V5,
            6 => IdVersion::V6,
            7 => IdVersion::V7,
            8 => IdVersion::V8,
            _ => IdVersion::Unknown,
        };

        // Determine variant
        let variant = match raw.get_variant() {
            uuid::Variant::RFC4122 => IdVariant::RFC4122,
            uuid::Variant::NCS => IdVariant::NCS,
            uuid::Variant::Microsoft => IdVariant::Microsoft,
            uuid::Variant::Future => IdVariant::Future,
            _ => IdVariant::Unknown,
        };

        Ok(Self {
            bytes: raw.into_bytes(),
            id_type: IdType::Uuid(version, variant),
        })
    }
} 