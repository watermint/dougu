use anyhow::{Context, Result, Error as AnyhowError};
use base64::Engine;
use clap::Parser;
use dougu_essentials_obj::{Decoder, Encoder, Format, Query};
use dougu_foundation_ui::{UIManager, OutputFormat};
use dougu_foundation_ui::resources::ui_messages::FORMAT_OPTION_DESCRIPTION;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::str::FromStr;

mod resources;
mod launcher;

pub use launcher::ObjCommandLayer;

use resources::messages::*;

// Constants for command descriptions
const CMD_QUERY_DESCRIPTION: &str = "Execute a query on an object notation file";
const CMD_CONVERT_DESCRIPTION: &str = "Convert between object notation formats";
const CMD_EXTRACT_DESCRIPTION: &str = "Extract raw value from input (for scripting)";

// Constants for argument descriptions
const ARG_FORMAT_DESCRIPTION: &str = "Input format (json, bson, xml, cbor)";
const ARG_OUTPUT_FORMAT_DESCRIPTION: &str = "Output format (json, bson, xml, cbor)";
const ARG_FILE_DESCRIPTION: &str = "Input file path (use - for stdin)";
const ARG_QUERY_DESCRIPTION: &str = "Query string in jq-like format";
const ARG_RAW_DESCRIPTION: &str = "Output raw value without quotes or escapes (for scripting)";

#[derive(Parser, Serialize, Deserialize)]
#[command(name = "obj")]
#[command(about = CMD_OBJ_DESCRIPTION)]
pub struct ObjCommand {
    /// Output format (default, jsonl, markdown)
    #[arg(long = "ui-format", help = FORMAT_OPTION_DESCRIPTION, value_parser = ["default", "jsonl", "markdown"], default_value = "default")]
    pub format: String,
    #[command(subcommand)]
    command: ObjCommands,
}

#[derive(clap::Subcommand, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjCommands {
    /// Execute a query on an object notation file
    #[clap(name = "query")]
    #[clap(about = CMD_QUERY_DESCRIPTION)]
    Query {
        /// Input format (json, bson, xml, cbor)
        #[clap(short = 'f', long = "format", default_value = "json", help = ARG_FORMAT_DESCRIPTION)]
        format: String,

        /// Input file path (use - for stdin)
        #[clap(help = ARG_FILE_DESCRIPTION)]
        file: PathBuf,

        /// Query string in jq-like format
        #[clap(help = ARG_QUERY_DESCRIPTION)]
        query: String,
    },

    /// Convert between object notation formats
    #[clap(name = "convert")]
    #[clap(about = CMD_CONVERT_DESCRIPTION)]
    Convert {
        /// Input format (json, bson, xml, cbor)
        #[clap(short = 'f', long = "format", default_value = "json", help = ARG_FORMAT_DESCRIPTION)]
        format: String,

        /// Output format (json, bson, xml, cbor)
        #[clap(short = 'o', long = "output-format", default_value = "json", help = ARG_OUTPUT_FORMAT_DESCRIPTION)]
        output_format: String,

        /// Input file path (use - for stdin)
        #[clap(help = ARG_FILE_DESCRIPTION)]
        file: PathBuf,
    },

    /// Extract raw value from input (for scripting)
    #[clap(name = "extract")]
    #[clap(about = CMD_EXTRACT_DESCRIPTION)]
    Extract {
        /// Input file path (use - for stdin)
        #[clap(help = ARG_FILE_DESCRIPTION)]
        file: PathBuf,

        /// Query string in jq-like format
        #[clap(help = ARG_QUERY_DESCRIPTION)]
        query: String,

        /// Input format (json, bson, xml, cbor)
        #[clap(short = 'f', long = "format", default_value = "json", help = ARG_FORMAT_DESCRIPTION)]
        format: String,
        
        /// Output raw value without quotes or escapes
        #[clap(long = "raw", help = ARG_RAW_DESCRIPTION)]
        raw: bool,
    },
}

