use super::error::{AddressError, Result};
use crate::obj::notation::{NotationType, NumberVariant};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct GeoUri {
    uri: String,
    latitude: f64,
    longitude: f64,
    altitude: Option<f64>,
    uncertainty: Option<f64>,
    params: HashMap<String, String>,
}

impl GeoUri {
    pub fn new<T: Into<String>>(uri: T) -> Result<Self> {
        let uri_str = uri.into();

        if !Self::is_valid(&uri_str) {
            return Err(AddressError::InvalidGeoUriFormat(uri_str));
        }

        // Parse the GeoURI
        let (latitude, longitude, altitude, uncertainty, params) = Self::parse_geo_uri(&uri_str)?;

        Ok(Self {
            uri: uri_str,
            latitude,
            longitude,
            altitude,
            uncertainty,
            params,
        })
    }

    pub fn from_coordinates(
        latitude: f64,
        longitude: f64,
        altitude: Option<f64>,
        uncertainty: Option<f64>,
        params: HashMap<String, String>,
    ) -> Result<Self> {
        // Validate coordinates
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(AddressError::InvalidGeoUriFormat(format!("Invalid latitude: {}", latitude)));
        }

        if !(-180.0..=180.0).contains(&longitude) {
            return Err(AddressError::InvalidGeoUriFormat(format!("Invalid longitude: {}", longitude)));
        }

        // Build the URI
        let mut uri = format!("geo:{},{}", latitude, longitude);

        if let Some(alt) = altitude {
            uri.push_str(&format!(",{}", alt));
        }

        if let Some(unc) = uncertainty {
            uri.push_str(&format!(";u={}", unc));
        }

        for (key, value) in &params {
            uri.push_str(&format!(";{}={}", key, value));
        }

        Ok(Self {
            uri,
            latitude,
            longitude,
            altitude,
            uncertainty,
            params,
        })
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn altitude(&self) -> Option<f64> {
        self.altitude
    }

    pub fn uncertainty(&self) -> Option<f64> {
        self.uncertainty
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn is_valid(uri: &str) -> bool {
        // Basic GeoURI validation (RFC 5870)
        if !uri.starts_with("geo:") {
            return false;
        }

        // Check for coordinates
        if let Some(coords_part) = uri.strip_prefix("geo:") {
            // Split into coordinates and parameters
            let parts: Vec<&str> = coords_part.split(';').collect();
            let coords = parts[0];

            // Check coordinates format
            let coord_parts: Vec<&str> = coords.split(',').collect();
            if coord_parts.len() < 2 || coord_parts.len() > 3 {
                return false;
            }

            // Try to parse latitude and longitude
            if let (Ok(lat), Ok(lon)) = (coord_parts[0].parse::<f64>(), coord_parts[1].parse::<f64>()) {
                // Check coordinate ranges
                if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
                    return false;
                }

                // If altitude is provided, try to parse it
                if coord_parts.len() == 3 && coord_parts[2].parse::<f64>().is_err() {
                    return false;
                }

                // Validate parameters if any
                if parts.len() > 1 {
                    for i in 1..parts.len() {
                        let param = parts[i];
                        if !param.contains('=') {
                            // The uncertainty parameter is special and might not have a value
                            if param != "u" && !param.starts_with("u=") {
                                return false;
                            }
                        }
                    }
                }

                return true;
            }
        }

        false
    }

    fn parse_geo_uri(uri: &str) -> Result<(f64, f64, Option<f64>, Option<f64>, HashMap<String, String>)> {
        // Strip the "geo:" prefix
        let content = uri.strip_prefix("geo:").ok_or_else(|| {
            AddressError::InvalidGeoUriFormat(format!("Missing geo: prefix in {}", uri))
        })?;

        // Split into coordinates and parameters
        let parts: Vec<&str> = content.split(';').collect();
        let coords = parts[0];

        // Parse coordinates
        let coord_parts: Vec<&str> = coords.split(',').collect();
        if coord_parts.len() < 2 {
            return Err(AddressError::InvalidGeoUriFormat(format!("Missing coordinates in {}", uri)));
        }

        let latitude = coord_parts[0].parse::<f64>()
            .map_err(|_| AddressError::InvalidGeoUriFormat(format!("Invalid latitude: {}", coord_parts[0])))?;

        let longitude = coord_parts[1].parse::<f64>()
            .map_err(|_| AddressError::InvalidGeoUriFormat(format!("Invalid longitude: {}", coord_parts[1])))?;

        // Validate coordinate ranges
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(AddressError::InvalidGeoUriFormat(format!("Latitude out of range: {}", latitude)));
        }

