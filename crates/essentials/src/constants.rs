/// Application name
pub const APP_NAME: &str = "db";

/// Default configuration directory
pub fn get_config_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join(APP_NAME)
}

/// Default data directory
pub fn get_data_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".local").join("share").join(APP_NAME)
} 