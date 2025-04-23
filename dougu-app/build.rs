use std::env;
use std::process::Command;
use chrono::Utc;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Generate and print build timestamp - will be available at compile time
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    println!("cargo:rustc-env=DOUGU_BUILD_TIMESTAMP={}", timestamp);
    
    // Format timestamp for display
    let human_timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // Set default build type - will be overridden if in CI
    println!("cargo:rustc-env=DOUGU_BUILD_TYPE=local");
    
    // Pass through release version if specified
    if let Ok(release) = env::var("DOUGU_RELEASE") {
        println!("cargo:rustc-env=DOUGU_RELEASE={}", release);
    }
    
    // Pass through executable name if specified
    if let Ok(executable_name) = env::var("DOUGU_EXECUTABLE_NAME") {
        println!("cargo:rustc-env=DOUGU_EXECUTABLE_NAME={}", executable_name);
    }
    
    // Try to get Rust version
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            let rustc_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version);
        }
    }
    
    // Detect CI environment
    let is_ci = env::var("GITHUB_ACTIONS").is_ok() || env::var("CI").is_ok();
    let build_type = if is_ci { "github" } else { "local" };
    
    // Get or generate build ID
    let build_id = if let Ok(run_number) = env::var("GITHUB_RUN_NUMBER") {
        run_number
    } else {
        format!("0-dev+{}", timestamp)
    };
    
    // Show build info for debugging
    println!(
        "cargo:warning=Build info: 0.{}.{} ({})",
        build_type,
        build_id,
        human_timestamp
    );
} 