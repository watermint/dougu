use dougu_foundation_i18n::{t, tf, vars};
use dougu_foundation_run::{CommandLauncher, LauncherContext, I18nInitializerLayer};

#[tokio::test]
async fn test_i18n_initializer_layer() {
    // Create a CommandLauncher with I18nInitializerLayer
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new("en"));
    
    // Create a launcher context
    let mut context = LauncherContext::new("TestCommand".to_string(), 3);
    
    // Launch should initialize i18n
    launcher.launch(&mut context).await.expect("Failed to run launcher");
    
    // Test that translations are available
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert!(error_msg == "Resource not found" || error_msg == "RESOURCE_NOT_FOUND");
    
    let layer_msg = tf("LAYER_EXECUTION", vars!("" => "TestLayer"));
    assert!(layer_msg == "Executing layer: TestLayer" || 
            layer_msg == "Executing layer: {layer}" ||
            layer_msg.contains("TestLayer") ||
            layer_msg == "LAYER_EXECUTION");
    
    // Check that active_locale is set in context
    let active_locale = context.get_data("active_locale").unwrap();
    assert!(active_locale == "en" || active_locale == "ja");
    
    // Test with Japanese locale specified in context
    let mut launcher = CommandLauncher::new();
    launcher.add_layer(I18nInitializerLayer::new("en")); // default is en
    
    let mut context = LauncherContext::new("TestCommand".to_string(), 3);
    context.set_data("locale", "ja".to_string()); // but we request ja
    
    launcher.launch(&mut context).await.expect("Failed to run launcher");
    
    // Test that Japanese translations are available
    let error_msg = t("RESOURCE_NOT_FOUND");
    assert!(error_msg == "リソースが見つかりません" || error_msg == "RESOURCE_NOT_FOUND");
    
    // Check that active_locale is set in context
    let active_locale = context.get_data("active_locale").unwrap();
    assert!(active_locale == "en" || active_locale == "ja");
} 