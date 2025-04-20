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
        input_dir.join("test_app.exe")
    } else {
        input_dir.join("test_app")
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
        name: Some("test_app".to_string()),
        version: Some("1.0.0".to_string()),
        platform: Some("test_platform".to_string()),
        input_dir: Some(input_dir.to_string_lossy().into_owned()),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
        cargo_output: None,
    }, &ui).await?;
    
    // Parse the JSON output
    let pack_output: PackOutput = serde_json::from_str(&output)?;
    
    // Verify the pack output contains the expected values
    assert_eq!(pack_output.name, "test_app-1.0.0-test_platform");
    
    // Verify artifact files exist
    let artifact_path_file = output_dir.join("artifact_path");
    let artifact_name_file = output_dir.join("artifact_name");
    
    assert!(artifact_path_file.exists());
    assert!(artifact_name_file.exists());
    
    // Verify artifact file contents
    let artifact_name_content = fs::read_to_string(artifact_name_file)?;
    assert_eq!(artifact_name_content, "test_app-1.0.0-test_platform");
    
    let artifact_path_content = fs::read_to_string(artifact_path_file)?;
    assert!(artifact_path_content.ends_with(".zip"));
    assert!(Path::new(&format!("{}/{}", output_dir.to_string_lossy(), artifact_path_content)).exists());
    
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
        cargo_output: None,
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
        cargo_output: None,
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
    
    // Verify artifact files were created
    let artifact_path_file = output_dir.join("artifact_path");
    let artifact_name_file = output_dir.join("artifact_name");
    
    assert!(artifact_path_file.exists());
    assert!(artifact_name_file.exists());
    
    // Verify artifact file contents
    let artifact_path_content = fs::read_to_string(artifact_path_file).unwrap();
    let artifact_name_content = fs::read_to_string(artifact_name_file).unwrap();
    
    assert_eq!(artifact_name_content, "test_pack-1.0.0-test-platform");
    assert!(artifact_path_content.ends_with(".zip"));
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
        cargo_output: None,
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
    
    // Verify artifact files were created
    let artifact_path_file = output_dir.join("artifact_path");
    let artifact_name_file = output_dir.join("artifact_name");
    
    assert!(artifact_path_file.exists());
    assert!(artifact_name_file.exists());
    
    // Verify artifact file contents
    let artifact_path_content = fs::read_to_string(artifact_path_file).unwrap();
    let artifact_name_content = fs::read_to_string(artifact_name_file).unwrap();
    
    assert_eq!(artifact_name_content, "test_pack_different-1.0.0-test-platform");
    assert!(artifact_path_content.ends_with(".zip"));
}

#[tokio::test]
async fn test_build_pack_with_cargo_output() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let output_dir = temp_dir.path().join("output");
    let cargo_output_dir = temp_dir.path().join("cargo_output");
    
    // Create directories
    fs::create_dir_all(&output_dir)?;
    fs::create_dir_all(&cargo_output_dir)?;
    
    // Create mock executable
    let mock_executable_dir = temp_dir.path().join("bin");
    fs::create_dir_all(&mock_executable_dir)?;
    let mock_executable = if cfg!(windows) {
        mock_executable_dir.join("test_exe.exe")
    } else {
        mock_executable_dir.join("test_exe")
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
    
    // Get the absolute path of the mock executable
    let mock_executable_absolute = fs::canonicalize(&mock_executable)?;
    
    // Create mock cargo output JSON
    let cargo_output_path = cargo_output_dir.join("cargo_output.json");
    let cargo_output_content = format!(
        r#"{{"reason":"compiler-artifact", "target":{{"kind":["bin"], "name":"test_executable", "src_path":"main.rs"}}, "profile":{{"opt-level":"0", "debuginfo":2, "debug-assertions":true, "overflow-checks":true, "test":false}}, "executable":"{}"}}"#,
        mock_executable_absolute.to_string_lossy().replace("\\", "\\\\") // Escape backslashes for Windows paths in JSON
    );
    fs::write(&cargo_output_path, cargo_output_content.clone())?;
    
    // Print paths for debugging
    println!("Mock executable path: {}", mock_executable_absolute.display());
    println!("Cargo output content: {}", cargo_output_content);
    
    // Create UI manager for testing (JSON format)
    let ui = UIManager::with_format(OutputFormat::JsonLines);
    
    // Run the build pack command with cargo output
    let output = dougu_command_build::execute_pack(&dougu_command_build::PackArgs {
        name: None,  // Let it detect from cargo output
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: None,
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
        cargo_output: Some(cargo_output_path.to_string_lossy().into_owned()),
    }, &ui).await?;
    
    // Parse the JSON output
    let pack_output: PackOutput = serde_json::from_str(&output)?;
    
    // Verify the output structure - name should come from cargo output
    assert_eq!(pack_output.name, "test_executable-1.0.0-test-platform");
    assert!(pack_output.path.ends_with(".zip")); // Should end with .zip
    assert_eq!(pack_output.version, "1.0.0");
    assert_eq!(pack_output.platform, "test-platform");
    
    // Verify the archive was created
    assert!(fs::metadata(&pack_output.path).is_ok());
    
    // Verify artifact files were created
    let artifact_path_file = output_dir.join("artifact_path");
    let artifact_name_file = output_dir.join("artifact_name");
    
    assert!(artifact_path_file.exists());
    assert!(artifact_name_file.exists());
    
    // Verify artifact file contents
    let artifact_path_content = fs::read_to_string(artifact_path_file)?;
    let artifact_name_content = fs::read_to_string(artifact_name_file)?;
    
    assert_eq!(artifact_name_content, "test_executable-1.0.0-test-platform");
    assert!(artifact_path_content.ends_with(".zip"));
    
    Ok(())
}

