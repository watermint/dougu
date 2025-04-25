#[cfg(test)]
mod tests {
    use crate::data::uniqueid::{IdFormatter, IdParser, IdTimestamp, IdType, IdVariant, IdVersion, UniqueId};
    use std::str::FromStr;

    #[test]
    fn test_create_uuid_v4() {
        let id = UniqueId::new_v4();
        assert!(id.is_uuid());
        assert_eq!(id.uuid_version(), Some(IdVersion::V4));
        assert_eq!(id.uuid_variant(), Some(IdVariant::RFC4122));
    }

    #[test]
    fn test_parse_uuid() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let id = IdParser::parse(uuid_str).unwrap();
        assert!(id.is_uuid());
        assert_eq!(id.uuid_version(), Some(IdVersion::V1));
        assert_eq!(id.uuid_variant(), Some(IdVariant::RFC4122));
    }

    #[test]
    fn test_format_uuid() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let id = IdParser::parse(uuid_str).unwrap();

        assert_eq!(IdFormatter::hyphenated(&id), uuid_str);
        assert_eq!(IdFormatter::simple(&id), "f81d4fae7dec11d0a76500a0c91e6bf6");
        assert_eq!(IdFormatter::uppercase(&id), "F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6");
        assert_eq!(IdFormatter::urn(&id).unwrap(), format!("urn:uuid:{}", uuid_str));
        assert_eq!(IdFormatter::with_separator(&id, '_').unwrap(), "f81d4fae_7dec_11d0_a765_00a0c91e6bf6");
    }

    #[test]
    fn test_from_str() {
        let uuid_str = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6";
        let id = UniqueId::from_str(uuid_str).unwrap();

        assert_eq!(id.to_string(), uuid_str);
        assert_eq!(id.uuid_version(), Some(IdVersion::V1));
    }

    #[test]
    fn test_create_ulid() {
        let id = UniqueId::new_ulid();
        assert!(id.is_ulid());
        assert_eq!(id.id_type(), IdType::Ulid);
    }

    #[test]
    fn test_create_ulid_with_timestamp() {
        let timestamp = 1656058000000; // Example timestamp: 2022-06-24T12:00:00Z
        let id = UniqueId::new_ulid_with_timestamp(timestamp);
        assert!(id.is_ulid());
        assert_eq!(id.id_type(), IdType::Ulid);

        // Extract and verify timestamp
        let dt = IdTimestamp::extract(&id).unwrap();
        assert_eq!(dt.milliseconds_since_epoch(), timestamp);
    }

    #[test]
    fn test_parse_ulid() {
        // Example ULID: 01G5APCJEVQECTPW9SX5G4V6Q9
        let ulid_str = "01G5APCJEVQECTPW9SX5G4V6Q9";
        let id = IdParser::parse(ulid_str).unwrap();

        assert!(id.is_ulid());
        assert_eq!(id.id_type(), IdType::Ulid);

        // Should format back to the same string 
        assert_eq!(id.to_ulid_string().unwrap(), ulid_str);
    }

    #[test]
    fn test_ulid_format() {
        let id = UniqueId::new_ulid();

        // Get ULID string representation
        let ulid_str = id.to_ulid_string().unwrap();

        // Should be 26 characters
        assert_eq!(ulid_str.len(), 26);

        // Should only contain base32 characters (uppercase alphanumeric excluding I, L, O, U)
        assert!(ulid_str.chars().all(|c| match c {
            '0'..='9' | 'A'..='H' | 'J'..='K' | 'M'..='N' | 'P'..='T' | 'V'..='Z' => true,
            _ => false,
        }));

        // Parse back to UniqueId
        let parsed_id = IdParser::parse(&ulid_str).unwrap();
        assert_eq!(parsed_id.bytes(), id.bytes());
    }
} 