use crate::obj::notation::NotationType;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressType {
    Email,
    Url,
    Uri,
    Domain,
    Ip,
    Socket,
    Mac,
    Urn,
    GeoUri,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressType::Email => write!(f, "Email"),
            AddressType::Url => write!(f, "URL"),
            AddressType::Uri => write!(f, "URI"),
            AddressType::Domain => write!(f, "Domain"),
            AddressType::Ip => write!(f, "IP"),
            AddressType::Socket => write!(f, "Socket"),
            AddressType::Mac => write!(f, "MAC"),
            AddressType::Urn => write!(f, "URN"),
            AddressType::GeoUri => write!(f, "GeoURI"),
        }
    }
}

impl TryFrom<&str> for AddressType {
    type Error = super::error::AddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "email" => Ok(AddressType::Email),
            "url" => Ok(AddressType::Url),
            "uri" => Ok(AddressType::Uri),
            "domain" => Ok(AddressType::Domain),
            "ip" => Ok(AddressType::Ip),
            "socket" => Ok(AddressType::Socket),
            "mac" => Ok(AddressType::Mac),
            "urn" => Ok(AddressType::Urn),
            "geouri" => Ok(AddressType::GeoUri),
            _ => Err(super::error::AddressError::InvalidAddressType(value.to_string())),
        }
    }
}

impl From<AddressType> for NotationType {
    fn from(value: AddressType) -> Self {
        match value {
            AddressType::Email => NotationType::String("Email".to_string()),
            AddressType::Url => NotationType::String("URL".to_string()),
            AddressType::Uri => NotationType::String("URI".to_string()),
            AddressType::Domain => NotationType::String("Domain".to_string()),
            AddressType::Ip => NotationType::String("IP".to_string()),
            AddressType::Socket => NotationType::String("Socket".to_string()),
            AddressType::Mac => NotationType::String("MAC".to_string()),
            AddressType::Urn => NotationType::String("URN".to_string()),
            AddressType::GeoUri => NotationType::String("GeoURI".to_string()),
        }
    }
}

impl TryFrom<NotationType> for AddressType {
    type Error = super::error::AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => Self::try_from(s.as_str()),
            _ => Err(super::error::AddressError::InvalidAddressType("Invalid notation type".to_string())),
        }
    }
} 