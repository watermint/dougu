use serde::{Deserialize, Serialize};
use chrono::{Utc, Datelike};

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
        
        // For patch version: use 0 for CI, derive from timestamp for local
        let patch = if self.build_type == "github" {
            0
        } else {
            // Use day of year as patch for local builds
            let now = Utc::now();
            now.ordinal() as u32 % 1000 // Day of year mod 1000
        };
        
        // Create build metadata from build_type and timestamp
        let build_metadata = format!("{}.{}", 
            self.build_type,
            self.build_timestamp.replace(" ", "").replace(":", "").replace("-", "")
        );
        
        format!("{}.{}.{}+{}", self.build_release, minor, patch, build_metadata)
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
    pub fn new_local() -> Self {
        let build_timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
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
    pub fn new_ci(_run_number: &str, build_release: u32) -> Self {
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
            build_timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            repository_owner,
            repository_name,
            copyright_owner,
            copyright_year,
            copyright_start_year: 2025,
            executable_name,
        }
    }
}

fn detect_repository() -> (String, String) {
    // Try GitHub Actions environment variables
    if let (Ok(repo), Ok(_)) = (std::env::var("GITHUB_REPOSITORY"), std::env::var("GITHUB_ACTIONS")) {
        let mut parts = repo.splitn(2, '/');
        let owner = parts.next().unwrap_or("").to_string();
        let name = parts.next().unwrap_or("").to_string();
        if !owner.is_empty() && !name.is_empty() {
            return (owner, name);
        }
    }
    // Try GitLab CI environment variables
    if let (Ok(owner), Ok(name)) = (std::env::var("CI_PROJECT_NAMESPACE"), std::env::var("CI_PROJECT_NAME")) {
        if !owner.is_empty() && !name.is_empty() {
            return (owner, name);
        }
    }
    // Try Bitbucket Pipelines environment variables
    if let (Ok(owner), Ok(name)) = (std::env::var("BITBUCKET_REPO_OWNER"), std::env::var("BITBUCKET_REPO_SLUG")) {
        if !owner.is_empty() && !name.is_empty() {
            return (owner, name);
        }
    }
    // Default fallback
    ("watermint".to_string(), "dougu".to_string())
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
        let build_release = {
            if let Some(release) = option_env!("DOUGU_RELEASE") {
                release.parse::<u32>().unwrap_or(0)
            } else if let Some(release) = option_env!("RELEASE") {
                release.parse::<u32>().unwrap_or(0)
            } else if let Ok(release) = std::env::var("DOUGU_RELEASE") {
                release.parse::<u32>().unwrap_or(0)
            } else if let Ok(release) = std::env::var("RELEASE") {
                release.parse::<u32>().unwrap_or(0)
            } else {
                0
            }
        };

        let run_number = if let Some(number) = option_env!("GITHUB_RUN_NUMBER") {
            number.to_string()
        } else if let Ok(number) = std::env::var("GITHUB_RUN_NUMBER") {
            number
        } else {
            // Fallback for CI environments without run number
            format!("ci-{}", Utc::now().format("%Y%m%dT%H%M%SZ"))
        };

        BuildInfo::new_ci(&run_number, build_release)
    } else {
        // If environment variables are set, use them
        if let (Some(release_str), Some(build_type), Some(timestamp)) = (
            option_env!("DOUGU_RELEASE"),
            option_env!("DOUGU_BUILD_TYPE"),
            option_env!("DOUGU_BUILD_TIMESTAMP")
        ) {
            if let Ok(build_release) = release_str.parse::<u32>() {
                let (repository_owner, repository_name) = detect_repository();
                
                // Use environment variables for copyright information if available
                let copyright_owner = option_env!("DOUGU_COPYRIGHT_OWNER")
                    .map(String::from)
                    .unwrap_or_else(|| "Takayuki Okazaki".to_string());
                
                let copyright_year = option_env!("DOUGU_COPYRIGHT_YEAR")
                    .and_then(|y| y.parse::<u32>().ok())
                    .unwrap_or_else(|| Utc::now().year() as u32);
                
                let executable_name = option_env!("DOUGU_EXECUTABLE_NAME")
                    .map(String::from)
                    .unwrap_or_else(|| "dougu".to_string());
                
                return BuildInfo {
                    build_release,
                    build_type: build_type.to_string(),
                    build_timestamp: timestamp.to_string(),
                    repository_owner,
                    repository_name,
                    copyright_owner,
                    copyright_year,
                    copyright_start_year: 2025,
                    executable_name,
                };
            }
        }
        BuildInfo::new_local()
    }
} 