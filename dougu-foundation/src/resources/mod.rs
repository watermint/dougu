// Resources module - Contains all resource files

// Re-export all resource modules
pub mod i18n;

// Error messages
pub mod error_messages {
    pub const ERROR_STDIN_READ: &str = "Failed to read from stdin";
    pub const ERROR_FILE_NOT_FOUND: &str = "File not found";
    pub const ERROR_INVALID_FORMAT: &str = "Invalid format";
    pub const ERROR_INVALID_QUERY: &str = "Invalid query";
}

// Log messages
pub mod log_messages {
    pub const ACTION_START: &str = "Starting {} action";
    pub const ACTION_COMPLETE: &str = "Completed {} action";
    pub const COMMAND_START: &str = "Starting {} command";
    pub const COMMAND_COMPLETE: &str = "Completed {} command";
    pub const SUBCOMMAND_START: &str = "Starting {} subcommand";
    pub const SUBCOMMAND_COMPLETE: &str = "Completed {} subcommand";
}

// UI messages
pub mod ui_messages {
    pub const FORMAT_OPTION_DESCRIPTION: &str = "Output format (default, jsonl, markdown)";
    pub const HEADING_LEVEL_ERROR: &str = "Invalid heading level";
    pub const THEME_COLOR_ERROR: &str = "Invalid theme color";
} 