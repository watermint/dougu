use anyhow::{Context, Result};
use dougu_essentials_obj::{Decoder, Encoder, Format, Query};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "convert" => {
            if args.len() < 5 {
                println!("Not enough arguments for convert command.");
                print_usage();
                return Ok(());
            }
            convert(&args[2], &args[3], &args[4])?;
        }
        "query" => {
            if args.len() < 4 {
                println!("Not enough arguments for query command.");
                print_usage();
                return Ok(());
            }
            query(&args[2], &args[3])?;
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  cli convert <input_file> <input_format> <output_format>");
    println!("  cli query <input_file> <query_string>");
    println!();
    println!("Formats: json, xml, cbor, bson");
    println!("Examples:");
    println!("  cli convert data.json json xml");
    println!("  cli query data.json '.users[] | select(.active == true).name'");
}

fn get_input(file_path: &str) -> Result<Vec<u8>> {
    if file_path == "-" {
        let mut buffer = Vec::new();
        io::stdin()
            .read_to_end(&mut buffer)
            .with_context(|| "Failed to read from stdin")?;
        Ok(buffer)
    } else {
        fs::read(file_path).with_context(|| format!("Failed to read file: {}", file_path))
    }
}

fn convert(input_path: &str, input_format_str: &str, output_format_str: &str) -> Result<()> {
    let input_format = Format::from_str(input_format_str)?;
    let output_format = Format::from_str(output_format_str)?;
    
    let input = get_input(input_path)?;
    
    // Read as JSON Value for maximum flexibility
    let value: Value = Decoder::decode(&input, input_format)?;
    
    // Convert to the output format
    let output = Encoder::encode(&value, output_format)?;
    
    if output_format == Format::Json || output_format == Format::Xml {
        // Print as text for text-based formats
        println!("{}", String::from_utf8_lossy(&output));
    } else {
        // Print as hex for binary formats
        println!("{}", hex::encode(&output));
    }
    
    Ok(())
}

fn query(input_path: &str, query_str: &str) -> Result<()> {
    let input = get_input(input_path)?;
    
    // Assume JSON input for query operations
    let value: Value = Decoder::decode(&input, Format::Json)?;
    
    // Create and execute the query
    let query = Query::compile(query_str)?;
    let result = query.execute(&value)?;
    
    println!("{}", result);
    
    Ok(())
} 