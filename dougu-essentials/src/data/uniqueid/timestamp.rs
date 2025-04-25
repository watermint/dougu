// use crate::core::{Error as CoreError, Result as CoreResult}; // Unused
use crate::data::uniqueid::error::{Error, Result};
use crate::data::uniqueid::types::{IdType, IdVersion, UniqueId};
use crate::time::ZonedDateTime;
// use chrono::TimeZone; // Removed unused
// use log::error;
// use std::time::SystemTime; // Unused
use uuid::Uuid as RawUuid;
// Removed unused Builder, Timestamp, Version
// use chrono::Utc; // Removed unused Utc import

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
            }
            IdType::Ulid => Self::extract_ulid_timestamp(id),
        }
    }

    /// Extract timestamp from V1 UUID
    fn extract_v1_timestamp(id: &UniqueId) -> Result<ZonedDateTime> {
        let _raw_uuid = RawUuid::from_bytes(*id.bytes());

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
    fn extract_v6_timestamp(_id: &UniqueId) -> Result<ZonedDateTime> {
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

// Use underscore for unused variables
fn extract_v6_timestamp_bits(_id: &UniqueId) -> Result<u64> {
    // error!("UUID version 6 timestamp extraction not yet implemented"); // Just return error
    Err(Error::TimestampExtraction("V6 timestamp bit extraction not implemented".to_string()))
}

// Use underscore for unused variables
fn extract_v6_timestamp(_id: &UniqueId) -> Result<ZonedDateTime> {
    // error!("UUID version 6 timestamp extraction not yet implemented"); // Just return error
    Err(Error::TimestampExtraction("V6 timestamp extraction not implemented".to_string()))
}

#[cfg(test)]
mod tests {
    // Import the struct with the extract method
    use crate::data::uniqueid::types::{IdType, IdVersion, UniqueId};
    use crate::data::uniqueid::IdTimestamp;
    use crate::time::Instant as TimeInstant;
    // Import correct Duration
    use crate::time::ZonedDateTime;
    // Import needed types
    use crate::IdVariant;
    // Import IdVariant as suggested
    use std::thread;
    use std::time::Duration;
    // Import Instant trait

    #[test]
    fn test_extract_v1_timestamp_unimplemented() {
        // V1 timestamp extraction is not fully implemented
        let id = UniqueId::new([0u8; 16], IdType::Uuid(IdVersion::V1, IdVariant::RFC4122)); // Create dummy V1 ID
        let result = IdTimestamp::extract(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("V1 timestamp extraction"));
    }

    #[test]
    fn test_extract_v6_timestamp_unimplemented() {
        // V6 timestamp extraction is not fully implemented
        let id = UniqueId::new([0u8; 16], IdType::Uuid(IdVersion::V6, IdVariant::RFC4122)); // Create dummy V6 ID
        let result = IdTimestamp::extract(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("V6 timestamp extraction"));
    }

    #[test]
    fn test_extract_v7_timestamp() {
        // V7 IDs are not generated by default UniqueId methods, so we'll create one manually
        // Example from RFC draft: 017F22E2-79B0-7CC3-98C4-DC0C0C07398F
        // Timestamp: 2022-02-02 02:02:02.000 UTC (1643767322000 ms)
        let timestamp_ms: u64 = 1643767322000;
        let mut bytes = [0u8; 16];
        // Put timestamp in first 48 bits (6 bytes)
        bytes[0..6].copy_from_slice(&timestamp_ms.to_be_bytes()[2..8]);
        // Set version (7) in bits 48-51
        bytes[6] = (bytes[6] & 0x0F) | (7 << 4);
        // Set variant (RFC4122) in bits 64-65
        bytes[8] = (bytes[8] & 0x3F) | 0x80;

        let id = UniqueId::new(bytes, IdType::Uuid(IdVersion::V7, IdVariant::RFC4122));

        let before = ZonedDateTime::of_unix_nanos(1643767321, 999_000_000).unwrap();
        let after = ZonedDateTime::of_unix_nanos(1643767322, 001_000_000).unwrap();

        let result = IdTimestamp::extract(&id);
        assert!(result.is_ok());
        let ts = result.unwrap();

        // Check if the extracted timestamp is very close to the original
        assert!(ts >= before && ts <= after, "Timestamp {:?} is not between {:?} and {:?}", ts, before, after);
        assert_eq!(TimeInstant::get_epoch_milli(&ts), timestamp_ms as i64);
    }

    #[test]
    fn test_extract_ulid_timestamp() {
        let before = ZonedDateTime::now();
        thread::sleep(Duration::from_millis(10));
        let id = UniqueId::new_ulid(); // Use constructor from UniqueId
        thread::sleep(Duration::from_millis(10));
        let after = ZonedDateTime::now();

        let result = IdTimestamp::extract(&id);
        assert!(result.is_ok());
        let ts = result.unwrap();

        // Check if the extracted timestamp is within the expected range
        assert!(ts >= before && ts <= after, "Timestamp {:?} is not between {:?} and {:?}", ts, before, after);
    }
} 