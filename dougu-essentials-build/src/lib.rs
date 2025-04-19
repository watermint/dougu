use serde::{Deserialize, Serialize};
use chrono::{Utc, Datelike, TimeZone};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub release: u32,
    pub build_type: String,
    pub build_id: String,
    pub timestamp: String,
}

impl BuildInfo {
    /// Returns the build information in the format: RELEASE.BUILD_TYPE.BUILD_ID
    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.release, self.build_type, self.build_id)
    }
    
    /// Returns a semantic version string in the format: MAJOR.MINOR.PATCH+BUILD_INFO
    /// Where:
    /// - MAJOR: is the release number
    /// - MINOR: is 0 for stable or 1+ for dev (incremented per build_type)
    /// - PATCH: is derived from build_id for local builds, or 0 for CI builds
    /// - BUILD_INFO: includes build_type and timestamp
    pub fn semantic_version(&self) -> String {
        // Convert build_type to a numeric minor version
        let minor = match self.build_type.as_str() {
            "github" => {
                // Check the branch from GITHUB_REF environment variable
                if let Ok(github_ref) = std::env::var("GITHUB_REF") {
                    if github_ref.contains("/main") {
                        8 // Main branch
                    } else if github_ref.contains("/current") {
                        7 // Current branch
                    } else {
                        6 // Other branches
                    }
                } else if let Some(github_ref) = option_env!("GITHUB_REF") {
                    if github_ref.contains("/main") {
                        8 // Main branch
                    } else if github_ref.contains("/current") {
                        7 // Current branch
                    } else {
                        6 // Other branches
                    }
                } else {
                    4 // Default for github builds without branch info
                }
            },
            "local" => 0,  // Development channel
            _ => 1,        // Other channels
        };
        
        // For patch version: use 0 for CI, extract from build_id for local
        let patch = if self.build_type == "github" {
            0
        } else if self.build_id.contains("+") {
            // For local builds with timestamp format like "0-dev+20250419T153700Z"
            // Just use a sequential counter based on day of year
            let date_str = self.build_id.split("+").nth(1).unwrap_or("").split("T").next().unwrap_or("");
            if !date_str.is_empty() && date_str.len() >= 8 {
                // Extract day of year from the date string (YYYYMMDD)
                if let Ok(year) = date_str[0..4].parse::<i32>() {
                    if let Ok(month) = date_str[4..6].parse::<u32>() {
                        if let Ok(day) = date_str[6..8].parse::<u32>() {
                            if let Some(dt) = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).single() {
                                dt.ordinal() as u32 % 1000 // Day of year mod 1000
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            // Try to parse build_id as a number, fallback to 0
            self.build_id.parse::<u32>().unwrap_or(0)
        };
        
        // Create build metadata from build_type and timestamp
        let build_metadata = format!("{}.{}", 
            self.build_type,
            self.timestamp.replace(" ", "").replace(":", "").replace("-", "")
        );
        
        format!("{}.{}.{}+{}", self.release, minor, patch, build_metadata)
    }
    
    /// Returns a formatted version string for display purposes
    pub fn display_string(&self) -> String {
        format!(
            "Version {}.{}.{} (built on {})",
            self.release,
            self.build_type,
            self.build_id,
            self.timestamp
        )
    }

    /// Create a new BuildInfo with default values for local development
    pub fn new_local() -> Self {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        Self {
            release: 0,
            build_type: "local".to_string(),
            build_id: format!("0-dev+{}", timestamp),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }

    /// Create a new BuildInfo for CI builds
    pub fn new_ci(run_number: &str, release: u32) -> Self {
        Self {
            release,
            build_type: "github".to_string(),
            build_id: run_number.to_string(),
            timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}

/// Detect if running in a CI environment
fn is_ci_environment() -> bool {
    option_env!("GITHUB_ACTIONS").is_some() ||
    option_env!("CI").is_some() ||
    std::env::var("GITHUB_ACTIONS").is_ok() ||
    std::env::var("CI").is_ok()
}

/// Get build information at runtime
pub fn get_build_info() -> BuildInfo {
    // Check if we're in a CI environment
    if is_ci_environment() {
        // CI environment detected
        let release = option_env!("DOUGU_RELEASE")
            .or_else(|| option_env!("RELEASE"))
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);

        let run_number = if let Some(number) = option_env!("GITHUB_RUN_NUMBER") {
            number.to_string()
        } else if let Ok(number) = std::env::var("GITHUB_RUN_NUMBER") {
            number
        } else {
            // Fallback for CI environments without run number
            format!("ci-{}", Utc::now().format("%Y%m%dT%H%M%SZ"))
        };

        BuildInfo::new_ci(&run_number, release)
    } else {
        // If environment variables are set, use them
        if let (Some(release_str), Some(build_type), Some(timestamp)) = (
            option_env!("DOUGU_RELEASE"),
            option_env!("DOUGU_BUILD_TYPE"),
            option_env!("DOUGU_BUILD_TIMESTAMP")
        ) {
            if let Ok(release) = release_str.parse::<u32>() {
                // Build ID from timestamp if not in CI
                let build_id = format!("0-dev+{}", timestamp);
                
                return BuildInfo {
                    release,
                    build_type: build_type.to_string(),
                    build_id,
                    timestamp: timestamp.to_string(),
                };
            }
        }
        
        // Fallback to local development defaults
        BuildInfo::new_local()
    }
} 