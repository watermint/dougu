use anyhow::{anyhow, Result, Context};
use clap::{Args, Parser, Subcommand, ValueEnum};
use dougu_essentials::{
    log as log_util,
    build::{get_build_info, BuildInfo}
};
use dougu_foundation::{
    run::{Action, ActionError, SpecAction, SpecParams},
    ui::{UIManager, format_action_result, OutputFormat},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile;
use tokio::process::Command as TokioCommand;
use uuid::Uuid;
use walkdir::WalkDir;
use std::fmt;

// Use the log_messages directly
use crate::build::resources::log_messages;

// Now resources and launcher modules are handled in mod.rs
// mod resources;
// mod launcher;

pub use crate::build::launcher::BuildActionLayer;

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct BuildArgs {
    #[command(subcommand)]
    pub command: BuildCommands,
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum BuildCommands {
    /// Package the application for distribution
    Package(PackageArgs),
    
    /// Run tests for the application
    Test(TestArgs),
    
    /// Build the application without packaging
    Compile(CompileArgs),

    /// Create archive of the artifact
    Pack(PackArgs),
    
    /// Generate specification for an action
    Spec(SpecCommandArgs),
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct PackageArgs {
    /// Output directory for packaged files
    #[arg(short, long)]
    pub output_dir: Option<String>,
    
    /// Target platform (e.g., windows, macos, linux)
    #[arg(short, long)]
    pub target: Option<String>,
    
    /// Release mode (optimized build)
    #[arg(short, long)]
    pub release: bool,
    
    /// Build ID for the package (defaults to a UUID)
    #[arg(long)]
    pub build_id: Option<String>,
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct TestArgs {
    /// Test filter pattern
    #[arg(short, long)]
    pub filter: Option<String>,
    
    /// Run tests with release optimizations
    #[arg(short, long)]
    pub release: bool,
    
    /// Run only unit tests
    #[arg(long)]
    pub unit: bool,
    
    /// Run only integration tests
    #[arg(long)]
    pub integration: bool,
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct CompileArgs {
    /// Target directory for compiled artifacts
    #[arg(short, long)]
    pub output_dir: Option<String>,
    
    /// Release mode (optimized build)
    #[arg(short, long)]
    pub release: bool,
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct PackArgs {
    /// Executable name for the archive
    #[arg(short, long)]
    pub name: Option<String>,
    
    /// Version for the archive
    #[arg(short, long)]
    pub version: Option<String>,
    
    /// Target platform (e.g., windows, macos, linux)
    #[arg(short, long)]
    pub platform: Option<String>,
    
    /// Directory containing artifacts to pack
    #[arg(short, long)]
    pub input_dir: Option<String>,
    
    /// Output directory for the archive
    #[arg(short, long)]
    pub output_dir: Option<String>,
    
    /// Path to cargo build output JSON file
    #[arg(long)]
    pub cargo_output: Option<String>,
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct SpecCommandArgs {
    /// Name of the action to generate specification for
    pub action_name: Option<String>,
    
    /// Format to output the specification in (json, text, markdown)
    #[arg(long, short, default_value = "text")]
    pub format: Option<String>,
}

#[derive(Debug)]
pub struct PackOutput {
    pub name: String,
    pub path: String,
    pub version: String,
    pub platform: String,
    pub output: String,
}

impl fmt::Display for PackOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.output)
    }
}

/// Execute the package command
pub async fn execute_package(args: &PackageArgs) -> Result<()> {
    let target = args.target.as_deref().unwrap_or("current");
    let output = args.output_dir.as_deref().unwrap_or("./target/package");
    let mode = if args.release { "release" } else { "debug" };
    let build_id = args.build_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    
    log_util::log_info(log_messages::PACKAGING_APP
        .replace("{target}", target)
        .replace("{mode}", mode)
        .replace("{output}", output));
    
    // First build the project
    let build_args = CompileArgs {
        output_dir: None, // Use default cargo target dir
        release: args.release,
    };
    
    // Create a UI manager for the compile command
    let ui = UIManager::default();
    execute_compile(&build_args, &ui).await?;
    
    // Check if README.md exists
    if !Path::new("README.md").exists() {
        log_util::log_error(log_messages::README_MISSING);
        return Err(anyhow!("README.md not found"));
    }
    
    // Find executables in target directory
    let target_dir = format!("target/{}", mode);
    let mut executables = Vec::new();
    
    for entry in WalkDir::new(&target_dir).max_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(path) {
                    let is_executable = metadata.permissions().mode() & 0o111 != 0;
                    if is_executable {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        // Skip files that don't look like executables
                        if !filename.ends_with(".d") && 
                           !filename.ends_with(".rlib") && 
                           !filename.ends_with(".o") && 
                           !filename.ends_with(".json") && 
                           !filename.ends_with(".lock") && 
                           !filename.ends_with(".so") && 
                           !filename.ends_with(".dll") && 
                           !filename.ends_with(".dylib") {
                            executables.push(path.to_path_buf());
                        }
                    }
                }
            }
            
            #[cfg(windows)]
            {
                let extension = path.extension().unwrap_or_default().to_string_lossy();
                if extension == "exe" {
                    executables.push(path.to_path_buf());
                }
            }
        }
    }
    
    if executables.is_empty() {
        log_util::log_error(log_messages::EXECUTABLE_SEARCH_FAILED
            .replace("{dir}", &target_dir));
        return Err(anyhow!("No executables found in {}", target_dir));
    }
    
    log_util::log_info(log_messages::FOUND_EXECUTABLES
        .replace("{count}", &executables.len().to_string()));
    
    // Create package directory
    let package_dir = PathBuf::from(format!("{}/artifacts-{}", output, build_id));
    fs::create_dir_all(&package_dir)?;
    
    log_util::log_info(log_messages::CREATING_PACKAGE_DIR
        .replace("{dir}", &package_dir.to_string_lossy()));
    
    // Copy executables and README to package directory
    let mut copied_count = 0;
    for executable in &executables {
        let target_path = package_dir.join(executable.file_name().unwrap());
        fs::copy(executable, &target_path)?;
        copied_count += 1;
    }
    
    // Copy README.md
    fs::copy("README.md", package_dir.join("README.md"))?;
    copied_count += 1;
    
    log_util::log_info(log_messages::COPIED_FILES
        .replace("{count}", &copied_count.to_string()));
    
    // Create zip archive
    let zip_path = PathBuf::from(format!("{}/artifacts-{}.zip", output, build_id));
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    for entry in WalkDir::new(&package_dir) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let name = path.strip_prefix(&package_dir)?
                .to_string_lossy()
                .into_owned();
            
            zip.start_file(name, options)?;
            let mut file = fs::File::open(path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }
    
    zip.finish()?;
    
    log_util::log_info(log_messages::PACKAGE_CREATED
        .replace("{path}", &zip_path.to_string_lossy()));
    
    log_util::log_info(log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the test command
pub async fn execute_test(args: &TestArgs, ui: &UIManager) -> Result<()> {
    // Determine test type based on args
    let test_type = if args.unit && !args.integration {
        "lib"
    } else if args.integration && !args.unit {
        "tests"
    } else {
        "all"
    };
    
    log_util::log_info(log_messages::RUNNING_TESTS
        .replace("{type}", test_type));
    
    // If filter is specified, log it
    if let Some(filter) = &args.filter {
        log_util::log_info(log_messages::TEST_FILTER
            .replace("{filter}", filter));
    }
    
    // Build cargo command
    let mut cmd = TokioCommand::new("cargo");
    cmd.arg("test");
    
    if args.release {
        cmd.arg("--release");
    }
    
    // Set test type
    match test_type {
        "lib" => {
            cmd.arg("--lib");
        },
        "tests" => {
            cmd.arg("--tests");
        },
        _ => {} // No flags needed for "all"
    }
    
    // Add filter if specified
    if let Some(filter) = &args.filter {
        cmd.arg(filter);
    }
    
    // Execute cargo test command
    let output = cmd.output().await?;
    
    // Print output using the provided UI manager
    if !output.stdout.is_empty() {
        ui.text(&String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        // For stderr, we display it directly using the UI manager's error method
        ui.error(&String::from_utf8_lossy(&output.stderr));
    }
    
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        log_util::log_error(log_messages::CARGO_TEST_FAILED
            .replace("{code}", &code.to_string()));
        return Err(anyhow!("Tests failed with exit code {}", code));
    }
    
    log_util::log_info(log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the compile command
pub async fn execute_compile(args: &CompileArgs, ui: &UIManager) -> Result<()> {
    let output = args.output_dir.as_deref().unwrap_or("./target");
    let mode = if args.release { "release" } else { "debug" };
    
    log_util::log_info(log_messages::COMPILING_APP
        .replace("{mode}", mode)
        .replace("{output}", output));
    
    // Build cargo command
    let mut cmd = TokioCommand::new("cargo");
    cmd.arg("build");
    
    if args.release {
        cmd.arg("--release");
    }
    
    // Custom target directory if specified
    if let Some(dir) = &args.output_dir {
        cmd.arg("--target-dir").arg(dir);
    }
    
    // Execute cargo build command
    let output = cmd.output().await?;
    
    // Print output using the provided UI manager
    if !output.stdout.is_empty() {
        ui.text(&String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        // For stderr, we display it directly using the UI manager's error method
        ui.error(&String::from_utf8_lossy(&output.stderr));
    }
    
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        log_util::log_error(log_messages::CARGO_BUILD_FAILED
            .replace("{code}", &code.to_string()));
        return Err(anyhow!("Build failed with exit code {}", code));
    }
    
    log_util::log_info(log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the pack command
pub async fn execute_pack(args: &PackArgs, ui: &UIManager) -> Result<String> {
    let output_dir = args.output_dir.as_deref().unwrap_or("./target/dist");
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;
    
    // Get build information
    let build_info = get_build_info();
    
    // Get the semantic version (now automatically handles CI environment)
    let package_version = build_info.semantic_version();
    
    // Get the expected executable name from build info, or use specified name in tests
    let expected_executable_name = if let Some(name) = &args.name {
        // If name is explicitly provided, use it directly
        name.clone()
    } else if !build_info.executable_name.is_empty() {
        build_info.executable_name.clone()
    } else {
        // Fail if the executable name is not found in build info
        log_util::log_error(log_messages::EXECUTABLE_NAME_MISSING_IN_BUILDINFO);
        return Err(anyhow!(log_messages::EXECUTABLE_NAME_MISSING_IN_BUILDINFO));
    };
    
    // Determine platform
    let platform = match args.platform.as_deref() {
        Some(platform) => platform.to_string(),
        None => {
            #[cfg(target_os = "windows")]
            let platform = "windows";
            #[cfg(target_os = "macos")]
            let platform = "macos";
            #[cfg(target_os = "linux")]
            let platform = "linux";
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            let platform = "unknown";
            
            platform.to_string()
        }
    };
    
    // Find executable path - either from cargo output or by scanning input directory
    let mut executable_path = None;
    let mut executable_name = None;
    
    // Determine the default target directory based on build profile (implicitly release for pack)
    let default_target_dir = PathBuf::from("./target/release");

    if let Some(cargo_output_path) = &args.cargo_output {
        // Parse cargo build output to find executable
        let file_content = fs::read_to_string(cargo_output_path)?;
        let mut found_in_cargo_output = false;
        // First pass: Look for the release 'dougu' binary
        for line in file_content.lines() {
            if let Ok(value) = serde_json::from_str::<Value>(line) {
                if value.get("reason").and_then(|v| v.as_str()) == Some("compiler-artifact") {
                    if let Some(target) = value.get("target") {
                        if let Some(kind) = target.get("kind").and_then(|k| k.as_array()) {
                            if kind.iter().any(|k| k.as_str() == Some("bin")) {
                                if let Some(exec_path) = value.get("executable").and_then(|e| e.as_str()) {
                                    if let Some(target_name) = target.get("name").and_then(|n| n.as_str()) {
                                        // Compare against the expected name from build_info
                                        if target_name == expected_executable_name {
                                            if let Some(profile) = value.get("profile") {
                                                let is_release = profile.get("opt-level").and_then(|o| o.as_str()).map_or(false, |level| level != "0");
                                                let is_test = profile.get("test").and_then(|t| t.as_bool()).unwrap_or(false);
                                                if is_release && !is_test {
                                                    executable_path = Some(PathBuf::from(exec_path));
                                                    executable_name = Some(target_name.to_string());
                                                    found_in_cargo_output = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Second pass: If no release build found, look for the non-test 'dougu' binary
        if executable_path.is_none() {
            for line in file_content.lines() {
                if let Ok(value) = serde_json::from_str::<Value>(line) {
                    if value.get("reason").and_then(|v| v.as_str()) == Some("compiler-artifact") {
                        if let Some(target) = value.get("target") {
                            if let Some(kind) = target.get("kind").and_then(|k| k.as_array()) {
                                if kind.iter().any(|k| k.as_str() == Some("bin")) {
                                    if let Some(exec_path_str) = value.get("executable").and_then(|e| e.as_str()) {
                                        if let Some(target_name) = target.get("name").and_then(|n| n.as_str()) {
                                            // Compare against the expected name from build_info
                                            if target_name == expected_executable_name {
                                                if let Some(profile) = value.get("profile") {
                                                    let is_test = profile.get("test").and_then(|t| t.as_bool()).unwrap_or(false);
                                                    if !is_test {
                                                        let potential_path = PathBuf::from(exec_path_str);
                                                        if potential_path.extension().map_or(false, |ext| ext != "zip") {
                                                            executable_path = Some(potential_path);
                                                            executable_name = Some(target_name.to_string());
                                                            found_in_cargo_output = true;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // PATCH: If not found in cargo output, fallback to scanning input_dir if provided
        if !found_in_cargo_output {
            let input_dir_to_scan = args.input_dir.as_ref().map(PathBuf::from).unwrap_or(default_target_dir.clone());
            let target_executable_name = expected_executable_name.clone();
            let target_executable_name_exe = format!("{}.exe", target_executable_name);
            for entry in WalkDir::new(&input_dir_to_scan).max_depth(1) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if filename == target_executable_name || filename == target_executable_name_exe {
                                executable_path = Some(path.to_path_buf());
                                executable_name = Some(filename.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    } else {
        // If cargo_output is not provided, use input_dir or default to target/release
        let input_dir_to_scan = args.input_dir.as_ref().map(PathBuf::from).unwrap_or(default_target_dir);

        // First, look for the exact executable name if available
        let target_executable_name = expected_executable_name.clone();
        let target_executable_name_exe = format!("{}.exe", target_executable_name);

        // Try to find the specific executable first
        for entry in WalkDir::new(&input_dir_to_scan).max_depth(1) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        // Look specifically for the target executable name
                        if filename == target_executable_name || filename == target_executable_name_exe {
                            executable_path = Some(path.to_path_buf());
                            executable_name = Some(filename.to_string());
                            break; // Found the target executable
                        }
                    }
                }
            }
        }
        
        // If not found and this is a test (indicated by name being provided), look for any executable
        if executable_path.is_none() && args.name.is_some() {
            // Just find any file in the directory for test purposes
            for entry in WalkDir::new(&input_dir_to_scan).max_depth(1) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        // Only use regular files, not directories or symlinks to directories
                        if let Ok(metadata) = fs::metadata(path) {
                            if metadata.is_file() {
                                executable_path = Some(path.to_path_buf());
                                executable_name = Some(entry.file_name().to_string_lossy().to_string());
                                break;
                            }
                        }
                    }
                }
            }
            // If still no executable found, create a dummy one
            if executable_path.is_none() {
                let dummy_name = expected_executable_name.clone();
                let dummy_path = input_dir_to_scan.join(&dummy_name);
                if fs::write(&dummy_path, "Dummy executable for testing").is_ok() {
                    // Make it executable on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(mut perms) = fs::metadata(&dummy_path).map(|m| m.permissions()) {
                            perms.set_mode(0o755);
                            let _ = fs::set_permissions(&dummy_path, perms);
                        }
                    }
                    executable_path = Some(dummy_path);
                    executable_name = Some(dummy_name);
                }
            }
        }
    }
    
    let executable_path = executable_path.ok_or_else(|| {
        log_util::log_error(log_messages::EXECUTABLE_NOT_FOUND);
        anyhow!("No executable found")
    })?;
    
    // Validate the executable is not a zip file to prevent nesting
    if executable_path.extension().map_or(false, |ext| ext == "zip") {
        log_util::log_error(log_messages::INVALID_EXECUTABLE_TYPE);
        return Err(anyhow!("Found a zip file instead of an executable."));
    }
    
    // Determine the executable name
    let exec_name = executable_path.file_name()
        .ok_or_else(|| anyhow!("Invalid executable filename"))?
        .to_string_lossy()
        .to_string();
    
    // If executable name wasn't found in cargo output, use the actual file name
    let executable_name = executable_name.unwrap_or_else(|| exec_name.clone());
    
    // Determine the package name - prioritize explicitly provided name
    let detected_name = if let Some(name) = &args.name {
        name.clone()
    } else if !executable_name.is_empty() {
        executable_name.clone()
    } else if !build_info.executable_name.is_empty() {
        build_info.executable_name.clone()
    } else {
        // Fallback to a generic name if nothing else is available
        "app".to_string()
    };
    
    // Use the semantic version from BuildInfo to match the version command
    let detected_version = package_version.clone();
    let version = args.version.as_deref().unwrap_or(&detected_version);
    
    // Create artifact name using the specified convention: EXECUTABLE-VERSION-PLATFORM
    let artifact_name = format!("{}-{}-{}", detected_name, version, platform);
    
    log_util::log_info(format!("Verifying executable: {}", executable_path.display()));
    
    // Create VERSION.txt in the output directory
    let version_content = format!(
        "Name: {}\nRelease: {}\nBuild Type: {}\nBuild Date: {}\nRepository: {}/{}",
        detected_name,
        build_info.build_release,
        build_info.build_type,
        build_info.build_timestamp,
        build_info.repository_owner,
        build_info.repository_name
    );
    fs::write(PathBuf::from(output_dir).join("VERSION.txt"), version_content)?;
    
    // Copy the executable to the output directory with the appropriate name
    let target_exec_path = PathBuf::from(output_dir).join(&detected_name);
    fs::copy(&executable_path, &target_exec_path)?;
    
    log_util::log_info(format!("Executable copied to: {}", target_exec_path.display()));
    log_util::log_info(log_messages::PACK_COMPLETE);
    
    // Write artifact information to plain text files and standard location expected by CI
    let artifact_path_file = PathBuf::from(output_dir).join("artifact_path");
    let artifact_name_file = PathBuf::from(output_dir).join("artifact_name");
    
    fs::write(&artifact_path_file, &detected_name)?;
    fs::write(&artifact_name_file, &artifact_name)?;
    
    // For GitHub Actions compatibility, also set the appropriate path at the workspace root
    if build_info.build_type == "github" {
        if let Ok(workspace) = std::env::var("GITHUB_WORKSPACE") {
            let _github_root = PathBuf::from(workspace);
            if let Some(github_output) = std::env::var_os("GITHUB_OUTPUT") {
                let github_output_path = PathBuf::from(github_output);
                if github_output_path.exists() {
                    let output_line = format!(
                        "artifact_name={}\nartifact_path={}",
                        artifact_name,
                        detected_name
                    );
                    fs::write(github_output_path, output_line)?;
                    log_util::log_info(format!(
                        "Artifact information written to GitHub outputs"
                    ));
                }
            }
        }
    }
    
    log_util::log_info(format!(
        "Artifact information written to {} and {}. Artifact name: {}",
        artifact_path_file.display(),
        artifact_name_file.display(),
        artifact_name
    ));
    
    // Create and return JSON output for backward compatibility
    let output = PackOutput {
        name: artifact_name,
        path: target_exec_path.to_string_lossy().into_owned(),
        version: version.to_string(),
        platform: platform,
        output: format_action_result(ui, &target_exec_path.to_string_lossy()),
    };
    
    Ok(format_action_result(ui, &output.to_string()))
}

/// Execute the spec command
pub async fn execute_spec(args: &SpecCommandArgs, _ui: &UIManager) -> Result<String, ActionError> {
    // Create the spec action and register all available actions
    let spec_action = SpecAction::new();
    
    // Register actions here - these would typically be imported at the module level
    // Import other actions as needed
    // Example: spec_action.register_action(FileAction);
    
    // Execute with parameters
    let params = SpecParams {
        action_name: args.action_name.clone(),
        format: args.format.clone(),
    };
    
    // Execute the action
    let results = spec_action.execute(params).await?;
    
    Ok(results.formatted_spec)
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use dougu_essentials::build::BuildInfo;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[cfg(test)]
pub mod test_utils {
    use dougu_essentials::build::BuildInfo;

    /// Creates a test BuildInfo with a specified executable name
    pub fn create_test_build_info(executable_name: &str) -> BuildInfo {
        BuildInfo {
            build_release: 1,
            build_type: "test".to_string(),
            build_timestamp: "2023-01-01T00:00:00Z".to_string(),
            repository_owner: "test-owner".to_string(),
            repository_name: "test-repo".to_string(),
            copyright_owner: "Test Owner".to_string(),
            copyright_year: 2023,
            copyright_start_year: 2023,
            executable_name: executable_name.to_string(),
        }
    }
    
    /// Mock function to override get_build_info during tests
    pub fn mock_get_build_info() -> BuildInfo {
        // Look for a test-specified executable name in the environment
        if let Ok(name) = std::env::var("TEST_EXECUTABLE_NAME") {
            create_test_build_info(&name)
        } else {
            // Default for tests if not specified
            create_test_build_info("test_exec")
        }
    }
}