impl ObjCommand {
    pub async fn execute(self) -> Result<()> {
        let output_format = OutputFormat::from_str(&self.format).unwrap_or(OutputFormat::Default);
        let ui = UIManager::with_format(output_format);
        let use_json_output = output_format == OutputFormat::JsonLines;
        
        match self.command {
            ObjCommands::Query {
                ref format,
                ref file,
                ref query,
            } => self.execute_query(format, file, query, &ui, use_json_output).await,
            ObjCommands::Convert {
                ref format,
                ref output_format,
                ref file,
            } => self.execute_convert(format, file, output_format, &ui, use_json_output).await,
            ObjCommands::Extract {
                ref file,
                ref query,
                ref format,
                raw,
            } => self.execute_extract(format, file, query, &ui, use_json_output, raw).await,
        }
    }

    async fn execute_query(&self, format_str: &str, file_path: &PathBuf, query_str: &str, ui: &UIManager, json_mode: bool) -> Result<()> {
        // Parse format
        let format = Format::from_str(format_str)
            .with_context(|| format!("{}: {}", ERROR_INVALID_FORMAT, format_str))?;
        
        // Read input from file or stdin
        let input = if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .with_context(|| ERROR_STDIN_READ)?;
            buffer
        } else {
            fs::read(file_path)
                .with_context(|| format!("{}: {}", ERROR_FILE_NOT_FOUND, file_path.display()))?
        };
        
