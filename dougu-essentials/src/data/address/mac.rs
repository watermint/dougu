use super::error::{AddressError, Result};
use crate::obj::notation::{NotationType, NumberVariant};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MacAddress {
    address: String,
    bytes: [u8; 6],
}

impl MacAddress {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address_str = address.into();

        if !Self::is_valid(&address_str) {
            return Err(AddressError::InvalidMacFormat(address_str));
        }

        // Normalize the address to a consistent format (xx:xx:xx:xx:xx:xx)
        let normalized = Self::normalize(&address_str);

        // Parse the normalized address into bytes
        let bytes = Self::parse_bytes(&normalized)?;

        Ok(Self {
            address: normalized,
            bytes,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn bytes(&self) -> [u8; 6] {
        self.bytes
    }

    pub fn is_valid(address: &str) -> bool {
        // Check for common MAC address formats
        let is_colon_format = Self::validate_format(address, ':', 17);
        let is_hyphen_format = Self::validate_format(address, '-', 17);
        let is_dot_format = Self::validate_format(address, '.', 14);
        let is_no_separator_format = Self::validate_no_separator_format(address);

        is_colon_format || is_hyphen_format || is_dot_format || is_no_separator_format
    }

    fn validate_format(address: &str, separator: char, expected_len: usize) -> bool {
        if address.len() != expected_len {
            return false;
        }

        let parts: Vec<&str> = address.split(separator).collect();
        if separator == '.' && parts.len() != 3 {
            return false;
        } else if separator != '.' && parts.len() != 6 {
            return false;
        }

        for part in parts {
            if separator == '.' {
                // For dot format (xxxx.xxxx.xxxx)
                if part.len() != 4 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
                    return false;
                }
            } else {
                // For colon or hyphen format (xx:xx:xx:xx:xx:xx or xx-xx-xx-xx-xx-xx)
                if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
                    return false;
                }
            }
        }

        true
    }

    fn validate_no_separator_format(address: &str) -> bool {
        // For format with no separator (xxxxxxxxxxxx)
        if address.len() != 12 {
            return false;
        }

        address.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn normalize(address: &str) -> String {
        if address.contains(':') {
            // Already in colon format
            return address.to_lowercase();
        } else if address.contains('-') {
            // Convert from hyphen format to colon format
            return address.replace('-', ":").to_lowercase();
        } else if address.contains('.') {
            // Convert from dot format (xxxx.xxxx.xxxx) to colon format
            let parts: Vec<&str> = address.split('.').collect();
            let mut result = String::new();

            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    result.push(':');
                }
                result.push_str(&part[0..2]);
                result.push(':');
                result.push_str(&part[2..4]);
            }

            return result.to_lowercase();
        } else {
            // Convert from no separator format to colon format
            let mut result = String::new();
            let chars: Vec<char> = address.chars().collect();

            for i in 0..6 {
                if i > 0 {
                    result.push(':');
                }
                result.push(chars[i * 2]);
                result.push(chars[i * 2 + 1]);
            }

            return result.to_lowercase();
        }
    }

    fn parse_bytes(normalized: &str) -> Result<[u8; 6]> {
        let mut bytes = [0u8; 6];
        let parts: Vec<&str> = normalized.split(':').collect();

        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| AddressError::InvalidMacFormat(normalized.to_string()))?;
        }

        Ok(bytes)
    }

    pub fn is_broadcast(&self) -> bool {
        // MAC broadcast address is FF:FF:FF:FF:FF:FF
        self.bytes == [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    }

    pub fn is_multicast(&self) -> bool {
        // The least significant bit of the first byte is set
        (self.bytes[0] & 0x01) == 0x01
    }

    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    pub fn is_locally_administered(&self) -> bool {
        // The second least significant bit of the first byte is set
        (self.bytes[0] & 0x02) == 0x02
    }

    pub fn is_universal(&self) -> bool {
        !self.is_locally_administered()
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for MacAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        MacAddress::new(s)
    }
}