#[tokio::test]
async fn test_build_pack_artifact_naming_format() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    
    // Create input directory and add test files
    fs::create_dir_all(&input_dir)?;
    
    // Create a mock executable
    let mock_executable = if cfg!(windows) {
        input_dir.join("test_app.exe")
    } else {
        input_dir.join("test_app")
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
    
    // Test various combinations of name, version, and platform
    let test_cases = vec![
        // name, version, platform, expected_name
        ("app1".to_string(), "2.0.0".to_string(), "linux".to_string(), "app1-2.0.0-linux"),
        ("app2".to_string(), "2.1.5".to_string(), "windows".to_string(), "app2-2.1.5-windows"),
        ("app3".to_string(), "0.1.0".to_string(), "macos".to_string(), "app3-0.1.0-macos"),
    ];
    
    for (name, version, platform, expected_name) in test_cases {
        // Clear output directory between tests
        for entry in fs::read_dir(&output_dir)? {
            let entry = entry?;
            fs::remove_file(entry.path())?;
        }
        
        // Run the build pack command
        let output = dougu_command_build::execute_pack(&dougu_command_build::PackArgs {
            name: Some(name),
            version: Some(version),
            platform: Some(platform),
            input_dir: Some(input_dir.to_string_lossy().into_owned()),
            output_dir: Some(output_dir.to_string_lossy().into_owned()),
            cargo_output: None,
        }, &ui).await?;
        
        // Parse the JSON output
        let pack_output: PackOutput = serde_json::from_str(&output)?;
        
        // Verify the name follows the EXECUTABLE-VERSION-PLATFORM format
        assert_eq!(pack_output.name, expected_name);
        
        // Verify artifact file contents
        let artifact_path_file = output_dir.join("artifact_path");
        let artifact_name_file = output_dir.join("artifact_name");
        
        let artifact_name_content = fs::read_to_string(artifact_name_file)?;
        assert_eq!(artifact_name_content, expected_name);
        
        // Make sure archive was created
        let artifact_path_content = fs::read_to_string(artifact_path_file)?;
        assert!(artifact_path_content.ends_with(".zip"));
        assert!(Path::new(&format!("{}/{}", output_dir.to_string_lossy(), artifact_path_content)).exists());
    }
    
    Ok(())
}

#[tokio::test]
#[ignore = "Requires additional setup for cargo output parsing"]
async fn test_build_pack_from_cargo_output() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let debug_dir = temp_dir.path().join("debug");
    let output_dir = temp_dir.path().join("output");
    
    // Create directory structure and add test files
    fs::create_dir_all(&debug_dir)?;
    fs::create_dir_all(&output_dir)?;
    
    // Create a mock executable in the target/debug directory
    let mock_executable_name = if cfg!(windows) { "test_app.exe" } else { "test_app" };
    let mock_executable = debug_dir.join(mock_executable_name);
    fs::write(&mock_executable, "mock executable content")?;
    
    // Make the file executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_executable)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_executable, perms)?;
    }
    
    // Create a mock cargo output JSON file
    let cargo_output_path = temp_dir.path().join("cargo_output.json");
    let cargo_json = format!(
        r#"{{
            "reason": "compiler-artifact",
            "package_id": "dougu-command-build 0.1.0",
            "executable": "{}",
            "target": {{
                "kind": ["bin"],
                "name": "test_app",
                "src_path": "src/main.rs"
            }},
            "profile": {{"opt-level": "0", "debuginfo": 2, "debug-assertions": true, "overflow-checks": true, "test": false}}
        }}"#,
        mock_executable.to_string_lossy().replace('\\', "\\\\")
    );
    fs::write(&cargo_output_path, cargo_json)?;
    
    // Run the build pack command with cargo output
    let ui = UIManager::with_format(OutputFormat::Default);
    let output = dougu_command_build::execute_pack(&dougu_command_build::PackArgs {
        name: None,
        version: Some("1.0.0".to_string()),
        platform: Some("test-platform".to_string()),
        input_dir: None,
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
        cargo_output: Some(cargo_output_path.to_string_lossy().into_owned()),
    }, &ui).await?;
    
    // Verify artifact files exist
    let artifact_path_file = output_dir.join("artifact_path");
    let artifact_name_file = output_dir.join("artifact_name");
    
    assert!(artifact_path_file.exists());
    assert!(artifact_name_file.exists());
    
    // Verify artifact file contents
    let artifact_name_content = fs::read_to_string(artifact_name_file)?;
    
    // Parse the JSON from the output to verify the name
    let pack_output: dougu_command_build::PackOutput = serde_json::from_str(&output)?;
    
    // Check if name starts with "test_app-1.0.0-" and ignore platform differences (- vs _)
    assert!(pack_output.name.starts_with("test_app-1.0.0-"), 
            "Expected name to start with test_app-1.0.0-, got {}", pack_output.name);
    
    // Make sure artifact name matches output
    assert_eq!(artifact_name_content, pack_output.name);
    
    let artifact_path_content = fs::read_to_string(artifact_path_file)?;
    assert!(artifact_path_content.ends_with(".zip"));
    
    Ok(())
} 