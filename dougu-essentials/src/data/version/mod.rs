use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;

/// Error types related to semantic versioning.
#[derive(Debug, Error)]
pub enum VersionError {
    #[error("invalid version format: {0}")]
    InvalidFormat(String),
    #[error("invalid version number: {0}")]
    InvalidNumber(String),
    #[error("invalid pre-release identifier: {0}")]
    InvalidPreRelease(String),
    #[error("invalid build metadata: {0}")]
    InvalidBuildMetadata(String),
}

/// Represents a semantic version (SemVer) according to the specification at semver.org.
///
/// A semantic version follows the format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
/// - MAJOR version when you make incompatible API changes
/// - MINOR version when you add functionality in a backward compatible manner
/// - PATCH version when you make backward compatible bug fixes
///
/// Pre-release and build metadata are optional.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    /// Major version number (incremented for incompatible API changes)
    pub major: u64,
    /// Minor version number (incremented for backward compatible functionality changes)
    pub minor: u64,
    /// Patch version number (incremented for backward compatible bug fixes)
    pub patch: u64,
    /// Pre-release identifiers (optional)
    pub pre_release: Option<Vec<String>>,
    /// Build metadata (optional)
    pub build: Option<Vec<String>>,
}

impl Version {
    /// Creates a new Version with the given version numbers.
    ///
    /// # Arguments
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    ///
    /// # Returns
    /// A new Version with the specified version numbers and no pre-release or build metadata.
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build: None,
        }
    }

    /// Creates a new Version with the given version numbers and pre-release identifiers.
    ///
    /// # Arguments
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    /// * `pre_release` - A vector of pre-release identifiers
    ///
    /// # Returns
    /// A new Version with the specified version numbers, pre-release identifiers, and no build metadata.
    pub fn with_pre_release(major: u64, minor: u64, patch: u64, pre_release: Vec<String>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
            build: None,
        }
    }

    /// Creates a new Version with the given version numbers, pre-release identifiers, and build metadata.
    ///
    /// # Arguments
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    /// * `pre_release` - A vector of pre-release identifiers, or None if not present
    /// * `build` - A vector of build metadata identifiers, or None if not present
    ///
    /// # Returns
    /// A new Version with the specified values.
    pub fn with_build(
        major: u64,
        minor: u64,
        patch: u64,
        pre_release: Option<Vec<String>>,
        build: Vec<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release,
            build: Some(build),
        }
    }

    /// Creates a new Version with all components.
    ///
    /// # Arguments
    /// * `major` - The major version number
    /// * `minor` - The minor version number
    /// * `patch` - The patch version number
    /// * `pre_release` - A vector of pre-release identifiers, or None if not present
    /// * `build` - A vector of build metadata identifiers, or None if not present
    ///
    /// # Returns
    /// A new Version with the specified values.
    pub fn with_all(
        major: u64,
        minor: u64,
        patch: u64,
        pre_release: Option<Vec<String>>,
        build: Option<Vec<String>>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release,
            build,
        }
    }
    
    /// Checks if this version is a pre-release.
    ///
    /// # Returns
    /// `true` if this version has pre-release identifiers, `false` otherwise.
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
    
    /// Checks if this version has build metadata.
    ///
    /// # Returns
    /// `true` if this version has build metadata identifiers, `false` otherwise.
    pub fn has_build_metadata(&self) -> bool {
        self.build.is_some()
    }

    /// Compares this version with another version.
    ///
    /// # Arguments
    /// * `other` - The version to compare with
    ///
    /// # Returns
    /// - `Ordering::Less` if this version is less than the other
    /// - `Ordering::Equal` if this version is equal to the other
    /// - `Ordering::Greater` if this version is greater than the other
    pub fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }

    /// Checks if this version is greater than another version.
    ///
    /// # Arguments
    /// * `other` - The version to compare with
    ///
    /// # Returns
    /// `true` if this version is greater than the other, `false` otherwise.
    pub fn is_greater_than(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Greater
    }

    /// Checks if this version is greater than or equal to another version.
    ///
    /// # Arguments
    /// * `other` - The version to compare with
    ///
    /// # Returns
    /// `true` if this version is greater than or equal to the other, `false` otherwise.
    pub fn is_greater_than_or_equal(&self, other: &Self) -> bool {
        let ordering = self.cmp(other);
        ordering == Ordering::Greater || ordering == Ordering::Equal
    }

    /// Checks if this version is less than another version.
    ///
    /// # Arguments
    /// * `other` - The version to compare with
    ///
    /// # Returns
    /// `true` if this version is less than the other, `false` otherwise.
    pub fn is_less_than(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Less
    }

    /// Checks if this version is less than or equal to another version.
    ///
    /// # Arguments
    /// * `other` - The version to compare with
    ///
    /// # Returns
    /// `true` if this version is less than or equal to the other, `false` otherwise.
    pub fn is_less_than_or_equal(&self, other: &Self) -> bool {
        let ordering = self.cmp(other);
        ordering == Ordering::Less || ordering == Ordering::Equal
    }

    /// Checks if this version is compatible with another version.
    /// 
    /// A version is compatible with another version if they have the same major version
    /// and the current version is greater than or equal to the specified minimum version.
    ///
    /// # Arguments
    /// * `min_version` - The minimum version to check compatibility with
    ///
    /// # Returns
    /// `true` if this version is compatible with the specified minimum version, `false` otherwise.
    pub fn is_compatible_with(&self, min_version: &Self) -> bool {
        self.major == min_version.major && self.is_greater_than_or_equal(min_version)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}", pre.join("."))?;
        }
        
        if let Some(build) = &self.build {
            write!(f, "+{}", build.join("."))?;
        }
        
        Ok(())
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split by + to separate build metadata
        let (version_pre, build) = match s.split_once('+') {
            Some((v, b)) => (v, Some(parse_build_metadata(b)?)),
            None => (s, None),
        };
        
        // Split by - to separate pre-release
        let (version, pre_release) = match version_pre.split_once('-') {
            Some((v, p)) => (v, Some(parse_pre_release(p)?)),
            None => (version_pre, None),
        };
        
        // Parse version numbers
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(format!("Expected 3 version components (MAJOR.MINOR.PATCH), found {}", parts.len())));
        }
        
        let major = parts[0].parse::<u64>()
            .map_err(|_| VersionError::InvalidNumber(format!("Invalid major version: {}", parts[0])))?;
        
        let minor = parts[1].parse::<u64>()
            .map_err(|_| VersionError::InvalidNumber(format!("Invalid minor version: {}", parts[1])))?;
        
        let patch = parts[2].parse::<u64>()
            .map_err(|_| VersionError::InvalidNumber(format!("Invalid patch version: {}", parts[2])))?;
        
        Ok(Version {
            major,
            minor,
            patch,
            pre_release,
            build,
        })
    }
}