        // Handle special case for jsonl format
        let input_values = if format == Format::Jsonl {
            // For JSON Lines, split by lines and parse each line
            let content = String::from_utf8_lossy(&input);
            let mut values = Vec::new();
            
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                let value: Value = serde_json::from_str(line)
                    .with_context(|| format!("{}: {}", ERROR_DECODE_FAILED, line))?;
                values.push(value);
            }
            values
        } else {
            // For other formats, decode as a single value
            let value: Value = Decoder::decode(&input, format)
                .with_context(|| ERROR_DECODE_FAILED)?;
            vec![value]
        };
        
        // Compile query
        let query = Query::compile(query_str)
            .with_context(|| format!("{}: {}", ERROR_QUERY_PARSE, query_str))?;
        
        // Process each value
        for value in input_values {
            // Execute query
            let result = query.execute(&value)
                .with_context(|| ERROR_QUERY_FAILED)?;
            
            // Convert result from jaq Val to JSON Value using string parsing
            // This is a workaround since Val doesn't implement Serialize
            let result_str = result.to_string();
            let json_result: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
            
            // Format and display result
            if json_mode {
                // For JSON output format, maintain proper structure
                ui.json(&json_result).map_err(|e| AnyhowError::msg(e.to_string()))?;
            } else {
                // For human-readable output
                ui.text(&format!("{}", json_result));
            }
        }
        
        Ok(())
    }
    
    /// Extract raw value from query result for scripting
    pub fn extract_raw_value(json_result: &str) -> String {
        // For string values, remove the enclosing quotes
        if let Some(value) = json_result.trim_start_matches('{')
            .trim_end_matches('}')
            .trim()
            .strip_prefix("\"type\":\"text\",\"text\":")
        {
            // Remove the outermost quotes if they exist
            if value.starts_with('"') && value.ends_with('"') {
                let inner = &value[1..value.len()-1];
                // Unescape any escaped quotes
                return inner.replace("\\\"", "\"");
            }
            return value.to_string();
        }
        
        // For other values, return as is
        json_result.to_string()
    }
    
    /// Processes a JSON value to output a clean raw string without quotes or escape characters
    /// This replaces the need for the `tr -d '"\\''` command
    fn process_raw_value(value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            _ => value.to_string()
                .trim_start_matches('"')
                .trim_end_matches('"')
                .replace("\\\"", "\"")
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\t", "\t")
        }
    }

    async fn execute_convert(&self, input_format_str: &str, input_file: &PathBuf, output_format_str: &str, ui: &UIManager, json_mode: bool) -> Result<()> {
        // Get the input data outside the closure so we can use it for UI display
        let input_data = self.get_input(input_file)
            .with_context(|| ERROR_FILE_NOT_FOUND)?;
        let input_size = input_data.len();
        
        let result: Result<serde_json::Value, anyhow::Error> = (|| {
            let input_format = Format::from_str(input_format_str)
                .with_context(|| ERROR_INVALID_FORMAT)?;
            let output_format = Format::from_str(output_format_str)
                .with_context(|| ERROR_INVALID_FORMAT)?;
            
            let value: Value = Decoder::decode(&input_data, input_format)
                .with_context(|| ERROR_DECODE_FAILED)?;
            let output = match output_format {
                Format::Json | Format::Xml => {
                    let output = Encoder::encode_to_string(&value, output_format)
                        .with_context(|| ERROR_DECODE_FAILED)?;
                    serde_json::json!({"result": output})
                },
                _ => {
                    let output = Encoder::encode(&value, output_format)
                        .with_context(|| ERROR_DECODE_FAILED)?;
                    serde_json::json!({"info": "Binary data written to stdout", "bytes": base64::engine::general_purpose::STANDARD.encode(&output)})
                }
            };
            Ok(output)
        })();
        if json_mode {
            match result {
                Ok(val) => {
                    let json_string = serde_json::to_string_pretty(&val).unwrap();
                    ui.text(&json_string);
                },
                Err(e) => {
                    let error_json = serde_json::json!({"error": e.to_string()}).to_string();
                    ui.text(&error_json);
                },
            }
        } else {
            match result {
                Ok(val) => {
                    ui.title("Format Conversion");
                    ui.key_value_list(&[
                        ("Input Format", &input_format_str),
                        ("Output Format", &output_format_str),
                        ("Input Size", &format!("{} bytes", input_size)),
                    ]);
                    
                    ui.subtitle("Conversion Result");
                    
                    let (formatted, error_json) = self.format_output(&ui, val.clone())?;
                    if !formatted.is_empty() {
                        ui.text(&error_json);
                    }
                    
                    ui.code(&formatted, Some("json"));
                    
                    if let Some(info) = val.get("info") {
                        ui.info(info.as_str().unwrap_or("Binary data written to stdout"));
                    }
                },
                Err(e) => {
                    ui.error(&e.to_string());
                }
            }
        }
        Ok(())
    }

    fn get_input(&self, file_path: &PathBuf) -> Result<Vec<u8>> {
        if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .with_context(|| "Failed to read from stdin")?;
            Ok(buffer)
        } else {
            fs::read(file_path).with_context(|| format!("Failed to read file: {:?}", file_path))
        }
    }

    fn format_output(&self, _ui: &UIManager, result: serde_json::Value) -> Result<(String, String)> {
        let formatted = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Error formatting JSON: {}", e));

        let error_json = if let Some(error) = result.get("error") {
            error.to_string()
        } else {
            String::new()
        };
        
        Ok((formatted, error_json))
    }

    async fn execute_extract(&self, format_str: &str, file_path: &PathBuf, query_str: &str, ui: &UIManager, json_mode: bool, raw: bool) -> Result<()> {
        // Parse format
        let format = Format::from_str(format_str)
            .with_context(|| format!("{}: {}", ERROR_INVALID_FORMAT, format_str))?;
        
        // Read input from file or stdin
        let input = if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .with_context(|| ERROR_STDIN_READ)?;
            buffer
        } else {
            fs::read(file_path)
                .with_context(|| format!("{}: {}", ERROR_FILE_NOT_FOUND, file_path.display()))?
        };
        
        // Handle special case for jsonl format
        let input_values = if format == Format::Jsonl {
            // For JSON Lines, split by lines and parse each line
            let content = String::from_utf8_lossy(&input);
            let mut values = Vec::new();
            
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                let value: Value = serde_json::from_str(line)
                    .with_context(|| format!("{}: {}", ERROR_DECODE_FAILED, line))?;
                values.push(value);
            }
            values
        } else {
            // For other formats, decode as a single value
            let value: Value = Decoder::decode(&input, format)
                .with_context(|| ERROR_DECODE_FAILED)?;
            vec![value]
        };
        
        // Compile query
        let query = Query::compile(query_str)
            .with_context(|| format!("{}: {}", ERROR_QUERY_PARSE, query_str))?;
        
        // Process each value
        for value in input_values {
            // Execute query
            let result = query.execute(&value)
                .with_context(|| ERROR_QUERY_FAILED)?;
            
            // Convert result using string parsing to avoid Serialize issues
            let result_str = result.to_string();
            let json_value: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
            
            if raw {
                // Output raw value directly without JSON wrapping or quotes
                let raw_value = Self::process_raw_value(&json_value);
                println!("{}", raw_value);
            } else {
                // Extract raw value with normal formatting
                let raw_value = ObjCommand::extract_raw_value(&json_value.to_string());
                
                // Format and display result
                if json_mode {
                    // For JSON output format, maintain proper structure
                    ui.json(&serde_json::json!({"raw_value": raw_value})).map_err(|e| AnyhowError::msg(e.to_string()))?;
                } else {
                    // For human-readable output
                    ui.text(&format!("{}", raw_value));
                }
            }
        }
        
        Ok(())
    }
}

