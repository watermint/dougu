# Message Bundle System

This module provides a unified approach to internationalization (i18n) and localization (l10n) using a simplified
message bundle system.

## Key Features

- Single, unified `MessageBundle` structure for all message formats
- Support for loading messages from JSON and Fluent (FTL) formats
- Simple `{name}` style placeholder syntax, similar to rust-i18n
- Automatic conversion from Fluent's `{ $name }` placeholders to simple `{name}` placeholders
- Support for fallback locales

## Basic Usage

```rust
use dougu_essentials::i18n::msg::{MessageBundle, MessageArgs, ResourceManager, MessageFormatter};
use dougu_essentials::i18n::msg::format::MessageFormat;
use dougu_essentials::i18n::locale::{LanguageId, LocaleId, RegionId};
use std::sync::Arc;

// Create locales
let en_us = LocaleId::new(LanguageId::new("en"), Some(RegionId::new("US")));
let ja_jp = LocaleId::new(LanguageId::new("ja"), Some(RegionId::new("JP")));

// Load message bundles from JSON strings
let en_json = r#"{
    "welcome": "Welcome, {name}!",
    "goodbye": "Goodbye, {name}!"
}"#;

let ja_json = r#"{
    "welcome": "こんにちは、{name}さん！",
    "goodbye": "さようなら、{name}さん！"
}"#;

// Parse the JSON into message bundles
let en_bundle = MessageFormat::load_from_json_string(en_json, en_us.clone()).unwrap();
let ja_bundle = MessageFormat::load_from_json_string(ja_json, ja_jp.clone()).unwrap();

// Create a resource manager with English as fallback
let mut manager = ResourceManager::new(en_us.clone());
manager.add_bundle(en_bundle).unwrap();
manager.add_bundle(ja_bundle).unwrap();

// Create a formatter for Japanese
let formatter = MessageFormatter::new(Arc::new(manager), ja_jp);

// Format a message with arguments
let args = MessageArgs::with("name", "世界");
let message = formatter.format("welcome", Some(&args)).unwrap();
// message = "こんにちは、世界さん！"
```

## Loading from Files

You can load message bundles from files:

```rust
use dougu_essentials::i18n::msg::{MessageBundle, ResourceManager};
use dougu_essentials::i18n::msg::format::MessageFormat;
use dougu_essentials::i18n::locale::{LanguageId, LocaleId, RegionId};

let locale = LocaleId::new(LanguageId::new("en"), Some(RegionId::new("US")));

// Load from JSON file
let json_bundle = MessageFormat::Json.load_from_file("messages.json", locale.clone()).unwrap();

// Load from Fluent file
let fluent_bundle = MessageFormat::Fluent.load_from_file("messages.ftl", locale.clone()).unwrap();

// Create a resource manager and add the bundles
let mut manager = ResourceManager::new(locale.clone());
manager.add_bundle(json_bundle).unwrap();
manager.add_bundle(fluent_bundle).unwrap();
```

## Message Arguments

The `MessageArgs` struct provides a convenient way to pass arguments to messages:

```rust
let mut args = MessageArgs::new();
args.add("name", "John")
    .add("count", "42");

// Or create with a single argument
let args = MessageArgs::with("name", "John");
```

## Format Conversion

The system automatically converts Fluent's `{ $name }` style placeholders to the simpler `{name}` format:

```rust
// Fluent content with { $name } style placeholders
let fluent = r#"
greeting = Hello, { $name }!
farewell = Goodbye, { $name }!
"#;

// Load and convert to {name} style internally
let bundle = MessageFormat::load_from_fluent_string(fluent, locale).unwrap();

// Use with {name} style arguments
let mut args = MessageArgs::new();
args.add("name", "World");

let greeting = bundle.format_message("greeting", Some(&args)).unwrap();
// greeting = "Hello, World!"
``` 