/// Parse pre-release identifiers from a string.
///
/// Pre-release identifiers are dot-separated and must consist of only ASCII
/// alphanumerics and hyphens, and must not be empty. Numeric identifiers must
/// not have leading zeros.
fn parse_pre_release(s: &str) -> Result<Vec<String>, VersionError> {
    if s.is_empty() {
        return Err(VersionError::InvalidPreRelease("Pre-release cannot be empty".to_string()));
    }
    
    let identifiers: Vec<String> = s.split('.').map(String::from).collect();
    
    for id in &identifiers {
        if id.is_empty() {
            return Err(VersionError::InvalidPreRelease("Pre-release identifiers cannot be empty".to_string()));
        }
        
        // Check if the identifier is purely numeric
        if id.chars().all(|c| c.is_ascii_digit()) {
            // Numeric identifiers must not have leading zeros
            if id.len() > 1 && id.starts_with('0') {
                return Err(VersionError::InvalidPreRelease(format!("Numeric pre-release identifier cannot have leading zeros: {}", id)));
            }
        } else {
            // Non-numeric identifiers must be valid
            if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(VersionError::InvalidPreRelease(format!("Pre-release identifier contains invalid characters: {}", id)));
            }
        }
    }
    
    Ok(identifiers)
}

/// Parse build metadata from a string.
///
/// Build metadata identifiers are dot-separated and must consist of only ASCII
/// alphanumerics and hyphens, and must not be empty.
fn parse_build_metadata(s: &str) -> Result<Vec<String>, VersionError> {
    if s.is_empty() {
        return Err(VersionError::InvalidBuildMetadata("Build metadata cannot be empty".to_string()));
    }
    
    let identifiers: Vec<String> = s.split('.').map(String::from).collect();
    
    for id in &identifiers {
        if id.is_empty() {
            return Err(VersionError::InvalidBuildMetadata("Build metadata identifiers cannot be empty".to_string()));
        }
        
        if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(VersionError::InvalidBuildMetadata(format!("Build metadata identifier contains invalid characters: {}", id)));
        }
    }
    
    Ok(identifiers)
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        // A version with pre-release identifiers has lower precedence than
        // one without pre-release identifiers
        match (self.pre_release.is_some(), other.pre_release.is_some()) {
            (false, true) => return Ordering::Greater,
            (true, false) => return Ordering::Less,
            (false, false) => return Ordering::Equal,
            (true, true) => {}
        }
        
        // Compare pre-release identifiers
        let self_pre = self.pre_release.as_ref().unwrap();
        let other_pre = other.pre_release.as_ref().unwrap();
        
        for (a, b) in self_pre.iter().zip(other_pre.iter()) {
            let a_is_numeric = a.chars().all(|c| c.is_ascii_digit());
            let b_is_numeric = b.chars().all(|c| c.is_ascii_digit());
            
            // According to semver.org spec:
            // Numeric identifiers always have lower precedence than non-numeric identifiers
            match (a_is_numeric, b_is_numeric) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                (true, true) => {
                    // If both are numeric, compare them numerically
                    let a_num = a.parse::<u64>().unwrap();
                    let b_num = b.parse::<u64>().unwrap();
                    match a_num.cmp(&b_num) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                },
                (false, false) => {
                    // If both are non-numeric, compare them lexically
                    match a.cmp(b) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
            }
        }
        
        // If one pre-release list is a prefix of the other, the shorter one has lower precedence
        self_pre.len().cmp(&other_pre.len())
        
        // Build metadata does not figure into precedence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_versions() {
        assert_eq!(
            "1.2.3".parse::<Version>().unwrap(),
            Version::new(1, 2, 3)
        );
        
        assert_eq!(
            "1.0.0-alpha".parse::<Version>().unwrap(),
            Version::with_pre_release(1, 0, 0, vec!["alpha".to_string()])
        );
        
        assert_eq!(
            "1.0.0-alpha.1".parse::<Version>().unwrap(),
            Version::with_pre_release(1, 0, 0, vec!["alpha".to_string(), "1".to_string()])
        );
        
        assert_eq!(
            "1.0.0+build.1".parse::<Version>().unwrap(),
            Version::with_build(1, 0, 0, None, vec!["build".to_string(), "1".to_string()])
        );
        
        assert_eq!(
            "1.0.0-alpha.1+build.2".parse::<Version>().unwrap(),
            Version::with_all(
                1, 0, 0,
                Some(vec!["alpha".to_string(), "1".to_string()]),
                Some(vec!["build".to_string(), "2".to_string()])
            )
        );
    }
    
    #[test]
    fn test_parse_invalid_versions() {
        assert!("1.2".parse::<Version>().is_err());
        assert!("1.2.3.4".parse::<Version>().is_err());
        assert!("1.2.3-".parse::<Version>().is_err());
        assert!("1.2.3+".parse::<Version>().is_err());
        assert!("1.2.3-a..b".parse::<Version>().is_err());
        assert!("1.2.3-01".parse::<Version>().is_err());
        assert!("a.b.c".parse::<Version>().is_err());
    }
    
    #[test]
    fn test_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        
        let version = Version::with_pre_release(1, 0, 0, vec!["alpha".to_string(), "1".to_string()]);
        assert_eq!(version.to_string(), "1.0.0-alpha.1");
        
        let version = Version::with_build(1, 0, 0, None, vec!["build".to_string(), "1".to_string()]);
        assert_eq!(version.to_string(), "1.0.0+build.1");
        
        let version = Version::with_all(
            1, 0, 0,
            Some(vec!["alpha".to_string(), "1".to_string()]),
            Some(vec!["build".to_string(), "2".to_string()])
        );
        assert_eq!(version.to_string(), "1.0.0-alpha.1+build.2");
    }
    
    #[test]
    fn test_ordering() {
        // Examples from SemVer spec:
        // 1.0.0 < 2.0.0 < 2.1.0 < 2.1.1
        assert!("1.0.0".parse::<Version>().unwrap() < "2.0.0".parse::<Version>().unwrap());
        assert!("2.0.0".parse::<Version>().unwrap() < "2.1.0".parse::<Version>().unwrap());
        assert!("2.1.0".parse::<Version>().unwrap() < "2.1.1".parse::<Version>().unwrap());
        
        // Pre-release versions have lower precedence: 1.0.0-alpha < 1.0.0
        assert!("1.0.0-alpha".parse::<Version>().unwrap() < "1.0.0".parse::<Version>().unwrap());
        
        // Pre-release versions ordered alphabetically: 1.0.0-alpha < 1.0.0-beta
        assert!("1.0.0-alpha".parse::<Version>().unwrap() < "1.0.0-beta".parse::<Version>().unwrap());
        
        // Numeric identifiers have lower precedence than non-numeric: 1.0.0-1 < 1.0.0-alpha
        assert!("1.0.0-1".parse::<Version>().unwrap() < "1.0.0-alpha".parse::<Version>().unwrap());
        
        // Numeric identifiers are compared numerically: 1.0.0-alpha.1 < 1.0.0-alpha.2
        assert!("1.0.0-alpha.1".parse::<Version>().unwrap() < "1.0.0-alpha.2".parse::<Version>().unwrap());
        
        // A larger set of pre-release fields has higher precedence: 1.0.0-alpha < 1.0.0-alpha.1
        assert!("1.0.0-alpha".parse::<Version>().unwrap() < "1.0.0-alpha.1".parse::<Version>().unwrap());
        
        // Build metadata is ignored for precedence: 1.0.0+build.1 == 1.0.0
        assert_eq!(
            "1.0.0+build.1".parse::<Version>().unwrap().cmp(&"1.0.0".parse::<Version>().unwrap()),
            Ordering::Equal
        );
    }
    
    #[test]
    fn test_comparison_methods() {
        let v1 = "1.0.0".parse::<Version>().unwrap();
        let v2 = "2.0.0".parse::<Version>().unwrap();
        let v3 = "2.0.0".parse::<Version>().unwrap();
        
        // Test compare
        assert_eq!(v1.compare(&v2), Ordering::Less);
        assert_eq!(v2.compare(&v1), Ordering::Greater);
        assert_eq!(v2.compare(&v3), Ordering::Equal);
        
        // Test is_greater_than
        assert!(!v1.is_greater_than(&v2));
        assert!(v2.is_greater_than(&v1));
        assert!(!v2.is_greater_than(&v3));
        
        // Test is_greater_than_or_equal
        assert!(!v1.is_greater_than_or_equal(&v2));
        assert!(v2.is_greater_than_or_equal(&v1));
        assert!(v2.is_greater_than_or_equal(&v3));
        
        // Test is_less_than
        assert!(v1.is_less_than(&v2));
        assert!(!v2.is_less_than(&v1));
        assert!(!v2.is_less_than(&v3));
        
        // Test is_less_than_or_equal
        assert!(v1.is_less_than_or_equal(&v2));
        assert!(!v2.is_less_than_or_equal(&v1));
        assert!(v2.is_less_than_or_equal(&v3));
    }
    
    #[test]
    fn test_compatibility() {
        // Same major version, equal versions
        let v1 = "1.0.0".parse::<Version>().unwrap();
        let v2 = "1.0.0".parse::<Version>().unwrap();
        assert!(v1.is_compatible_with(&v2));
        
        // Same major version, higher minor or patch
        let v3 = "1.1.0".parse::<Version>().unwrap();
        let v4 = "1.0.1".parse::<Version>().unwrap();
        assert!(v3.is_compatible_with(&v1));
        assert!(v4.is_compatible_with(&v1));
        
        // Same major version, lower minor or patch
        assert!(!v1.is_compatible_with(&v3));
        assert!(!v1.is_compatible_with(&v4));
        
        // Different major versions
        let v5 = "2.0.0".parse::<Version>().unwrap();
        assert!(!v1.is_compatible_with(&v5));
        assert!(!v5.is_compatible_with(&v1));
        
        // Pre-release versions
        let v6 = "1.0.0-alpha".parse::<Version>().unwrap();
        let v7 = "1.0.0".parse::<Version>().unwrap();
        assert!(!v6.is_compatible_with(&v7)); // Pre-release version is considered less than release
        assert!(v7.is_compatible_with(&v6)); // Release version is compatible with pre-release
    }
    
    #[test]
    fn test_numeric_vs_non_numeric_identifiers() {
        // Testing the rule: Numeric identifiers always have lower precedence than non-numeric identifiers
        
        // Comparing numeric identifier with non-numeric in different positions
        assert!("1.0.0-1.alpha".parse::<Version>().unwrap() < "1.0.0-alpha.1".parse::<Version>().unwrap());
        assert!("1.0.0-alpha.1".parse::<Version>().unwrap() > "1.0.0-1.alpha".parse::<Version>().unwrap());
        
        // Comparing at the same position
        assert!("1.0.0-1".parse::<Version>().unwrap() < "1.0.0-alpha".parse::<Version>().unwrap());
        assert!("1.0.0-alpha".parse::<Version>().unwrap() > "1.0.0-1".parse::<Version>().unwrap());
        
        // Multiple identifiers with mix of numeric and non-numeric
        assert!("1.0.0-alpha.1".parse::<Version>().unwrap() < "1.0.0-alpha.beta".parse::<Version>().unwrap());
        assert!("1.0.0-alpha.beta".parse::<Version>().unwrap() > "1.0.0-alpha.1".parse::<Version>().unwrap());
        
        // Zero as numeric identifier (still has lower precedence)
        assert!("1.0.0-0".parse::<Version>().unwrap() < "1.0.0-a".parse::<Version>().unwrap());
    }
} 