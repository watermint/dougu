# dougu-essentials-i18n

A Rust internationalization (i18n) library with support for advanced message container formats to assist with translation workflows.

## Features

- Simple key-value translation lookup
- Advanced message container format with:
  - Source text preservation
  - Context information for translators
  - Translator comments
  - Variable interpolation
- Support for multiple locales
- Easy API for switching between locales
- Compatibility with translation memory tools

## Usage

### Basic Setup

```rust
use dougu_essentials_i18n::I18n;

// Create an i18n instance with default locale
let mut i18n = I18n::new("en");

// Load translations from JSON files
i18n.load_advanced_file("en", "translations/en.json")?;
i18n.load_advanced_file("fr", "translations/fr.json")?;

// Simple translation lookup
let welcome = i18n.t("welcome");

// Switch to another locale
i18n.set_locale("fr")?;
let welcome_fr = i18n.t("welcome");
```

### Variable Interpolation

```rust
use std::collections::HashMap;

// Set up variables
let mut vars = HashMap::new();
vars.insert("name", "Alice");
vars.insert("count", "3");

// Get translation with variables interpolated
let greeting = i18n.tf("greeting", &vars); // "Hello, Alice!"
let items = i18n.tf("items_count", &vars); // "You have 3 item(s)"
```

### Advanced Message Access

```rust
// Get the full message container with metadata
if let Ok(msg) = i18n.translate_message("greeting") {
    println!("Source text: {}", msg.source);
    println!("Translated text: {}", msg.text);
    
    if let Some(context) = &msg.context {
        println!("Context: {}", context);
    }
    
    if let Some(comments) = &msg.comments {
        println!("Translator comments: {}", comments);
    }
}
```

## Translation File Format

The library supports an advanced JSON format for translations:

```json
{
  "welcome": {
    "source": "Welcome to our application",
    "context": "Homepage header",
    "text": "Welcome to our application"
  },
  "greeting": {
    "source": "Hello, {name}!",
    "context": "Personalized greeting",
    "comments": "Variable {name} is the user's name",
    "text": "Hello, {name}!"
  }
}
```

## Benefits for Translation Workflows

The advanced message container format provides several benefits:

1. **Context for translators**: Helps translators understand where and how a message is used
2. **Source text preservation**: Keeps the original text for reference
3. **Translator comments**: Allows explaining variables or special formatting
4. **Compatible with translation memory**: Works with translation tools that can leverage previous translations

## License

MIT 