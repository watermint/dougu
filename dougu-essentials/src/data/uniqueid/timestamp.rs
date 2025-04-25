use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{UniqueId, IdVersion, IdType};
use crate::time::{ZonedDateTime, TimeError};
use uuid::Uuid as RawUuid;

/// Helper for extracting timestamps from unique identifiers
#[derive(Debug, Clone, Copy)]
pub struct IdTimestamp;

impl IdTimestamp {
    /// Extract timestamp from a unique identifier if available
    /// 
    /// Only works with identifier types that contain timestamp information:
    /// - UUID V1: Time-based UUID
    /// - UUID V2: DCE Security UUID
    /// - UUID V6: Reordered time-based UUID
    /// - UUID V7: Time-ordered UUID with Unix timestamp
    /// - ULID: Universally Unique Lexicographically Sortable Identifier
    pub fn extract(id: &UniqueId) -> Result<ZonedDateTime> {
        if !id.has_timestamp() {
            return Err(Error::TimestampExtraction(format!(
                "Identifier type {:?} does not contain timestamp information",
                id.id_type()
            )));
        }

        match id.id_type() {
            IdType::Uuid(version, _) => {
                match version {
                    IdVersion::V1 | IdVersion::V2 => Self::extract_v1_timestamp(id),
                    IdVersion::V6 => Self::extract_v6_timestamp(id),
                    IdVersion::V7 => Self::extract_v7_timestamp(id),
                    _ => Err(Error::TimestampExtraction(
                        "Timestamp extraction not implemented for this UUID version".to_string(),
                    )),
                }
            },
            IdType::Ulid => Self::extract_ulid_timestamp(id),
        }
    }

    /// Extract timestamp from V1 UUID
    fn extract_v1_timestamp(id: &UniqueId) -> Result<ZonedDateTime> {
        let raw_uuid = RawUuid::from_bytes(*id.bytes());
        
        // The uuid crate doesn't expose timestamp extraction directly,
        // so we would need to implement the extraction logic manually
        
        // For V1 UUIDs, the timestamp is spread across several fields:
        // - time_low (32 bits)
        // - time_mid (16 bits)
        // - time_high_and_version (16 bits, with the high 4 bits being the version)
        // Together they form a 60-bit timestamp in 100-nanosecond intervals since 15 October 1582
        
        // This is a simplified implementation for demonstration
        // A real implementation would extract and combine the timestamp fields
        
        Err(Error::TimestampExtraction(
            "V1 timestamp extraction requires detailed byte manipulation - not fully implemented".to_string(),
        ))
    }

    /// Extract timestamp from V6 UUID
    fn extract_v6_timestamp(id: &UniqueId) -> Result<ZonedDateTime> {
        // V6 is similar to V1 but with the timestamp fields reordered for better sorting
        Err(Error::TimestampExtraction(
            "V6 timestamp extraction not fully implemented".to_string(),
        ))
    }

    /// Extract timestamp from V7 UUID
    fn extract_v7_timestamp(id: &UniqueId) -> Result<ZonedDateTime> {
        // V7 contains a Unix timestamp in milliseconds in the first 48 bits
        let bytes = id.bytes();
        
        // Extract the first 48 bits (6 bytes) which contain the Unix timestamp in milliseconds
        let msec = ((bytes[0] as u64) << 40)
            | ((bytes[1] as u64) << 32)
            | ((bytes[2] as u64) << 24)
            | ((bytes[3] as u64) << 16)
            | ((bytes[4] as u64) << 8)
            | (bytes[5] as u64);
        
        // Convert milliseconds to seconds and nanoseconds
        let secs = (msec / 1000) as i64;
        let nsecs = ((msec % 1000) * 1_000_000) as u32;
        
        // Create ZonedDateTime from timestamp
        ZonedDateTime::of_unix_nanos(secs, nsecs).map_err(|e| {
            Error::TimestampExtraction(format!("Invalid timestamp value in V7 UUID: {}", e))
        })
    }
    
    /// Extract timestamp from ULID
    fn extract_ulid_timestamp(id: &UniqueId) -> Result<ZonedDateTime> {
        let bytes = id.bytes();
        let ulid = ulid::Ulid::from_bytes(*bytes);
        
        // Get the timestamp in milliseconds
        let msec = ulid.timestamp_ms();
        
        // Convert milliseconds to seconds and nanoseconds
        let secs = (msec / 1000) as i64;
        let nsecs = ((msec % 1000) * 1_000_000) as u32;
        
        // Create ZonedDateTime from timestamp
        ZonedDateTime::of_unix_nanos(secs, nsecs).map_err(|e| {
            Error::TimestampExtraction(format!("Invalid timestamp value in ULID: {}", e))
        })
    }
} 