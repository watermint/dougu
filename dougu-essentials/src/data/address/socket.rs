use super::error::{AddressError, Result};
use super::ip::IpAddress;
use crate::obj::notation::{NotationType, NumberVariant};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketAddress {
    address: String,
    ip: IpAddress,
    port: u16,
}

impl SocketAddress {
    pub fn new<T: Into<String>>(address: T) -> Result<Self> {
        let address_str = address.into();

        if !Self::is_valid(&address_str) {
            return Err(AddressError::InvalidSocketFormat(address_str));
        }

        // Parse IP and port
        let (ip_str, port_str) = Self::split_socket_address(&address_str)?;
        let ip = IpAddress::new(ip_str)?;
        let port = port_str.parse::<u16>().map_err(|_| AddressError::InvalidSocketFormat(address_str.clone()))?;

        Ok(Self {
            address: address_str,
            ip,
            port,
        })
    }

    pub fn from_parts(ip: IpAddress, port: u16) -> Self {
        let address = format!("{ip}:{port}");
        Self {
            address,
            ip,
            port,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn ip(&self) -> &IpAddress {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_valid(address: &str) -> bool {
        // For IPv4: a.b.c.d:port
        // For IPv6: [a:b:c:d:e:f:g:h]:port

        if address.is_empty() {
            return false;
        }

        if let Ok((ip_str, port_str)) = Self::split_socket_address(address) {
            // Check if IP is valid
            if IpAddress::new(ip_str).is_err() {
                return false;
            }

            // Check if port is valid
            if let Ok(port) = port_str.parse::<u16>() {
                return true;
            }
        }

        false
    }

    fn split_socket_address(address: &str) -> Result<(String, String)> {
        // Handle IPv6 format: [ipv6]:port
        if address.starts_with('[') {
            if let Some(closing_bracket) = address.find(']') {
                if address.len() > closing_bracket + 1 && address.chars().nth(closing_bracket + 1) == Some(':') {
                    let ip_str = address[1..closing_bracket].to_string();
                    let port_str = address[closing_bracket + 2..].to_string();
                    return Ok((ip_str, port_str));
                }
            }
            return Err(AddressError::InvalidSocketFormat(address.to_string()));
        }

        // Handle IPv4 format: ipv4:port
        if let Some(last_colon) = address.rfind(':') {
            let ip_str = address[..last_colon].to_string();
            let port_str = address[last_colon + 1..].to_string();
            return Ok((ip_str, port_str));
        }

        Err(AddressError::InvalidSocketFormat(address.to_string()))
    }
}

impl fmt::Display for SocketAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl FromStr for SocketAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        SocketAddress::new(s)
    }
}

impl From<SocketAddress> for NotationType {
    fn from(socket: SocketAddress) -> Self {
        let mut obj = HashMap::new();

        obj.insert("address".to_string(), NotationType::String(socket.address().to_string()));
        obj.insert("ip".to_string(), socket.ip().clone().into());
        obj.insert("port".to_string(), NotationType::Number(NumberVariant::Uint(socket.port() as u64)));

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for SocketAddress {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => SocketAddress::new(s),
            NotationType::Object(obj) => {
                if let Some(NotationType::String(addr)) = obj.get("address") {
                    SocketAddress::new(addr)
                } else if let (Some(ip_value), Some(NotationType::Number(port_value))) = (obj.get("ip"), obj.get("port")) {
                    let ip = IpAddress::try_from(ip_value.clone())?;

                    let port = match port_value {
                        NumberVariant::Uint(n) => *n as u16,
                        NumberVariant::Int(n) => {
                            if *n < 0 || *n > 65535 {
                                return Err(AddressError::InvalidSocketFormat("Port out of range".to_string()));
                            }
                            *n as u16
                        }
                        NumberVariant::Float(n) => {
                            if *n < 0.0 || *n > 65535.0 || n.fract() != 0.0 {
                                return Err(AddressError::InvalidSocketFormat("Invalid port number".to_string()));
                            }
                            *n as u16
                        }
                    };

                    Ok(SocketAddress::from_parts(ip, port))
                } else {
                    Err(AddressError::InvalidSocketFormat("Missing required fields in object".to_string()))
                }
            }
            _ => Err(AddressError::InvalidSocketFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_socket() {
        assert!(SocketAddress::is_valid("127.0.0.1:8080"));
        assert!(SocketAddress::is_valid("192.168.1.1:80"));
        assert!(SocketAddress::is_valid("[::1]:8080"));
        assert!(SocketAddress::is_valid("[2001:db8::1]:443"));
    }

    #[test]
    fn test_invalid_socket() {
        assert!(!SocketAddress::is_valid(""));
        assert!(!SocketAddress::is_valid("127.0.0.1")); // No port
        assert!(!SocketAddress::is_valid("127.0.0.1:")); // Empty port
        assert!(!SocketAddress::is_valid("127.0.0.1:65536")); // Port too large
        assert!(!SocketAddress::is_valid("[::1]")); // No port for IPv6
        assert!(!SocketAddress::is_valid("[::1]:abc")); // Invalid port
    }

    #[test]
    fn test_socket_parts() {
        let socket4 = SocketAddress::new("192.168.1.1:8080").unwrap();
        assert_eq!(socket4.ip().address(), "192.168.1.1");
        assert_eq!(socket4.port(), 8080);

        let socket6 = SocketAddress::new("[2001:db8::1]:443").unwrap();
        assert_eq!(socket6.ip().address(), "2001:db8::1");
        assert_eq!(socket6.port(), 443);
    }

    #[test]
    fn test_from_parts() {
        let ip4 = IpAddress::new("127.0.0.1").unwrap();
        let socket = SocketAddress::from_parts(ip4, 8080);
        assert_eq!(socket.address(), "127.0.0.1:8080");

        let ip6 = IpAddress::new("::1").unwrap();
        let socket = SocketAddress::from_parts(ip6, 8080);
        assert_eq!(socket.address(), "::1:8080");
    }

    #[test]
    fn test_serialization() {
        let socket = SocketAddress::new("127.0.0.1:8080").unwrap();
        let notation = NotationType::from(socket.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("address").and_then(|v| v.as_str()), Some("127.0.0.1:8080"));
            assert_eq!(obj.get("port").and_then(|v| v.as_u64()), Some(8080));
        }

        let socket_back = SocketAddress::try_from(notation);
        assert!(socket_back.is_ok());
        assert_eq!(socket_back.unwrap().address(), socket.address());
    }
} 