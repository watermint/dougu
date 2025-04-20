use anyhow::Result;
use dougu_command_build::PackOutput;
use dougu_foundation_ui::{UIManager, OutputFormat};
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use serde_json::Value;

#[tokio::test]
async fn test_build_pack_json_output() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create input directory and add test files
    fs::create_dir_all(&input_dir)?;
    
    // Create a mock executable
    let mock_executable = if cfg!(windows) {
        input_dir.join("test.exe")
    } else {
        input_dir.join("test")
    };
    fs::write(&mock_executable, "mock executable content")?;
    
    // Make the file executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_executable)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_executable, perms)?;
    }
    
    fs::create_dir_all(&output_dir)?;
    
    // Create UI manager for testing (JSON format)
    let ui = UIManager::with_format(OutputFormat::JsonLines);
    
    // Run the build pack command
    let output = dougu_command_build::execute_pack(&dougu_command_build::PackArgs {
        name: Some("test".to_string()),
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: Some(input_dir.to_string_lossy().into_owned()),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
    }, &ui).await?;
    
    // Parse the JSON output
    let pack_output: PackOutput = serde_json::from_str(&output)?;
    
    // Verify the output structure
    assert_eq!(pack_output.name, "test");
    assert!(pack_output.path.ends_with(".zip")); // Should end with .zip
    assert_eq!(pack_output.version, "1.0.0");
    assert_eq!(pack_output.platform, "test-platform");
    
    // Verify the archive was created
    assert!(fs::metadata(&pack_output.path).is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_build_pack_invalid_input() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let input_dir = temp_dir.path().join("nonexistent");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&output_dir)?;
    
    // Create UI manager for testing (JSON format)
    let ui = UIManager::with_format(OutputFormat::JsonLines);
    
    // Try to run the build pack command with invalid input
    let result = dougu_command_build::execute_pack(&dougu_command_build::PackArgs {
        name: Some("test".to_string()),
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: Some(input_dir.to_string_lossy().into_owned()),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
    }, &ui).await;
    
    // Should return an error
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_pack_basic() {
    // Create temporary directories for testing
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create input directory
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create a mock executable
    let mock_executable = if cfg!(windows) {
        input_dir.join("test_exec.exe")
    } else {
        input_dir.join("test_exec")
    };
    fs::write(&mock_executable, "mock executable content").unwrap();
    
    // Make the file executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_executable).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_executable, perms).unwrap();
    }
    
    fs::create_dir_all(&output_dir).unwrap();
    
    let pack_args = dougu_command_build::PackArgs {
        name: Some("test_pack".to_string()),
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: Some(input_dir.to_string_lossy().into_owned()),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
    };
    
    let ui = UIManager::with_format(OutputFormat::JsonLines);
    
    let result = dougu_command_build::execute_pack(&pack_args, &ui);
    
    // We're running the test in a blocking context, so we need to block_on
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(result);
    
    assert!(result.is_ok(), "Pack operation failed: {:?}", result.err());
    let formatted_result = result.unwrap();
    
    // Parse the JSON output and verify it contains the expected info
    let json: Value = serde_json::from_str(&formatted_result).unwrap();
    assert!(json.is_object());
    assert!(json.get("path").is_some());
    
    // Verify the file exists
    let archive_path = json["path"].as_str().unwrap();
    assert!(Path::new(archive_path).exists());
}

#[test]
fn test_pack_with_different_name() {
    // Create temporary directories for testing
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create input directory
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create a mock executable
    let mock_executable = if cfg!(windows) {
        input_dir.join("test_exec.exe")
    } else {
        input_dir.join("test_exec")
    };
    fs::write(&mock_executable, "mock executable content").unwrap();
    
    // Make the file executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_executable).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_executable, perms).unwrap();
    }
    
    fs::create_dir_all(&output_dir).unwrap();
    
    let pack_args = dougu_command_build::PackArgs {
        name: Some("test_pack_different".to_string()),
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: Some(input_dir.to_string_lossy().into_owned()),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
    };
    
    let ui = UIManager::with_format(OutputFormat::JsonLines);
    
    let result = dougu_command_build::execute_pack(&pack_args, &ui);
    
    // We're running the test in a blocking context, so we need to block_on
    let rt = Runtime::new().unwrap();
    let result = rt.block_on(result);
    
    assert!(result.is_ok(), "Pack operation failed: {:?}", result.err());
    let formatted_result = result.unwrap();
    
    // Parse the JSON output and verify it contains the expected info
    let json: Value = serde_json::from_str(&formatted_result).unwrap();
    assert!(json.is_object());
    assert!(json.get("path").is_some());
    
    // Verify the file exists
    let archive_path = json["path"].as_str().unwrap();
    assert!(Path::new(archive_path).exists());
} 