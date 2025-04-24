// Foundation crate - Main entry point
// Re-exports all submodules

// UI module
pub mod ui;

// I18n module
pub mod i18n;

// Run module
pub mod run;

// Resources module
pub mod resources;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Test UI module
        let theme = ui::UITheme::default();
        assert_eq!(theme.heading_color, "blue");
        
        // Test i18n module
        let locale = i18n::Locale::new("en");
        assert_eq!(locale.language(), "en");
        
        // Test run module
        let spec = run::CommandletSpec {
            name: "test".to_string(),
            description: Some("Test commandlet".to_string()),
            behavior: "Test behavior".to_string(),
            options: Vec::new(),
            parameters: Vec::new(),
            result_types: Vec::new(),
            errors: Vec::new(),
        };
        assert_eq!(spec.name, "test");
    }
} 