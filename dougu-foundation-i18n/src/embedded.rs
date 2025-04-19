// This module embeds translation resources into the binary
// Each resource is included at compile time as a static string

/// Foundation English resources
pub const FOUNDATION_EN: &str = include_str!("../../dougu-foundation-run/src/resources/i18n-en.json");

/// Foundation Japanese resources
pub const FOUNDATION_JA: &str = include_str!("../../dougu-foundation-run/src/resources/i18n-ja.json");

/// File Command English resources
pub const FILE_COMMAND_EN: &str = include_str!("../../dougu-command-file/src/resources/i18n-en.json");

/// File Command Japanese resources
pub const FILE_COMMAND_JA: &str = include_str!("../../dougu-command-file/src/resources/i18n-ja.json");

/// Root Command English resources
pub const ROOT_COMMAND_EN: &str = include_str!("../../dougu-command-root/src/resources/i18n-en.json");

/// Root Command Japanese resources
pub const ROOT_COMMAND_JA: &str = include_str!("../../dougu-command-root/src/resources/i18n-ja.json");

/// Get embedded resource content by module and locale
pub fn get_resource(module: &str, locale: &str) -> Option<&'static str> {
    match (module, locale) {
        ("foundation", "en") => Some(FOUNDATION_EN),
        ("foundation", "ja") => Some(FOUNDATION_JA),
        ("file", "en") => Some(FILE_COMMAND_EN),
        ("file", "ja") => Some(FILE_COMMAND_JA),
        ("root", "en") => Some(ROOT_COMMAND_EN),
        ("root", "ja") => Some(ROOT_COMMAND_JA),
        _ => None,
    }
}

/// Get a list of all available modules
pub fn available_modules() -> Vec<&'static str> {
    vec!["foundation", "file", "root"]
}

/// Get a list of all available locales
pub fn available_locales() -> Vec<&'static str> {
    vec!["en", "ja"]
}

/// Struct to represent a module resource
pub struct ModuleResource {
    pub module: &'static str,
    pub locale: &'static str,
    pub content: &'static str,
}

/// Get all available resources
pub fn all_resources() -> Vec<ModuleResource> {
    vec![
        ModuleResource { module: "foundation", locale: "en", content: FOUNDATION_EN },
        ModuleResource { module: "foundation", locale: "ja", content: FOUNDATION_JA },
        ModuleResource { module: "file", locale: "en", content: FILE_COMMAND_EN },
        ModuleResource { module: "file", locale: "ja", content: FILE_COMMAND_JA },
        ModuleResource { module: "root", locale: "en", content: ROOT_COMMAND_EN },
        ModuleResource { module: "root", locale: "ja", content: ROOT_COMMAND_JA },
    ]
} 