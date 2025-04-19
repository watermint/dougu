use anyhow::Result;
use dougu_command_build::PackOutput;
use dougu_foundation_ui::{UIManager, OutputFormat};
use std::fs;
use tempfile::tempdir;

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
    let ui = UIManager::with_format(OutputFormat::Json);
    
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
    let ui = UIManager::with_format(OutputFormat::Json);
    
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