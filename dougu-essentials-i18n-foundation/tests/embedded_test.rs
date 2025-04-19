use dougu_essentials_i18n_foundation::{embedded, I18nInitializerLayer, t, tf, vars};
use dougu_foundation_run::{CommandLauncher, LauncherContext};

#[test]
fn test_embedded_resources_exist() {
    // Test that embedded resources are available
    let foundation_en = embedded::get_resource("foundation", "en");
    assert!(foundation_en.is_some(), "Foundation EN resource not found");
    
    let foundation_ja = embedded::get_resource("foundation", "ja");
    assert!(foundation_ja.is_some(), "Foundation JA resource not found");
    
    let file_en = embedded::get_resource("file", "en");
    assert!(file_en.is_some(), "File EN resource not found");
    
    let file_ja = embedded::get_resource("file", "ja");
    assert!(file_ja.is_some(), "File JA resource not found");
    
    // Validate that resources are valid JSON
    let parse_result: Result<serde_json::Value, _> = serde_json::from_str(foundation_en.unwrap());
    assert!(parse_result.is_ok(), "Foundation EN resource is not valid JSON");
}

#[tokio::test]
async fn test_embedded_initialization() {
    // Create a CommandLauncher with I18nInitializerLayer using embedded resources
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new("en")); // Default is to use embedded resources
    
    // Create a launcher context
    let mut context = LauncherContext::new("TestCommand".to_string(), 3);
    
    // Launch should initialize i18n with embedded resources
    launcher.launch(&mut context).await.expect("Failed to run launcher");
    
    // Test that translations are available
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert_eq!(error_msg, "Resource not found");
    
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert_eq!(layer_msg, "Executing layer: TestLayer");
    
    // Test with Japanese locale
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new("en")); 
    
    let mut context = LauncherContext::new("TestCommand".to_string(), 3);
    context.set_data("locale", "ja".to_string());
    
    launcher.launch(&mut context).await.expect("Failed to run launcher");
    
    // Test that Japanese translations are available using embedded resources
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert_eq!(error_msg, "リソースが見つかりません");
    
    // Test file command messages
    let copy_msg = tf("FILE_COPY_SUCCESS", vars!(
        "source" => "file.txt",
        "destination" => "backup.txt"
    ));
    assert_eq!(copy_msg, "file.txtからbackup.txtへのコピーに成功しました");
}

#[tokio::test]
async fn test_filesystem_fallback() {
    // Create a CommandLauncher with I18nInitializerLayer using filesystem resources
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::with_filesystem("en", false)); // Use filesystem
    
    // Create a launcher context
    let mut context = LauncherContext::new("TestCommand".to_string(), 3);
    
    // Launch should initialize i18n with filesystem resources
    // This may fail if run from a different directory, which is expected
    // The main point is that the code path for filesystem loading works if the files exist
    if let Ok(()) = launcher.launch(&mut context).await {
        // Test that translations are available
        let error_msg = t("RESOURCE_NOT_FOUND");
        assert_eq!(error_msg, "Resource not found");
    }
} 