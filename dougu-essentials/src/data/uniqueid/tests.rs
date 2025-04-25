#[cfg(test)]
mod tests {
    use crate::data::uniqueid::{Uuid, UuidParser, UuidFormatter, UuidVersion, UuidVariant, UuidTimestamp};
    use std::str::FromStr;

    #[test]
    fn test_create_uuid_v4() {
        let uuid = Uuid::new_v4();
        assert_eq!(uuid.version(), UuidVersion::V4);
        assert_eq!(uuid.variant(), UuidVariant::RFC4122);
    }

    #[test]
    fn test_parse_uuid() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let uuid = UuidParser::parse(uuid_str).unwrap();
        assert_eq!(uuid.version(), UuidVersion::V1);
        assert_eq!(uuid.variant(), UuidVariant::RFC4122);
    }

    #[test]
    fn test_format_uuid() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let uuid = UuidParser::parse(uuid_str).unwrap();
        
        assert_eq!(UuidFormatter::hyphenated(&uuid), uuid_str);
        assert_eq!(UuidFormatter::simple(&uuid), "f81d4fae7dec11d0a76500a0c91e6bf6");
        assert_eq!(UuidFormatter::uppercase(&uuid), "F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6");
        assert_eq!(UuidFormatter::urn(&uuid), format!("urn:uuid:{}", uuid_str));
        assert_eq!(UuidFormatter::with_separator(&uuid, '_'), "f81d4fae_7dec_11d0_a765_00a0c91e6bf6");
    }

    #[test]
    fn test_from_str() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let uuid = Uuid::from_str(uuid_str).unwrap();
        
        assert_eq!(uuid.to_string(), uuid_str);
        assert_eq!(uuid.version(), UuidVersion::V1);
    }
    
    #[test]
    fn test_create_ulid() {
        let uuid = Uuid::new_ulid();
        assert_eq!(uuid.version(), UuidVersion::Ulid);
        assert_eq!(uuid.variant(), UuidVariant::RFC4122);
    }
    
    #[test]
    fn test_create_ulid_with_timestamp() {
        let timestamp = 1656058000000; // Example timestamp: 2022-06-24T12:00:00Z
        let uuid = Uuid::new_ulid_with_timestamp(timestamp);
        assert_eq!(uuid.version(), UuidVersion::Ulid);
        assert_eq!(uuid.variant(), UuidVariant::RFC4122);
        
        // Extract and verify timestamp
        let dt = UuidTimestamp::extract(&uuid).unwrap();
        assert_eq!(dt.milliseconds_since_epoch(), timestamp);
    }
    
    #[test]
    fn test_parse_ulid() {
        // Example ULID: 01G5APCJEVQECTPW9SX5G4V6Q9
        let ulid_str = "01G5APCJEVQECTPW9SX5G4V6Q9";
        let uuid = UuidParser::parse(ulid_str).unwrap();
        
        assert_eq!(uuid.version(), UuidVersion::Ulid);
        assert_eq!(uuid.variant(), UuidVariant::RFC4122);
        
        // Should format back to the same string when using ulid formatter
        assert_eq!(UuidFormatter::ulid(&uuid).unwrap(), ulid_str);
    }
    
    #[test]
    fn test_ulid_format() {
        let uuid = Uuid::new_ulid();
        
        // Get ULID string representation
        let ulid_str = UuidFormatter::ulid(&uuid).unwrap();
        
        // Should be 26 characters
        assert_eq!(ulid_str.len(), 26);
        
        // Should only contain base32 characters (uppercase alphanumeric excluding I, L, O, U)
        assert!(ulid_str.chars().all(|c| match c {
            '0'..='9' | 'A'..='H' | 'J'..='K' | 'M'..='N' | 'P'..='T' | 'V'..='Z' => true,
            _ => false,
        }));
        
        // Parse back to UUID
        let parsed_uuid = UuidParser::parse(&ulid_str).unwrap();
        assert_eq!(parsed_uuid.bytes(), uuid.bytes());
    }
} 