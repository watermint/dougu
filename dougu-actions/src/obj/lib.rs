use anyhow::{anyhow, Context, Error as AnyhowError, Result};
use base64::Engine;
use clap::Parser;
use dougu_essentials::{Decoder, Encoder, Format, Query};
use dougu_foundation::{
    ui::{OutputFormat, UIManager},
    resources::ui_messages::FORMAT_OPTION_DESCRIPTION
};
use serde::{Deserialize, Serialize};
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
                ref format,
                ref file,
                ref query,
                raw,
            } => self.execute_extract(format, file, query, &ui, use_json_output, raw).await,
        }
    }

    async fn execute_query(&self, format_str: &str, file_path: &PathBuf, query_str: &str, ui: &UIManager, json_mode: bool) -> Result<()> {
        // Parse format
        let format_value = Format::from_str(format_str).map_err(|e| anyhow!("{}", e))?;
        
        // Read input from file or stdin
        let input = if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|_| anyhow!("{}", ERROR_STDIN_READ))?;
            buffer
        } else {
            fs::read(file_path)
                .map_err(|_| anyhow!("{}", ERROR_FILE_NOT_FOUND))?
        };
        
        // Handle special case for jsonl format
        let input_values = if format_value == Format::Jsonl {
            // For JSON Lines, split by lines and parse each line
            let content = String::from_utf8_lossy(&input);
            let mut values = Vec::new();
            
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                let value: Value = serde_json::from_str(line)
                    .map_err(|e| anyhow!("{}: {}: {}", ERROR_DECODE_FAILED, line, e))?;
                values.push(value);
            }
            values
        } else {
            // For other formats, decode as a single value
            let value: Value = Decoder::decode(&input, format_value)
                .map_err(|e| anyhow!("{}: {}", ERROR_DECODE_FAILED, e))?;
            vec![value]
        };
        
        // Compile query
        let query = Query::compile(query_str)
            .map_err(|e| anyhow!("{}: {}: {}", ERROR_QUERY_PARSE, query_str, e))?;
        
        // Process each value
        for value in input_values {
            // Execute query
            let result = query.execute(&value)
                .map_err(|e| anyhow!("{}: {}", ERROR_QUERY_FAILED, e))?;
            
            // Convert result from jaq Val to JSON Value using string parsing
            // This is a workaround since Val doesn't implement Serialize
            let result_str = result.to_string();
            let json_result: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
            
            // Format and display result
            if json_mode {
                // For JSON output format, maintain proper structure
                ui.json(&json_result).map_err(|e| anyhow!("{}", e))?;
            } else {
                // For human-readable output
                ui.text(&format!("{}", json_result));
            }
        }
        
        Ok(())
    }
    
    /// Extract raw value from query result for scripting
    pub fn extract_raw_value(json_result: &Value) -> String {
        Self::process_raw_value(json_result)
    }
    
    /// Processes a JSON value to output a clean raw string without quotes or escape characters
    pub fn process_raw_value(value: &Value) -> String {
        match value {
            Value::String(s) => {
                // Replace escaped quotes with actual quotes 
                s.replace("\\\"", "\"")
                    .replace("\\\\", "\\")
                    .replace("\\n", "\n")
                    .replace("\\r", "\r")
                    .replace("\\t", "\t")
            },
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
        
        // Match function based on formats
        let result = (|| {
            // Parse formats
            let input_format = Format::from_str(input_format_str)?;
            let output_format = Format::from_str(output_format_str)?;
            
            // Decode input data
            let value: Value = Decoder::decode(&input_data, input_format)?;
            
            // Encode to output format
            let output_data = Encoder::encode(&value, output_format)?;
            
            Ok::<(Vec<u8>, Value), AnyhowError>((output_data, value))
        })();
        
        // Process the result
        match result {
            Ok((output_data, json_value)) => {
                if json_mode {
                    // Use structured output for JSON mode
                    let val = serde_json::json!({
                        "input_format": input_format_str,
                        "output_format": output_format_str,
                        "input_size": input_size,
                        "output_size": output_data.len(),
                        "data": base64::engine::general_purpose::STANDARD.encode(&output_data),
                        "info": format!("Converted {} bytes from {} to {}", input_size, input_format_str, output_format_str)
                    });
                    
                    ui.json(&val).map_err(|e| anyhow!("{}", e))?;
                } else {
                    // For normal UI output, show summary and formatted JSON view of data
                    ui.info(&format!("Converted {} bytes from {} to {}", 
                         input_size, input_format_str, output_format_str));
                    
                    // For textual formats, display as string; for binary formats, just mention size
                    let formatted = match output_format_str {
                        "json" | "xml" | "yaml" | "toml" => {
                            String::from_utf8_lossy(&output_data).to_string()
                        },
                        _ => {
                            format!("Binary data of {} bytes", output_data.len())
                        }
                    };
                    
                    ui.code(&formatted, Some("json"));
                    
                    if let Some(info) = json_value.get("info") {
                        ui.info(info.as_str().unwrap_or("Binary data written to stdout"));
                    }
                }
            },
            Err(e) => {
                // Display error
                if json_mode {
                    let val = serde_json::json!({
                        "error": e.to_string(),
                        "input_format": input_format_str,
                        "output_format": output_format_str,
                        "input_size": input_size
                    });
                    
                    ui.json(&val).map_err(|e| anyhow!("{}", e))?;
                } else {
                    ui.error(&e.to_string());
                }
            }
        }
        Ok(())
    }

    fn get_input(&self, file_path: &PathBuf) -> Result<Vec<u8>> {
        let input = if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .with_context(|| ERROR_STDIN_READ)?;
            buffer
        } else {
            fs::read(file_path).map_err(|_| anyhow!("{}", ERROR_FILE_NOT_FOUND))?
        };
        Ok(input)
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
        let format_value = Format::from_str(format_str).map_err(|e| anyhow!("{}", e))?;
        
        // Read input
        let input = if file_path.to_string_lossy() == "-" {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|_| anyhow!("{}", ERROR_STDIN_READ))?;
            buffer
        } else {
            fs::read(file_path).map_err(|_| anyhow!("{}", ERROR_FILE_NOT_FOUND))?
        };
        
        // Compile query
        let mut query_obj = Query::compile(query_str).map_err(|_| anyhow!("{}", ERROR_QUERY_PARSE))?;
        
        // Process based on format
        match format_value {
            Format::Jsonl => {
                // For JSONL, process line by line
                let content = String::from_utf8_lossy(&input);
                
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    // Parse the line as JSON
                    let line_value: Value = match serde_json::from_str(line) {
                        Ok(val) => val,
                        Err(_) => continue, // Skip invalid lines
                    };
                    
                    // Execute query on this line's JSON
                    let result = match query_obj.execute(&line_value) {
                        Ok(res) => res,
                        Err(_) => continue, // Skip if query fails for this line
                    };
                    
                    // Convert to JSON value
                    let result_str = result.to_string();
                    let json_value: Value = serde_json::from_str(&result_str)
                        .unwrap_or_else(|_| Value::String(result_str));
                    
                    // If the query matches, return the value
                    let output_str: Result<String, String> = if raw {
                        // Output raw value directly
                        Ok(ObjCommand::process_raw_value(&json_value))
                    } else {
                        // Extract with normal formatting
                        Ok(extract_raw_value(&json_value).unwrap_or_default())
                    };
                    
                    match output_str {
                        Ok(s) => {
                            ui.text(&s);
                            return Ok(());
                        },
                        Err(e) => {
                            return Err(anyhow!("{}", e));
                        }
                    }
                }
                
                // If we got here, no matching line was found
                Err(anyhow!("No matching result found in the JSONL file"))
            },
            _ => {
                // For other formats, decode as a single value
                let value: Value = Decoder::decode(&input, format_value)
                    .map_err(|_| anyhow!("{}", ERROR_DECODE_FAILED))?;
                
                // Execute query
                let result = query_obj.execute(&value)
                    .map_err(|_| anyhow!("{}", ERROR_QUERY_FAILED))?;
                
                // Convert to JSON value
                let result_str = result.to_string();
                let json_value: Value = serde_json::from_str(&result_str)
                    .unwrap_or_else(|_| Value::String(result_str));
                
                let output_str: Result<String, String> = if raw {
                    Ok(ObjCommand::process_raw_value(&json_value))
                } else {
                    Ok(result.to_string())
                };
                
                match output_str {
                    Ok(s) => {
                        ui.text(&s);
                        Ok(())
                    },
                    Err(e) => {
                        Err(anyhow!("{}", e))
                    }
                }
            }
        }
    }

    fn execute_query_sync(&self, input: &str, query: &str) -> Result<Value, anyhow::Error> {
        let mut program = jq_rs::compile(query)
            .map_err(|e| anyhow!("Failed to compile query '{}': {}", query, e))?;
        let result = program.run(input)
            .map_err(|e| anyhow!("Failed to execute query '{}': {}", query, e))?;
        let value: Value = serde_json::from_str(&result)
            .map_err(|e| anyhow!("Failed to parse query result '{}': {}", result, e))?;
        Ok(value)
    }
}