pub type ExecuteResult = Result<(), String>;

pub fn execute_command(args: &ObjCommand) -> ExecuteResult {
    // Using a simplified implementation here since the async implementation
    // contains the full logic
    match &args.command {
        ObjCommands::Query { format, file, query } => {
            // Simplified synchronous implementation
            let _format_value = Format::from_str(format).map_err(|e| e.to_string())?;
            let input = if file.to_string_lossy() == "-" {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer).map_err(|_| ERROR_STDIN_READ.to_string())?;
                buffer
            } else {
                fs::read_to_string(file).map_err(|_| ERROR_FILE_NOT_FOUND.to_string())?
            };

            // Explicitly specify the type to avoid inference issues
            let value: serde_json::Value = serde_json::from_str(&input).map_err(|_| ERROR_DECODE_FAILED.to_string())?;
            let query = Query::compile(query).map_err(|_| ERROR_QUERY_PARSE.to_string())?;
            let result = query.execute(&value).map_err(|_| ERROR_QUERY_FAILED.to_string())?;
            
            // Use string conversion to handle Val
            let result_str = result.to_string();
            let json_result: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
                
            println!("{}", json_result);
            
            Ok(())
        },
        ObjCommands::Convert { format, output_format, .. } => {
            // Simplified implementation
            println!("Converting from {} to {}", format, output_format);
            Ok(())
        },
        ObjCommands::Extract { query, format, file, raw } => {
            // Read input
            let mut input = String::new();
            if file.to_string_lossy() == "-" {
                io::stdin()
                    .read_to_string(&mut input)
                    .map_err(|_| ERROR_STDIN_READ.to_string())?;
            } else {
                input = fs::read_to_string(file)
                    .map_err(|_| ERROR_FILE_NOT_FOUND.to_string())?;
            }
            
            // Parse input based on format
            let value: Value = match format.to_lowercase().as_str() {
                "json" => {
                    serde_json::from_str(&input)
                        .map_err(|_| ERROR_DECODE_FAILED.to_string())?
                },
                "jsonl" => {
                    // For JSONL, use just the first line for simplicity in sync mode
                    let line = input.lines().next().ok_or(ERROR_DECODE_FAILED.to_string())?;
                    serde_json::from_str(line)
                        .map_err(|_| ERROR_DECODE_FAILED.to_string())?
                },
                _ => return Err(ERROR_INVALID_FORMAT.to_string()),
            };
            
            // Compile and execute query
            let query_obj = Query::compile(query)
                .map_err(|_| ERROR_QUERY_PARSE.to_string())?;
            let result = query_obj.execute(&value)
                .map_err(|_| ERROR_QUERY_FAILED.to_string())?;
            
            // Convert result to JSON
            let result_str = result.to_string();
            let json_value: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
            
            if *raw {
                // Output raw value directly
                let raw_value = ObjCommand::process_raw_value(&json_value);
                println!("{}", raw_value);
            } else {
                // Extract and output with normal formatting
                let raw_value = extract_raw_value(&json_value).map_err(|e| e)?;
                println!("{}", raw_value);
            }
            
            Ok(())
        }
    }
}

