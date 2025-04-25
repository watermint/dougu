use chrono::{Datelike, NaiveDate, Utc};
// Build module provides utilities for managing build information and versioning
use serde::{Deserialize, Serialize};

// Minor version constants
const MINOR_VERSION_GITHUB_MAIN: u32 = 8;
const MINOR_VERSION_GITHUB_CURRENT: u32 = 7;
const MINOR_VERSION_GITHUB_OTHER: u32 = 6;
const MINOR_VERSION_GITHUB_DEFAULT: u32 = 4;
const MINOR_VERSION_LOCAL: u32 = 0;
const MINOR_VERSION_OTHER_CHANNELS: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub build_release: u32,
    pub build_timestamp: String,
    pub build_type: String,
    pub repository_name: String,
    pub repository_owner: String,
    pub copyright_owner: String,
    pub copyright_year: u32,
    pub copyright_start_year: u32,
    pub executable_name: String,
}

impl BuildInfo {
    /// Returns the build information in the format: RELEASE.BUILD_TYPE
    pub fn version_string(&self) -> String {
        format!("{}.{}", self.build_release, self.build_type)
    }

    /// Returns a semantic version string in the format: MAJOR.MINOR.PATCH+BUILD_INFO
    /// Where:
    /// - MAJOR: is the release number
    /// - MINOR: is 0 for stable or 1+ for dev (incremented per build_type)
    /// - PATCH: is derived from timestamp for local builds, or 0 for CI builds
    /// - BUILD_INFO: includes build_type and timestamp
    pub fn semantic_version(&self) -> String {
        // Convert build_type to a numeric minor version
        let minor = match self.build_type.as_str() {
            "github" => {
                // Check the branch from GITHUB_REF environment variable
                if let Ok(github_ref) = std::env::var("GITHUB_REF") {
                    if github_ref.contains("/main") {
                        MINOR_VERSION_GITHUB_MAIN // Main branch
                    } else if github_ref.contains("/current") {
                        MINOR_VERSION_GITHUB_CURRENT // Current branch
                    } else {
                        MINOR_VERSION_GITHUB_OTHER // Other branches
                    }
                } else if let Some(github_ref) = option_env!("GITHUB_REF") {
                    if github_ref.contains("/main") {
                        MINOR_VERSION_GITHUB_MAIN // Main branch
                    } else if github_ref.contains("/current") {
                        MINOR_VERSION_GITHUB_CURRENT // Current branch
                    } else {
                        MINOR_VERSION_GITHUB_OTHER // Other branches
                    }
                } else {
                    MINOR_VERSION_GITHUB_DEFAULT // Default for github builds without branch info
                }
            }
            "local" => MINOR_VERSION_LOCAL,  // Development channel
            _ => MINOR_VERSION_OTHER_CHANNELS,        // Other channels
        };

        // For CI builds (github), try to use GITHUB_RUN_NUMBER for patch version
        if self.build_type == "github" {
            let patch = if let Ok(run_number) = std::env::var("GITHUB_RUN_NUMBER") {
                run_number.parse::<u32>().unwrap_or(0)
            } else if let Some(run_number) = option_env!("GITHUB_RUN_NUMBER") {
                run_number.parse::<u32>().unwrap_or(0)
            } else {
                0 // Default if no run number available
            };

            format!("{}.{}.{}", self.build_release, minor, patch)
        } else {
            // For non-CI builds, use days since 2025-01-01 as patch
            let patch = {
                const EPOCH_YEAR: i32 = 2025;
                const EPOCH_MONTH: u32 = 1;
                const EPOCH_DAY: u32 = 1;

                let epoch_date = NaiveDate::from_ymd_opt(EPOCH_YEAR, EPOCH_MONTH, EPOCH_DAY)
                    .expect("Invalid epoch date components");
                let current_date = Utc::now().date_naive();
                current_date.signed_duration_since(epoch_date).num_days() as u32
            };

            // Create build metadata from build_type and timestamp for non-CI builds
            let build_metadata = format!("{}.{}",
                                         self.build_type,
                                         self.build_timestamp.replace("T", "").replace(":", "").replace("-", "").replace("Z", "")
            );

            format!("{}.{}.{}+{}", self.build_release, minor, patch, build_metadata)
        }
    }

    /// Returns a formatted version string for display purposes
    pub fn display_string(&self) -> String {
        format!(
            "Version {}.{} (built on {})",
            self.build_release,
            self.build_type,
            self.build_timestamp
        )
    }

    /// Create a new BuildInfo with default values for local development
    fn new_local() -> Self {
        let build_timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let (repository_owner, repository_name) = detect_repository();
        // Use the major version from Cargo.toml as release
        let version_str = env!("CARGO_PKG_VERSION");
        let build_release = version_str.split('.').next()
            .and_then(|s| s.parse::<u32>().ok())
            .expect("Failed to parse major version from CARGO_PKG_VERSION");
        Self {
            build_release,
            build_type: "local".to_string(),
            build_timestamp,
            repository_owner,
            repository_name,
            copyright_owner: "Takayuki Okazaki".to_string(),
            copyright_year: Utc::now().year() as u32,
            copyright_start_year: 2025,
            executable_name: "dougu".to_string(),
        }
    }

