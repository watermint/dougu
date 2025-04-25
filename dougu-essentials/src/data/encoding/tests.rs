#[cfg(test)]
mod tests {
    use crate::data::encoding::{
        Base32, Base64, BinaryTextCodec, Hex, UUEncode,
    };

    #[test]
    fn test_base64_standard() {
        let codec = Base64::standard();
        let data = b"Hello, World!";

        let encoded = codec.encode(data).unwrap();
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_url_safe() {
        let codec = Base64::url_safe();
        let data = b"Hello, World! This+is/a=test";

        let encoded = codec.encode(data).unwrap();
        // URL-safe encoding should not contain '+', '/' or '='
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_lower() {
        let codec = Hex::lower();
        let data = b"\x01\x02\x03\xAB\xCD\xEF";

        let encoded = codec.encode(data).unwrap();
        assert_eq!(encoded, "010203abcdef");

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_upper() {
        let codec = Hex::upper();
        let data = b"\x01\x02\x03\xAB\xCD\xEF";

        let encoded = codec.encode(data).unwrap();
        assert_eq!(encoded, "010203ABCDEF");

        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_uuencode() {
        let codec = UUEncode::standard();
        let data = b"Hello, World!";

        let encoded = codec.encode(data).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base32_standard() {
        let codec = Base32::standard();
        let data = b"Hello, World!";

        let encoded = codec.encode(data).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base32_hex() {
        let codec = Base32::hex();
        let data = b"Hello, World!";

        let encoded = codec.encode(data).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded, data);
    }
} 