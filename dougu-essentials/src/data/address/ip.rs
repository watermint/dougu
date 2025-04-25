use super::error::{AddressError, Result};
use crate::obj::notation::{NotationType, NumberVariant};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IpAddress {
    V4(Ipv4Address),
    V6(Ipv6Address),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ipv4Address {
    address: String,
    octets: [u8; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ipv6Address {
    address: String,
    segments: [u16; 8],
}

impl IpAddress {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address_str = address.into();

        if Ipv4Address::is_valid(&address_str) {
            Ok(IpAddress::V4(Ipv4Address::new(address_str)?))
        } else if Ipv6Address::is_valid(&address_str) {
            Ok(IpAddress::V6(Ipv6Address::new(address_str)?))
        } else {
            Err(AddressError::InvalidIpFormat(address_str))
        }
    }

    pub fn address(&self) -> &str {
        match self {
            IpAddress::V4(addr) => addr.address(),
            IpAddress::V6(addr) => addr.address(),
        }
    }

    pub fn is_v4(&self) -> bool {
        matches!(self, IpAddress::V4(_))
    }

    pub fn is_v6(&self) -> bool {
        matches!(self, IpAddress::V6(_))
    }

    pub fn is_loopback(&self) -> bool {
        match self {
            IpAddress::V4(addr) => addr.is_loopback(),
            IpAddress::V6(addr) => addr.is_loopback(),
        }
    }

    pub fn is_private(&self) -> bool {
        match self {
            IpAddress::V4(addr) => addr.is_private(),
            IpAddress::V6(addr) => addr.is_private(),
        }
    }
}

impl Ipv4Address {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address_str = address.into();

        if !Self::is_valid(&address_str) {
            return Err(AddressError::InvalidIpFormat(address_str));
        }

        // Parse octets
        let octets: Result<[u8; 4]> = address_str
            .split('.')
            .map(|s| s.parse::<u8>().map_err(|_| AddressError::InvalidIpFormat(address_str.clone())))
            .collect::<std::result::Result<Vec<u8>, _>>()
            .map(|v| {
                let arr: [u8; 4] = [v[0], v[1], v[2], v[3]];
                arr
            });

        Ok(Self {
            address: address_str,
            octets: octets?,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn octets(&self) -> [u8; 4] {
        self.octets
    }

    pub fn is_valid(address: &str) -> bool {
        // Basic IPv4 validation
        let parts: Vec<&str> = address.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        for part in parts {
            // Check that the part is a valid number
            if let Ok(num) = part.parse::<u8>() {
                // Leading zeros are not allowed (except the number 0)
                if part.len() > 1 && part.starts_with('0') {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn is_loopback(&self) -> bool {
        self.octets[0] == 127
    }

    pub fn is_private(&self) -> bool {
        // RFC 1918 private IP ranges
        (self.octets[0] == 10) ||
            (self.octets[0] == 172 && (self.octets[1] >= 16 && self.octets[1] <= 31)) ||
            (self.octets[0] == 192 && self.octets[1] == 168)
    }
}

impl Ipv6Address {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address_str = address.into();

        if !Self::is_valid(&address_str) {
            return Err(AddressError::InvalidIpFormat(address_str));
        }

        // Normalize the address (expand :: shorthand)
        let normalized = Self::normalize(&address_str);

        // Parse segments
        let segments: Result<[u16; 8]> = normalized
            .split(':')
            .map(|s| u16::from_str_radix(s, 16).map_err(|_| AddressError::InvalidIpFormat(address_str.clone())))
            .collect::<std::result::Result<Vec<u16>, _>>()
            .map(|v| {
                let arr: [u16; 8] = [v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
                arr
            });

        Ok(Self {
            address: address_str,
            segments: segments?,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn segments(&self) -> [u16; 8] {
        self.segments
    }

    pub fn is_valid(address: &str) -> bool {
        // Basic IPv6 validation

        // Check for invalid characters
        if !address.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
            return false;
        }

        // Count colons
        let colon_count = address.chars().filter(|&c| c == ':').count();
        if colon_count < 2 || colon_count > 8 {
            return false;
        }

        // Check for :: shorthand
        let double_colon_count = address.matches("::").count();
        if double_colon_count > 1 {
            return false;
        }

        // Check for valid segments
        let parts: Vec<&str> = address.split(':').collect();

        for part in parts {
            // Empty part is valid in :: shorthand
            if part.is_empty() {
                continue;
            }

            // Each part must be a valid hex number of max 4 digits
            if part.len() > 4 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
                return false;
            }
        }

        true
    }

    fn normalize(address: &str) -> String {
        if !address.contains("::") {
            return address.to_string();
        }

        // Count how many segments are present
        let parts: Vec<&str> = address.split(':').collect();
        let existing_segments = parts.iter().filter(|&&p| !p.is_empty()).count();
        let missing_segments = 8 - existing_segments;

        // Replace :: with the appropriate number of zeros
        address.replace("::", &format!(":{}", "0:".repeat(missing_segments)))
    }

    pub fn is_loopback(&self) -> bool {
        // ::1 is the loopback address
        self.segments[0] == 0 && self.segments[1] == 0 && self.segments[2] == 0 &&
            self.segments[3] == 0 && self.segments[4] == 0 && self.segments[5] == 0 &&
            self.segments[6] == 0 && self.segments[7] == 1
    }

    pub fn is_private(&self) -> bool {
        // fc00::/7 is the private address range for IPv6
        (self.segments[0] & 0xfe00) == 0xfc00
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddress::V4(addr) => write!(f, "{}", addr),
            IpAddress::V6(addr) => write!(f, "{}", addr),
        }
    }
}

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl fmt::Display for Ipv6Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for IpAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        IpAddress::new(s)
    }
}

impl From<IpAddress> for NotationType {
    fn from(ip: IpAddress) -> Self {
        let mut obj = HashMap::new();

        obj.insert("address".to_string(), NotationType::String(ip.address().to_string()));

        match ip {
            IpAddress::V4(v4) => {
                obj.insert("type".to_string(), NotationType::String("ipv4".to_string()));
                let octets = v4.octets();
                obj.insert("octets".to_string(), NotationType::Array(
                    octets.iter().map(|&o| NotationType::Number(NumberVariant::Uint(o as u64))).collect()
                ));
            }
            IpAddress::V6(v6) => {
                obj.insert("type".to_string(), NotationType::String("ipv6".to_string()));
                let segments = v6.segments();
                obj.insert("segments".to_string(), NotationType::Array(
                    segments.iter().map(|&s| NotationType::Number(NumberVariant::Uint(s as u64))).collect()
                ));
            }
        }

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for IpAddress {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => IpAddress::new(s),
            NotationType::Object(obj) => {
                if let Some(NotationType::String(addr)) = obj.get("address") {
                    IpAddress::new(addr)
                } else {
                    Err(AddressError::InvalidIpFormat("Missing address field in object".to_string()))
                }
            }
            _ => Err(AddressError::InvalidIpFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ipv4() {
        assert!(Ipv4Address::is_valid("192.168.1.1"));
        assert!(Ipv4Address::is_valid("127.0.0.1"));
        assert!(Ipv4Address::is_valid("0.0.0.0"));
        assert!(Ipv4Address::is_valid("255.255.255.255"));
    }

    #[test]
    fn test_invalid_ipv4() {
        assert!(!Ipv4Address::is_valid(""));
        assert!(!Ipv4Address::is_valid("192.168.1"));
        assert!(!Ipv4Address::is_valid("192.168.1.256"));
        assert!(!Ipv4Address::is_valid("192.168.01.1")); // Leading zero
        assert!(!Ipv4Address::is_valid("a.b.c.d"));
    }

    #[test]
    fn test_valid_ipv6() {
        assert!(Ipv6Address::is_valid("2001:db8::1"));
        assert!(Ipv6Address::is_valid("::1"));
        assert!(Ipv6Address::is_valid("::"));
        assert!(Ipv6Address::is_valid("2001:db8:85a3:0:0:8a2e:370:7334"));
        assert!(Ipv6Address::is_valid("2001:db8:85a3::8a2e:370:7334"));
    }

    #[test]
    fn test_invalid_ipv6() {
        assert!(!Ipv6Address::is_valid(""));
        assert!(!Ipv6Address::is_valid("2001::db8::1")); // Multiple ::
        assert!(!Ipv6Address::is_valid("2001:db8:85a3:0:0:8a2e:370:7334:abcd")); // Too many segments
        assert!(!Ipv6Address::is_valid("g:0:0:0:0:0:0:0")); // Invalid hex
    }

    #[test]
    fn test_ip_properties() {
        let loopback4 = IpAddress::new("127.0.0.1").unwrap();
        assert!(loopback4.is_v4());
        assert!(loopback4.is_loopback());

        let loopback6 = IpAddress::new("::1").unwrap();
        assert!(loopback6.is_v6());
        assert!(loopback6.is_loopback());

        let private4 = IpAddress::new("192.168.1.1").unwrap();
        assert!(private4.is_private());

        let private6 = IpAddress::new("fc00::1").unwrap();
        assert!(private6.is_private());
    }

    #[test]
    fn test_serialization() {
        let ip4 = IpAddress::new("192.168.1.1").unwrap();
        let notation = NotationType::from(ip4.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("address").and_then(|v| v.as_str()), Some("192.168.1.1"));
            assert_eq!(obj.get("type").and_then(|v| v.as_str()), Some("ipv4"));
        }

        let ip_back = IpAddress::try_from(notation);
        assert!(ip_back.is_ok());
        assert_eq!(ip_back.unwrap().address(), ip4.address());

        let ip6 = IpAddress::new("2001:db8::1").unwrap();
        let notation = NotationType::from(ip6.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("address").and_then(|v| v.as_str()), Some("2001:db8::1"));
            assert_eq!(obj.get("type").and_then(|v| v.as_str()), Some("ipv6"));
        }

        let ip_back = IpAddress::try_from(notation);
        assert!(ip_back.is_ok());
        assert_eq!(ip_back.unwrap().address(), ip6.address());
    }
} 