impl From<MacAddress> for NotationType {
    fn from(mac: MacAddress) -> Self {
        let mut obj = HashMap::new();

        obj.insert("address".to_string(), NotationType::String(mac.address));
        obj.insert("bytes".to_string(), NotationType::Array(
            mac.bytes.iter().map(|&b| NotationType::Number(NumberVariant::Uint(b as u64))).collect()
        ));

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for MacAddress {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => MacAddress::new(s),
            NotationType::Object(obj) => {
                if let Some(NotationType::String(addr)) = obj.get("address") {
                    MacAddress::new(addr)
                } else if let Some(NotationType::Array(bytes_arr)) = obj.get("bytes") {
                    if bytes_arr.len() != 6 {
                        return Err(AddressError::InvalidMacFormat("MAC address must have 6 bytes".to_string()));
                    }

                    let mut bytes = [0u8; 6];
                    for (i, byte_value) in bytes_arr.iter().enumerate() {
                        let byte = match byte_value {
                            NotationType::Number(NumberVariant::Uint(n)) => {
                                if *n > 255 {
                                    return Err(AddressError::InvalidMacFormat("Byte value out of range".to_string()));
                                }
                                *n as u8
                            }
                            NotationType::Number(NumberVariant::Int(n)) => {
                                if *n < 0 || *n > 255 {
                                    return Err(AddressError::InvalidMacFormat("Byte value out of range".to_string()));
                                }
                                *n as u8
                            }
                            _ => return Err(AddressError::InvalidMacFormat("Invalid byte value".to_string())),
                        };
                        bytes[i] = byte;
                    }

                    // Format the MAC address as a string
                    let address = format!(
                        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
                    );

                    Ok(MacAddress { address, bytes })
                } else {
                    Err(AddressError::InvalidMacFormat("Missing address or bytes field".to_string()))
                }
            }
            _ => Err(AddressError::InvalidMacFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mac() {
        assert!(MacAddress::is_valid("00:11:22:33:44:55"));
        assert!(MacAddress::is_valid("00-11-22-33-44-55"));
        assert!(MacAddress::is_valid("0011.2233.4455"));
        assert!(MacAddress::is_valid("001122334455"));
        assert!(MacAddress::is_valid("FF:FF:FF:FF:FF:FF"));
        assert!(MacAddress::is_valid("ff:ff:ff:ff:ff:ff"));
    }

    #[test]
    fn test_invalid_mac() {
        assert!(!MacAddress::is_valid(""));
        assert!(!MacAddress::is_valid("00:11:22:33:44")); // Too short
        assert!(!MacAddress::is_valid("00:11:22:33:44:55:66")); // Too long
        assert!(!MacAddress::is_valid("00:11:22:33:44:GG")); // Invalid hex
        assert!(!MacAddress::is_valid("00:112:22:33:44:55")); // Invalid segment length
    }

    #[test]
    fn test_normalize() {
        let mac1 = MacAddress::new("00:11:22:33:44:55").unwrap();
        assert_eq!(mac1.address(), "00:11:22:33:44:55");

        let mac2 = MacAddress::new("00-11-22-33-44-55").unwrap();
        assert_eq!(mac2.address(), "00:11:22:33:44:55");

        let mac3 = MacAddress::new("0011.2233.4455").unwrap();
        assert_eq!(mac3.address(), "00:11:22:33:44:55");

        let mac4 = MacAddress::new("001122334455").unwrap();
        assert_eq!(mac4.address(), "00:11:22:33:44:55");
    }

    #[test]
    fn test_mac_properties() {
        let mac_broadcast = MacAddress::new("FF:FF:FF:FF:FF:FF").unwrap();
        assert!(mac_broadcast.is_broadcast());
        assert!(mac_broadcast.is_multicast());

        let mac_multicast = MacAddress::new("01:00:5E:00:00:01").unwrap();
        assert!(!mac_multicast.is_broadcast());
        assert!(mac_multicast.is_multicast());
        assert!(!mac_multicast.is_unicast());

        let mac_unicast = MacAddress::new("00:11:22:33:44:55").unwrap();
        assert!(!mac_unicast.is_broadcast());
        assert!(!mac_unicast.is_multicast());
        assert!(mac_unicast.is_unicast());

        let mac_local = MacAddress::new("02:11:22:33:44:55").unwrap();
        assert!(mac_local.is_locally_administered());
        assert!(!mac_local.is_universal());

        let mac_universal = MacAddress::new("00:11:22:33:44:55").unwrap();
        assert!(!mac_universal.is_locally_administered());
        assert!(mac_universal.is_universal());
    }

    #[test]
    fn test_serialization() {
        let mac = MacAddress::new("00:11:22:33:44:55").unwrap();
        let notation = NotationType::from(mac.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("address").and_then(|v| v.as_str()), Some("00:11:22:33:44:55"));

            if let Some(NotationType::Array(bytes)) = obj.get("bytes") {
                assert_eq!(bytes.len(), 6);
                assert_eq!(bytes[0].as_u64(), Some(0));
                assert_eq!(bytes[1].as_u64(), Some(17));
                assert_eq!(bytes[2].as_u64(), Some(34));
                assert_eq!(bytes[3].as_u64(), Some(51));
                assert_eq!(bytes[4].as_u64(), Some(68));
                assert_eq!(bytes[5].as_u64(), Some(85));
            } else {
                panic!("Expected bytes array");
            }
        }

        let mac_back = MacAddress::try_from(notation);
        assert!(mac_back.is_ok());
        assert_eq!(mac_back.unwrap().address(), mac.address());
    }
} 