pub type ExecuteResult = Result<String, anyhow::Error>;

pub fn execute_command(args: &ObjCommand) -> ExecuteResult {
    let output_format = OutputFormat::from_str(&args.format)
        .unwrap_or(OutputFormat::Default);
    let ui = UIManager::with_format(output_format);
    
    match &args.command {
        ObjCommands::Query { format, file, query } => {
            let format_value = Format::from_str(format)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_INVALID_FORMAT, format, e))?;
            
            let input = if file.to_string_lossy() == "-" {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|e| anyhow!("{}: {}", ERROR_STDIN_READ, e))?;
                buffer
            } else {
                fs::read_to_string(file)
                    .map_err(|e| anyhow!("{}: {}: {}", ERROR_FILE_NOT_FOUND, file.display(), e))?
            };
            
            let query_obj = Query::compile(query)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_QUERY_PARSE, query, e))?;
            
            let result = query_obj.execute(&input)
                .map_err(|e| anyhow!("{}: {}", ERROR_QUERY_FAILED, e))?;
            
            Ok(result.to_string())
        },
        ObjCommands::Convert { format, output_format, file } => {
            let input_format = Format::from_str(format)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_INVALID_FORMAT, format, e))?;
            let output_format = Format::from_str(output_format)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_INVALID_FORMAT, output_format, e))?;
            
            let input = if file.to_string_lossy() == "-" {
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| anyhow!("{}: {}", ERROR_STDIN_READ, e))?;
                buffer
            } else {
                fs::read(file)
                    .map_err(|e| anyhow!("{}: {}: {}", ERROR_FILE_NOT_FOUND, file.display(), e))?
            };
            
            let value: Value = Decoder::decode(&input, input_format)
                .map_err(|e| anyhow!("{}: {}", ERROR_DECODE_FAILED, e))?;
            
            let output = Encoder::encode(&value, output_format)
                .map_err(|e| anyhow!("{}: {}", ERROR_ENCODE_FAILED, e))?;
            
            String::from_utf8(output)
                .map_err(|e| anyhow!("{}: {}", ERROR_INVALID_UTF8, e))
        },
        ObjCommands::Extract { format, file, query, raw } => {
            let format_value = Format::from_str(format)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_INVALID_FORMAT, format, e))?;
            
            let input = if file.to_string_lossy() == "-" {
                let mut buffer = Vec::new();
                io::stdin()
                    .read_to_end(&mut buffer)
                    .map_err(|e| anyhow!("{}: {}", ERROR_STDIN_READ, e))?;
                buffer
            } else {
                fs::read(file)
                    .map_err(|e| anyhow!("{}: {}: {}", ERROR_FILE_NOT_FOUND, file.display(), e))?
            };
            
            let value: Value = Decoder::decode(&input, format_value)
                .map_err(|e| anyhow!("{}: {}", ERROR_DECODE_FAILED, e))?;
            
            let query_obj = Query::compile(query)
                .map_err(|e| anyhow!("{}: {}: {}", ERROR_QUERY_PARSE, query, e))?;
            
            let result = query_obj.execute(&value)
                .map_err(|e| anyhow!("{}: {}", ERROR_QUERY_FAILED, e))?;
            
            // Convert result to a suitable Value to use with process_raw_value
            let result_str = result.to_string();
            let json_value: Value = serde_json::from_str(&result_str)
                .unwrap_or_else(|_| Value::String(result_str));
                
            if *raw {
                Ok(ObjCommand::process_raw_value(&json_value))
            } else {
                Ok(result.to_string())
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper struct for tests
    struct Obj;
    
    impl Obj {
        fn new() -> Self {
            Self
        }
        
        fn execute_query(&self, input: &str, query: &str) -> Result<Value, anyhow::Error> {
            let cmd = ObjCommand {
                format: "default".to_string(),
                command: ObjCommands::Query {
                    format: "json".to_string(),
                    file: PathBuf::from("-"),
                    query: query.to_string(),
                }
            };
            cmd.execute_query_sync(input, query)
        }
        
        fn process_raw_value(&self, value: &Value) -> String {
            ObjCommand::process_raw_value(value)
        }
    }

    #[test]
    fn test_format_parsing() {
        let _obj = Obj::new();
        assert!(Format::from_str("json").is_ok());
        assert!(Format::from_str("xml").is_ok());
        assert!(Format::from_str("bson").is_ok());
        assert!(Format::from_str("cbor").is_ok());
        assert!(Format::from_str("invalid").is_err());
    }

    #[test]
    fn test_process_raw_value() {
        let obj = Obj::new();
        let input = json!({"name": "test"});
        assert_eq!(obj.process_raw_value(&input["name"]), "test");
        
        let input = json!({"count": 42});
        assert_eq!(obj.process_raw_value(&input["count"]), "42");
        
        let input = json!({"flag": true});
        assert_eq!(obj.process_raw_value(&input["flag"]), "true");
        
        let input = json!({"value": null});
        assert_eq!(obj.process_raw_value(&input["value"]), "");
    }

    #[test]
    fn test_extract_raw_value() {
        let obj = Obj::new();
        let input = r#"{"name": "test"}"#;
        let result = obj.execute_query(input, ".name").unwrap();
        assert_eq!(obj.process_raw_value(&result), "test");
        
        let input = r#"{"count": 42}"#;
        let result = obj.execute_query(input, ".count").unwrap();
        assert_eq!(obj.process_raw_value(&result), "42");
        
        let input = r#"{"flag": true}"#;
        let result = obj.execute_query(input, ".flag").unwrap();
        assert_eq!(obj.process_raw_value(&result), "true");
        
        let input = r#"{"value": null}"#;
        let result = obj.execute_query(input, ".value").unwrap();
        assert_eq!(obj.process_raw_value(&result), "");
    }

    #[test]
    fn test_execute_query() {
        let obj = Obj::new();
        let input = r#"{"name": "test", "value": 42}"#;
        
        // Test simple field access
        let result = obj.execute_query(input, ".name").unwrap();
        assert_eq!(result.as_str().unwrap(), "test");
        
        // Test numeric value
        let result = obj.execute_query(input, ".value").unwrap();
        assert_eq!(result.as_i64().unwrap(), 42);
        
        // Test invalid query
        assert!(obj.execute_query(input, "invalid").is_err());
        
        // Test invalid input
        let invalid_input = "invalid json";
        assert!(obj.execute_query(invalid_input, ".name").is_err());
    }
} 