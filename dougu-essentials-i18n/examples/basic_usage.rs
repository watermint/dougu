use anyhow::Result;
use dougu_essentials_i18n::I18n;
use std::collections::HashMap;
use std::path::Path;

fn main() -> Result<()> {
    let mut i18n = I18n::new("en");
    
    // Load advanced translations with message containers
    i18n.load_advanced_file("en", "examples/translations/en.json")?;
    i18n.load_advanced_file("fr", "examples/translations/fr.json")?;
    
    // Set up variables for interpolation
    let mut vars = HashMap::new();
    vars.insert("name", "Alice");
    vars.insert("count", "3");
    
    // Get translations in English
    println!("=== English ===");
    println!("Welcome message: {}", i18n.t("welcome"));
    
    // Get a translation with variable interpolation
    println!("Greeting: {}", i18n.tf("greeting", &vars));
    println!("Items count: {}", i18n.tf("items_count", &vars));
    
    // Get a translation with metadata
    if let Ok(msg) = i18n.translate_message("greeting") {
        println!("Greeting source: {}", msg.source);
        println!("Greeting context: {}", msg.context.as_deref().unwrap_or("none"));
        if let Some(comments) = &msg.comments {
            println!("Translator comments: {}", comments);
        }
    }
    
    // Switch to French
    i18n.set_locale("fr")?;
    
    println!("\n=== French ===");
    println!("Welcome message: {}", i18n.t("welcome"));
    
    // Get a translation with variable interpolation in French
    println!("Greeting: {}", i18n.tf("greeting", &vars));
    println!("Items count: {}", i18n.tf("items_count", &vars));
    
    println!("Submit button: {}", i18n.t("submit_button"));
    println!("Cancel button: {}", i18n.t("cancel_button"));
    
    Ok(())
} 