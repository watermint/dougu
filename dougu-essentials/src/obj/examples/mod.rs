use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::obj::{Format, notation, Notation};

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
    hobbies: Vec<String>,
}

pub fn run_example() -> Result<()> {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        hobbies: vec!["reading".to_string(), "hiking".to_string()],
    };
    
    // Using direct notation implementations
    let json_notation = notation::json::JsonNotation;
    let yaml_notation = notation::yaml::YamlNotation;
    let toml_notation = notation::toml::TomlNotation;
    
    // Convert to different formats
    let json = json_notation.encode_to_string(&person)?;
    let yaml = yaml_notation.encode_to_string(&person)?;
    let toml = toml_notation.encode_to_string(&person)?;
    
    println!("JSON:\n{}\n", json);
    println!("YAML:\n{}\n", yaml);
    println!("TOML:\n{}\n", toml);
    
    // Parse back from YAML
    let yaml_bytes = yaml.as_bytes();
    let decoded: Person = yaml_notation.decode(yaml_bytes)?;
    println!("Decoded: {:?}", decoded);
    
    // Using the format-based approach
    let json_format_notation = notation::get_notation(Format::Json);
    let json2 = json_format_notation.encode_to_string(&person)?;
    println!("JSON (via Format): {}", json2);
    
    // JSONL collection example
    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
            hobbies: vec!["reading".to_string(), "hiking".to_string()],
        },
        Person {
            name: "Bob".to_string(),
            age: 25,
            hobbies: vec!["gaming".to_string(), "cooking".to_string()],
        },
    ];
    
    let jsonl_notation = notation::jsonl::JsonlNotation;
    let jsonl = jsonl_notation.encode_collection(&people)?;
    println!("JSONL:\n{}", String::from_utf8_lossy(&jsonl));
    
    Ok(())
} 