    /// Create a new BuildInfo for CI builds
    fn new_ci(_run_number: &str, build_release: u32) -> Self {
        let (repository_owner, repository_name) = detect_repository();

        // Use environment variables for copyright information if available
        let copyright_owner = {
            if let Some(owner) = option_env!("DOUGU_COPYRIGHT_OWNER") {
                owner.to_string()
            } else if let Ok(owner) = std::env::var("DOUGU_COPYRIGHT_OWNER") {
                owner
            } else {
                "Takayuki Okazaki".to_string()
            }
        };

        let copyright_year = {
            if let Some(year) = option_env!("DOUGU_COPYRIGHT_YEAR").and_then(|y| y.parse::<u32>().ok()) {
                year
            } else if let Ok(year_str) = std::env::var("DOUGU_COPYRIGHT_YEAR") {
                year_str.parse::<u32>().unwrap_or_else(|_| Utc::now().year() as u32)
            } else {
                Utc::now().year() as u32
            }
        };

        let executable_name = {
            if let Some(name) = option_env!("DOUGU_EXECUTABLE_NAME") {
                name.to_string()
            } else if let Ok(name) = std::env::var("DOUGU_EXECUTABLE_NAME") {
                name
            } else {
                "dougu".to_string()
            }
        };

        Self {
            build_release,
            build_type: "github".to_string(),
            build_timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            repository_owner,
            repository_name,
            copyright_owner,
            copyright_year,
            copyright_start_year: 2025,
            executable_name,
        }
    }

    /// Factory function to create a new BuildInfo based on the environment
    pub fn new() -> Self {
        // Check if we're in a CI environment
        if is_ci_environment() {
            // CI environment detected
            // Try to get release from environment variables first,
            // then from CARGO_PKG_VERSION, default to 1 (instead of 0)
            let build_release = {
                if let Some(release) = option_env!("DOUGU_RELEASE") {
                    release.parse::<u32>().unwrap_or(1)
                } else if let Some(release) = option_env!("RELEASE") {
                    release.parse::<u32>().unwrap_or(1)
                } else if let Ok(release) = std::env::var("DOUGU_RELEASE") {
                    release.parse::<u32>().unwrap_or(1)
                } else if let Ok(release) = std::env::var("RELEASE") {
                    release.parse::<u32>().unwrap_or(0)
                } else {
                    // Try to get from Cargo.toml version
                    let version_str = env!("CARGO_PKG_VERSION");
                    version_str.split('.')
                        .next()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(1)
                }
            };

            // Get the run number from environment
            let run_number = {
                if let Some(num) = option_env!("GITHUB_RUN_NUMBER") {
                    num.to_string()
                } else if let Ok(num) = std::env::var("GITHUB_RUN_NUMBER") {
                    num
                } else {
                    "0".to_string()
                }
            };
            Self::new_ci(run_number.as_str(), build_release)
        } else {
            // Local development environment
            Self::new_local()
        }
    }

    /// Returns a copyright notice string
    pub fn copyright_notice(&self) -> String {
        let year_range = if self.copyright_start_year < self.copyright_year {
            format!("{}-{}", self.copyright_start_year, self.copyright_year)
        } else {
            self.copyright_year.to_string()
        };

        format!("© {} {}", year_range, self.copyright_owner)
    }
}

/// Detect the repository owner and name from git configuration or environment variables
fn detect_repository() -> (String, String) {
    // Try to get from environment first (useful in CI environments)
    if let Some(github_repository) = option_env!("GITHUB_REPOSITORY") {
        if let Some((owner, repo)) = github_repository.split_once('/') {
            return (owner.to_string(), repo.to_string());
        }
    }

    if let Ok(github_repository) = std::env::var("GITHUB_REPOSITORY") {
        if let Some((owner, repo)) = github_repository.split_once('/') {
            return (owner.to_string(), repo.to_string());
        }
    }

    // Default values if detection fails
    ("unknown".to_string(), "dougu".to_string())
}

/// Detect if we're running in a CI environment
fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok() ||
        option_env!("CI").is_some() ||
        std::env::var("GITHUB_ACTIONS").is_ok() ||
        option_env!("GITHUB_ACTIONS").is_some()
}

/// Utility function to get the build info
pub fn get_build_info() -> BuildInfo {
    BuildInfo::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_version() {
        let mut info = BuildInfo::new_local();

        // Test local build version format
        let version = info.semantic_version();
        assert!(version.contains('+'), "Local builds should include build metadata");
        assert!(version.contains("local"), "Local builds should have 'local' in metadata");

        // Test GitHub build version format
        info.build_type = "github".to_string();
        let version = info.semantic_version();
        assert!(version.split('.').count() >= 3, "Should have major, minor, and patch components");
    }

    #[test]
    fn test_copyright_notice() {
        let mut info = BuildInfo::new();

        // Same start and current year
        info.copyright_start_year = 2025;
        info.copyright_year = 2025;
        info.copyright_owner = "Test Owner".to_string();
        assert_eq!(info.copyright_notice(), "© 2025 Test Owner");

        // Different start and current year
        info.copyright_start_year = 2020;
        info.copyright_year = 2025;
        assert_eq!(info.copyright_notice(), "© 2020-2025 Test Owner");
    }
} 