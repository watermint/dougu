use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::process::Command;
use uuid::Uuid;
use walkdir::WalkDir;
use dougu_essentials_build::{BuildInfo, get_build_info};

mod resources;

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
}

/// Execute the package command
pub async fn execute_package(args: &PackageArgs) -> Result<()> {
    let target = args.target.as_deref().unwrap_or("current");
    let output = args.output_dir.as_deref().unwrap_or("./target/package");
    let mode = if args.release { "release" } else { "debug" };
    let build_id = args.build_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    
    dougu_essentials_logger::log_info(resources::log_messages::PACKAGING_APP
        .replace("{target}", target)
        .replace("{mode}", mode)
        .replace("{output}", output));
    
    // First build the project
    let build_args = CompileArgs {
        output_dir: None, // Use default cargo target dir
        release: args.release,
    };
    
    // Create a UI manager for the compile command
    let ui = dougu_foundation_ui::UIManager::default();
    execute_compile(&build_args, &ui).await?;
    
    // Check if README.md exists
    if !Path::new("README.md").exists() {
        dougu_essentials_logger::log_error(resources::log_messages::README_MISSING);
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
        dougu_essentials_logger::log_error(resources::log_messages::EXECUTABLE_SEARCH_FAILED
            .replace("{dir}", &target_dir));
        return Err(anyhow!("No executables found in {}", target_dir));
    }
    
    dougu_essentials_logger::log_info(resources::log_messages::FOUND_EXECUTABLES
        .replace("{count}", &executables.len().to_string()));
    
    // Create package directory
    let package_dir = PathBuf::from(format!("{}/artifacts-{}", output, build_id));
    fs::create_dir_all(&package_dir)?;
    
    dougu_essentials_logger::log_info(resources::log_messages::CREATING_PACKAGE_DIR
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
    
    dougu_essentials_logger::log_info(resources::log_messages::COPIED_FILES
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
    
    dougu_essentials_logger::log_info(resources::log_messages::PACKAGE_CREATED
        .replace("{path}", &zip_path.to_string_lossy()));
    
    dougu_essentials_logger::log_info(resources::log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the test command
pub async fn execute_test(args: &TestArgs, ui: &dougu_foundation_ui::UIManager) -> Result<()> {
    dougu_essentials_logger::log_info(resources::log_messages::RUNNING_TESTS);
    
    // Build cargo command
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    
    if args.release {
        cmd.arg("--release");
    }
    
    // Apply test filter if specified
    if let Some(filter) = &args.filter {
        cmd.arg(filter);
    }
    
    // Unit or integration tests
    match (args.unit, args.integration) {
        (true, false) => {
            cmd.arg("--lib");
        },
        (false, true) => {
            cmd.arg("--test=*");
        },
        _ => {}, // Run all tests
    }
    
    // Execute cargo test command
    let output = cmd.output().await?;
    
    // Print output using the provided UI manager
    if !output.stdout.is_empty() {
        ui.print(&String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        // For stderr, we still want to use the error stream
        // but format it using UI manager
        let error_text = ui.error(&String::from_utf8_lossy(&output.stderr));
        eprintln!("{}", error_text);
    }
    
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        dougu_essentials_logger::log_error(resources::log_messages::CARGO_TEST_FAILED
            .replace("{code}", &code.to_string()));
        return Err(anyhow!("Tests failed with exit code {}", code));
    }
    
    dougu_essentials_logger::log_info(resources::log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the compile command
pub async fn execute_compile(args: &CompileArgs, ui: &dougu_foundation_ui::UIManager) -> Result<()> {
    let output = args.output_dir.as_deref().unwrap_or("./target");
    let mode = if args.release { "release" } else { "debug" };
    
    dougu_essentials_logger::log_info(resources::log_messages::COMPILING_APP
        .replace("{mode}", mode)
        .replace("{output}", output));
    
    // Build cargo command
    let mut cmd = Command::new("cargo");
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
        ui.print(&String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        // For stderr, we still want to use the error stream
        // but format it using UI manager
        let error_text = ui.error(&String::from_utf8_lossy(&output.stderr));
        eprintln!("{}", error_text);
    }
    
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        dougu_essentials_logger::log_error(resources::log_messages::CARGO_BUILD_FAILED
            .replace("{code}", &code.to_string()));
        return Err(anyhow!("Build failed with exit code {}", code));
    }
    
    dougu_essentials_logger::log_info(resources::log_messages::BUILD_COMPLETE);
    
    Ok(())
}

/// Execute the pack command
pub async fn execute_pack(args: &PackArgs) -> Result<()> {
    let input_dir = args.input_dir.as_deref().unwrap_or("./target/package");
    let output_dir = args.output_dir.as_deref().unwrap_or("./target/dist");
    
    // Get build information
    let build_info = get_build_info();
    
    // Determine platform if not specified
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
    
    // Find executables in input directory
    let mut executables = Vec::new();
    for entry in WalkDir::new(input_dir).max_depth(1) {
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
        dougu_essentials_logger::log_error(resources::log_messages::EXECUTABLE_SEARCH_FAILED
            .replace("{dir}", input_dir));
        return Err(anyhow!("No executables found in {}", input_dir));
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;
    
    // Process each executable
    for executable_path in executables {
        let exec_name = executable_path.file_name()
            .ok_or_else(|| anyhow!("Invalid executable filename"))?
            .to_string_lossy()
            .to_string();
        
        // Use build_info for name detection
        let detected_name = if !build_info.executable_name.is_empty() {
            build_info.executable_name.clone()
        } else {
            exec_name.clone()
        };
        
        // Use build_info for version detection
        let detected_version = build_info.build_release.to_string();
        
        // Use provided name or detected name
        let name = args.name.as_deref().unwrap_or(&detected_name);
        
        // Use provided version or detected version
        let version = args.version.as_deref().unwrap_or(&detected_version);
        
        // Create archive name using the specified convention
        let archive_name = format!("{}-{}-{}.zip", name, version, platform);
        let archive_path = PathBuf::from(output_dir).join(&archive_name);
        
        dougu_essentials_logger::log_info(resources::log_messages::PACKING_ARTIFACT
            .replace("{name}", &archive_name));
        
        // Create the zip file
        let zip_file = fs::File::create(&archive_path)?;
        let mut zip = zip::ZipWriter::new(zip_file);
        
        // Set file options (executable permissions for binaries)
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        // Add the executable to the archive
        zip.start_file(exec_name, options)?;
        let mut file = fs::File::open(&executable_path)?;
        std::io::copy(&mut file, &mut zip)?;
        
        // Add README.md if it exists in the input directory
        let readme_path = Path::new(input_dir).join("README.md");
        if readme_path.exists() {
            zip.start_file("README.md", options)?;
            let mut file = fs::File::open(readme_path)?;
            std::io::copy(&mut file, &mut zip)?;
        }
        
        // Add VERSION.txt if it exists in the input directory
        let version_path = Path::new(input_dir).join("VERSION.txt");
        if version_path.exists() {
            zip.start_file("VERSION.txt", options)?;
            let mut file = fs::File::open(version_path)?;
            std::io::copy(&mut file, &mut zip)?;
        } else {
            // Generate a VERSION.txt with build info
            let version_content = format!(
                "Name: {}\nRelease: {}\nBuild Type: {}\nBuild Date: {}\nRepository: {}/{}",
                name,
                build_info.build_release,
                build_info.build_type,
                build_info.build_timestamp,
                build_info.repository_owner,
                build_info.repository_name
            );
            zip.start_file("VERSION.txt", options)?;
            std::io::copy(&mut std::io::Cursor::new(version_content.into_bytes()), &mut zip)?;
        }
        
        zip.finish()?;
        
        dougu_essentials_logger::log_info(resources::log_messages::PACKAGE_CREATED
            .replace("{path}", &archive_path.to_string_lossy()));
    }
    
    dougu_essentials_logger::log_info(resources::log_messages::PACK_COMPLETE);
    
    Ok(())
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