        if !(-180.0..=180.0).contains(&longitude) {
            return Err(AddressError::InvalidGeoUriFormat(format!("Longitude out of range: {}", longitude)));
        }

        // Parse altitude if present
        let altitude = if coord_parts.len() > 2 {
            Some(coord_parts[2].parse::<f64>()
                .map_err(|_| AddressError::InvalidGeoUriFormat(format!("Invalid altitude: {}", coord_parts[2])))?)
        } else {
            None
        };

        // Parse parameters
        let mut params = HashMap::new();
        let mut uncertainty = None;

        for i in 1..parts.len() {
            let param = parts[i];

            // Handle the uncertainty parameter specially
            if param.starts_with("u=") {
                let value = param.strip_prefix("u=").unwrap();
                uncertainty = Some(value.parse::<f64>()
                    .map_err(|_| AddressError::InvalidGeoUriFormat(format!("Invalid uncertainty: {}", value)))?);
            } else if param.contains('=') {
                let kv: Vec<&str> = param.split('=').collect();
                if kv.len() == 2 {
                    params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok((latitude, longitude, altitude, uncertainty, params))
    }
}

impl fmt::Display for GeoUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl FromStr for GeoUri {
    type Err = AddressError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        GeoUri::new(s)
    }
}

impl Eq for GeoUri {}

impl std::hash::Hash for GeoUri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state);
    }
}

impl From<GeoUri> for NotationType {
    fn from(geo: GeoUri) -> Self {
        let mut obj = HashMap::new();

        obj.insert("uri".to_string(), NotationType::String(geo.uri));
        obj.insert("latitude".to_string(), NotationType::Number(NumberVariant::Float(geo.latitude)));
        obj.insert("longitude".to_string(), NotationType::Number(NumberVariant::Float(geo.longitude)));

        if let Some(alt) = geo.altitude {
            obj.insert("altitude".to_string(), NotationType::Number(NumberVariant::Float(alt)));
        }

        if let Some(unc) = geo.uncertainty {
            obj.insert("uncertainty".to_string(), NotationType::Number(NumberVariant::Float(unc)));
        }

        if !geo.params.is_empty() {
            let params_obj: HashMap<String, NotationType> = geo.params
                .into_iter()
                .map(|(k, v)| (k, NotationType::String(v)))
                .collect();
            obj.insert("params".to_string(), NotationType::Object(params_obj));
        }

        NotationType::Object(obj)
    }
}

impl TryFrom<NotationType> for GeoUri {
    type Error = AddressError;

