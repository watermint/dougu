use dougu_foundation_ui::{UIManager, OutputFormat};
use dougu_essentials_build::get_build_info;
use serde::{Serialize, Deserialize};

/// Application info for JSON serialization
#[derive(Serialize, Deserialize)]
struct AppInfoJson {
    app_name: String,
    version: String,
    copyright: String,
    license: String,
    banner_type: String,
}

/// Display application information banner at startup
pub fn display_app_info(ui: &UIManager, _verbose: bool) {
    // Get build information
    let build_info = get_build_info();
    
    // Create a formatted version string using the build information
    // First, get the semantic version string
    let semantic_version = build_info.semantic_version();
    
    // Split to extract just the version part without build metadata
    let version_components: Vec<&str> = semantic_version.split('+').collect();
    let version = version_components[0];
    
    // Format the application name with version
    let app_title = format!("{} {}", 
        build_info.executable_name, 
        version
    );
    
    // Create separator line of equal length to the app title
    let separator = "=".repeat(app_title.chars().count());
    
    // Format the copyright information using the start and current year
    // If start year equals current year, show only one year
    let copyright = if build_info.copyright_start_year == build_info.copyright_year {
        format!("© {} {}", 
            build_info.copyright_year, 
            build_info.copyright_owner
        )
    } else {
        format!("© {}-{} {}", 
            build_info.copyright_start_year,
            build_info.copyright_year, 
            build_info.copyright_owner
        )
    };
    
    // License information
    let license = "Licensed under open source licenses. Use the `license` command for more detail.";
    
    // Check if we should output in JSON format
    if ui.format() == OutputFormat::JsonLines {
        // Create JSON structure for the app info
        let app_info_json = AppInfoJson {
            app_name: build_info.executable_name.clone(),
            version: build_info.semantic_version(),
            copyright: copyright.clone(),
            license: license.to_string(),
            banner_type: "app_info".to_string()
        };
        
        // Serialize to JSON and print
        if let Ok(_) = ui.jsonl(&app_info_json) {
            // Output handled by jsonl method
        }
    } else {
        // Display the application banner in text format
        ui.heading(1, &app_title);
        ui.text(&separator);
        ui.line_break(); // Empty line for spacing
        ui.text(&copyright);
        ui.text(license);
        ui.line_break(); // Add break line after the app info
    }
} 