/// Extracts raw value from query result
pub fn extract_raw_value(query_result: &Value) -> Result<String, String> {
    match query_result {
        Value::String(s) => {
            // For string values, return the raw string without quotes
            Ok(s.clone())
        },
        Value::Number(n) => {
            // For numbers, convert to string
            Ok(n.to_string())
        },
        Value::Bool(b) => {
            // For booleans, convert to string
            Ok(b.to_string())
        },
        Value::Null => {
            // For null, return empty string
            Ok(String::new())
        },
        _ => {
            // For arrays and objects, return the JSON string
            serde_json::to_string(query_result).map_err(|e| e.to_string())
        }
    }
}

/// Executes extract command to return raw value (for scripting)
pub fn execute_extract(format: &str, file: &PathBuf, query: &str) -> ExecuteResult {
    let mut input = String::new();
    
    // Read input from file or stdin
    if file.to_string_lossy() == "-" {
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|_| ERROR_STDIN_READ.to_string())?;
    } else {
        input = fs::read_to_string(file)
            .map_err(|_| ERROR_FILE_NOT_FOUND.to_string())?;
    }
    
    // Parse the input based on format, being explicit about the type
    let value: Value = match format.to_lowercase().as_str() {
        "json" => {
            serde_json::from_str(&input)
                .map_err(|_| ERROR_DECODE_FAILED.to_string())?
        },
        // Support for other formats can be added here
        _ => return Err(ERROR_INVALID_FORMAT.to_string()),
    };
    
    // Compile and execute the query
    let query = Query::compile(query)
        .map_err(|_| ERROR_QUERY_PARSE.to_string())?;
    
    let result = query.execute(&value)
        .map_err(|_| ERROR_QUERY_FAILED.to_string())?;
    
    // Convert result using string parsing to avoid Serialize issues
    let result_str = result.to_string();
    let json_value: Value = serde_json::from_str(&result_str)
        .unwrap_or_else(|_| Value::String(result_str));
    
    // Extract the raw value
    let raw_value = extract_raw_value(&json_value).map_err(|e| e)?;
    
    // Output the raw value
    println!("{}", raw_value);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parsing() {
        assert!(Format::from_str("json").is_ok());
        assert!(Format::from_str("xml").is_ok());
        assert!(Format::from_str("bson").is_ok());
        assert!(Format::from_str("cbor").is_ok());
        assert!(Format::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_process_raw_value() {
        assert_eq!(ObjCommand::process_raw_value(&Value::String("test".to_string())), "test");
        assert_eq!(ObjCommand::process_raw_value(&Value::Number(42.into())), "42");
        assert_eq!(ObjCommand::process_raw_value(&Value::Bool(true)), "true");
        assert_eq!(ObjCommand::process_raw_value(&Value::Null), "");
        
        // Test string with quotes and escapes
        let escaped_json = Value::String(r#"hello \"world\""#.to_string());
        assert_eq!(ObjCommand::process_raw_value(&escaped_json), r#"hello "world""#);
    }
} 