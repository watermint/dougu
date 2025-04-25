#[cfg(test)]
mod tests {
    use crate::data::uniqueid::{Uuid, UuidParser, UuidFormatter, UuidVersion, UuidVariant};
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
} 