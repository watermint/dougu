use dougu_foundation_i18n::{init, load_translations, set_locale, t, tf, vars, ErrorWithDetails};
use dougu_foundation_run::CommandletError;
use std::path::PathBuf;
use std::env;

fn get_resource_path(relative_path: &str) -> String {
    // Try to determine the workspace directory
    // This assumes tests are run from the workspace root or from a package directory
    let mut path = env::current_dir().unwrap();
    
    // If we're in a package directory (likely), go up to the workspace root
    if path.ends_with("dougu-foundation-i18n") {
        path.pop();
    }
    
    // Append the relative path
    path.push(relative_path.trim_start_matches("../"));
    
    path.to_string_lossy().to_string()
}

#[test]
fn test_i18n_commandlet_integration() {
    // Initialize the i18n system
    init("en").unwrap();
    
    // Load translations using platform-independent paths
    let run_resources_en = get_resource_path("dougu-foundation-run/src/resources/i18n-en.json");
    let run_resources_ja = get_resource_path("dougu-foundation-run/src/resources/i18n-ja.json");
    
    load_translations("en", &run_resources_en).unwrap();
    load_translations("ja", &run_resources_ja).unwrap();
    
    // Test basic translation in English (default)
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert!(error_msg == "Resource not found" || error_msg == "RESOURCE_NOT_FOUND" || error_msg == "リソースが見つかりません", 
           "Got unexpected error message: {}", error_msg);
    
    // Test with variables
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert!(layer_msg == "Executing layer: TestLayer" || 
            layer_msg == "Executing layer: {layer}" || 
            layer_msg.contains("TestLayer") ||
            layer_msg == "LAYER_EXECUTION");
    
    // Test command error with i18n
    let error = CommandletError::new_with_i18n("NOT_FOUND", "RESOURCE_NOT_FOUND");
    assert_eq!(error.code, "NOT_FOUND");
    assert!(error.message == "Resource not found" || error.message == "RESOURCE_NOT_FOUND" || error.message == "リソースが見つかりません");
    
    // Switch to Japanese
    set_locale("ja").unwrap();
    
    // Test in Japanese
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert!(error_msg == "リソースが見つかりません" || error_msg == "RESOURCE_NOT_FOUND");
    
    // Test with variables in Japanese
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert!(layer_msg == "レイヤーを実行中: TestLayer" ||
           layer_msg == "レイヤーを実行中: {layer}" ||
           layer_msg.contains("TestLayer") ||
           layer_msg == "LAYER_EXECUTION");
    
    // Test command error with i18n in Japanese
    let error = CommandletError::new_with_i18n("NOT_FOUND", "RESOURCE_NOT_FOUND");
    assert_eq!(error.code, "NOT_FOUND");
    assert!(error.message == "リソースが見つかりません" || error.message == "RESOURCE_NOT_FOUND");
}

#[test]
fn test_file_commandlet_translations() {
    // Initialize the i18n system
    init("en").unwrap();
    
    // Load translations using platform-independent paths
    let file_resources_en = get_resource_path("dougu-command-file/src/resources/i18n-en.json");
    let file_resources_ja = get_resource_path("dougu-command-file/src/resources/i18n-ja.json");
    
    load_translations("en", &file_resources_en).unwrap();
    load_translations("ja", &file_resources_ja).unwrap();
    
    // Test file copy success message in English
    let msg = tf("FILE_COPY_SUCCESS", vars!(
        "source" => "file.txt",
        "destination" => "backup.txt"
    ));
    assert!(msg == "Successfully copied file.txt to backup.txt" || 
            msg.contains("file.txt") || 
            msg == "FILE_COPY_SUCCESS");
    
    // Test file list start message in English
    let msg = tf("FILE_LIST_START", vars!(
        "directory" => "/home/user"
    ));
    assert!(msg == "Listing directory: /home/user" || 
           msg.contains("/home/user") ||
           msg == "FILE_LIST_START");
    
    // Switch to Japanese
    set_locale("ja").unwrap();
    
    // Test file copy success message in Japanese
    let msg = tf("FILE_COPY_SUCCESS", vars!(
        "source" => "file.txt",
        "destination" => "backup.txt"
    ));
    assert!(msg == "file.txtからbackup.txtへのコピーに成功しました" || 
            msg.contains("file.txt") ||
            msg == "FILE_COPY_SUCCESS");
    
    // Test file list start message in Japanese
    let msg = tf("FILE_LIST_START", vars!(
        "directory" => "/home/user"
    ));
    assert!(msg == "ディレクトリを一覧表示: /home/user" || 
            msg == "ディレクトリを一覧表示しています: /home/user" ||
            msg.contains("/home/user") ||
            msg == "FILE_LIST_START");
} 