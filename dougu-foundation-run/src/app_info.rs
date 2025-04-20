use dougu_foundation_ui::UIManager;
use dougu_essentials_build::get_build_info;

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
    
    // Display the application banner
    ui.print(&app_title);
    ui.print(&separator);
    ui.line_break(); // Empty line for spacing using line_break
    ui.print(&copyright);
    ui.print(license);
    ui.line_break(); // Add break line after the app info using line_break
} 