    fn try_from(value: NotationType) -> std::result::Result<Self, Self::Error> {
        match value {
            NotationType::String(s) => GeoUri::new(s),
            NotationType::Object(obj) => {
                if let Some(NotationType::String(uri)) = obj.get("uri") {
                    GeoUri::new(uri)
                } else if let (Some(NotationType::Number(lat)), Some(NotationType::Number(lon))) =
                    (obj.get("latitude"), obj.get("longitude")) {
                    let latitude = match lat {
                        NumberVariant::Float(f) => *f,
                        NumberVariant::Int(i) => *i as f64,
                        NumberVariant::Uint(u) => *u as f64,
                    };

                    let longitude = match lon {
                        NumberVariant::Float(f) => *f,
                        NumberVariant::Int(i) => *i as f64,
                        NumberVariant::Uint(u) => *u as f64,
                    };

                    let altitude = obj.get("altitude").and_then(|v| match v {
                        NotationType::Number(NumberVariant::Float(f)) => Some(*f),
                        NotationType::Number(NumberVariant::Int(i)) => Some(*i as f64),
                        NotationType::Number(NumberVariant::Uint(u)) => Some(*u as f64),
                        _ => None,
                    });

                    let uncertainty = obj.get("uncertainty").and_then(|v| match v {
                        NotationType::Number(NumberVariant::Float(f)) => Some(*f),
                        NotationType::Number(NumberVariant::Int(i)) => Some(*i as f64),
                        NotationType::Number(NumberVariant::Uint(u)) => Some(*u as f64),
                        _ => None,
                    });

                    let mut params = HashMap::new();
                    if let Some(NotationType::Object(params_obj)) = obj.get("params") {
                        for (key, value) in params_obj {
                            if let NotationType::String(s) = value {
                                params.insert(key.clone(), s.clone());
                            }
                        }
                    }

                    GeoUri::from_coordinates(latitude, longitude, altitude, uncertainty, params)
                } else {
                    Err(AddressError::InvalidGeoUriFormat("Missing required fields in object".to_string()))
                }
            }
            _ => Err(AddressError::InvalidGeoUriFormat("Invalid notation type".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_geouri() {
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605"));
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605,100"));
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605;u=35"));
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605,100;u=35"));
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605;crs=wgs84"));
        assert!(GeoUri::is_valid("geo:40.714623,-74.006605;u=35;crs=wgs84"));
    }

    #[test]
    fn test_invalid_geouri() {
        assert!(!GeoUri::is_valid(""));
        assert!(!GeoUri::is_valid("geo:")); // Missing coordinates
        assert!(!GeoUri::is_valid("geo:40.714623")); // Missing longitude
        assert!(!GeoUri::is_valid("geo:40.714623,-74.006605,")); // Empty altitude
        assert!(!GeoUri::is_valid("geo:100,-74.006605")); // Latitude out of range
        assert!(!GeoUri::is_valid("geo:40.714623,200")); // Longitude out of range
        assert!(!GeoUri::is_valid("geo:invalid,-74.006605")); // Invalid latitude
    }

    #[test]
    fn test_geouri_components() {
        let geo = GeoUri::new("geo:40.714623,-74.006605,100;u=35;crs=wgs84").unwrap();
        assert_eq!(geo.latitude(), 40.714623);
        assert_eq!(geo.longitude(), -74.006605);
        assert_eq!(geo.altitude(), Some(100.0));
        assert_eq!(geo.uncertainty(), Some(35.0));
        assert_eq!(geo.params().get("crs"), Some(&"wgs84".to_string()));
    }

    #[test]
    fn test_from_coordinates() {
        let params = HashMap::from([("crs".to_string(), "wgs84".to_string())]);
        let geo = GeoUri::from_coordinates(40.714623, -74.006605, Some(100.0), Some(35.0), params).unwrap();
        assert_eq!(geo.uri(), "geo:40.714623,-74.006605,100;u=35;crs=wgs84");
    }

    #[test]
    fn test_serialization() {
        let geo = GeoUri::new("geo:40.714623,-74.006605,100;u=35").unwrap();
        let notation = NotationType::from(geo.clone());

        assert!(matches!(notation, NotationType::Object(_)));

        if let NotationType::Object(obj) = &notation {
            assert_eq!(obj.get("uri").and_then(|v| v.as_str()), Some("geo:40.714623,-74.006605,100;u=35"));
            assert_eq!(obj.get("latitude").and_then(|v| v.as_f64()), Some(40.714623));
            assert_eq!(obj.get("longitude").and_then(|v| v.as_f64()), Some(-74.006605));
            assert_eq!(obj.get("altitude").and_then(|v| v.as_f64()), Some(100.0));
            assert_eq!(obj.get("uncertainty").and_then(|v| v.as_f64()), Some(35.0));
        }

        let geo_back = GeoUri::try_from(notation);
        assert!(geo_back.is_ok());
        assert_eq!(geo_back.unwrap().uri(), geo.uri());
    }
} 