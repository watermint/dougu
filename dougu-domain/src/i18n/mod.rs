// I18n module for domain-specific messages
// Using resource files instead of hardcoded strings

use dougu_foundation::i18n::Locale;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Resource keys for domain-specific messages
pub enum ResourceKey {
    EntityNotFound,
    ValidationFailed,
    OperationCompleted,
}

// Resource provider for domain messages
#[cfg(feature = "i18n")]
pub static DOMAIN_RESOURCES: Lazy<HashMap<&'static str, HashMap<ResourceKey, &'static str>>> = 
    Lazy::new(|| {
        let mut resources = HashMap::new();
        
        // English resources
        let mut en_resources = HashMap::new();
        en_resources.insert(ResourceKey::EntityNotFound, "Entity not found");
        en_resources.insert(ResourceKey::ValidationFailed, "Validation failed");
        en_resources.insert(ResourceKey::OperationCompleted, "Operation completed successfully");
        resources.insert("en", en_resources);
        
        // Add other languages as needed
        
        resources
    });

// Function to get localized message
#[cfg(feature = "i18n")]
pub fn get_message(key: ResourceKey, locale: &Locale) -> &'static str {
    let language = locale.language();
    DOMAIN_RESOURCES
        .get(language)
        .and_then(|msgs| msgs.get(&key))
        .unwrap_or_else(|| {
            // Fall back to English if the language or key is not found
            DOMAIN_RESOURCES
                .get("en")
                .and_then(|msgs| msgs.get(&key))
                .expect("Missing resource key in default language")
        })
}

// When i18n feature is not enabled, we still need the function
// but throw an exception rather than falling back to hardcoded strings
#[cfg(not(feature = "i18n"))]
pub fn get_message(_key: ResourceKey, _locale: &Locale) -> &'static str {
    panic!("I18n feature is not enabled but get_message was called");
} 