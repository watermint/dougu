use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dougu_essentials_obj::{Decoder, Encoder, Format, Query};
use dougu_foundation_ui::UIManager;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

mod resources;
use resources::messages::*;

#[derive(Parser, Serialize, Deserialize)]
#[command(name = "obj")]
#[command(about = CMD_OBJ_DESCRIPTION)]
pub struct ObjCommand {
    #[command(subcommand)]
    command: ObjCommands,
}

#[derive(Subcommand, Serialize, Deserialize)]
enum ObjCommands {
    /// Execute a query on an object notation file
    #[command(about = CMD_QUERY_DESCRIPTION)]
    Query {
        /// Input format (json, bson, xml, cbor)
        #[arg(help = ARG_FORMAT_DESCRIPTION)]
        format: String,

        /// Input file path (use - for stdin)
        #[arg(help = ARG_FILE_DESCRIPTION)]
        file: PathBuf,

        /// Query string in jq-like format
        #[arg(help = ARG_QUERY_DESCRIPTION)]
        query: String,
    },

    /// Convert between object notation formats
    #[command(about = CMD_CONVERT_DESCRIPTION)]
    Convert {
        /// Input format (json, bson, xml, cbor)
        #[arg(help = ARG_FORMAT_DESCRIPTION)]
        input_format: String,

        /// Input file path (use - for stdin)
        #[arg(help = ARG_FILE_DESCRIPTION)]
        input_file: PathBuf,

        /// Output format (json, bson, xml, cbor)
        #[arg(help = ARG_OUTPUT_FORMAT_DESCRIPTION)]
        output_format: String,
    },
}

impl ObjCommand {
    pub async fn execute(self) -> Result<()> {
        // Create UI manager for formatted output
        let ui = UIManager::default();
        
        match self.command {
            ObjCommands::Query {
                ref format,
                ref file,
                ref query,
            } => self.execute_query(format, file, query, &ui).await,
            ObjCommands::Convert {
                ref input_format,
                ref input_file,
                ref output_format,
            } => self.execute_convert(input_format, input_file, output_format, &ui).await,
        }
    }

    async fn execute_query(&self, format_str: &str, file_path: &PathBuf, query_str: &str, ui: &UIManager) -> Result<()> {
        let format = Format::from_str(format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let input = self.get_input(file_path)
            .with_context(|| ERROR_FILE_NOT_FOUND)?;
        
        // Display operation heading
        ui.print(&ui.heading(1, "Query Result"));
        
        // Decode the input data
        let value: Value = Decoder::decode(&input, format)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        // Create and execute the query
        let query = Query::compile(query_str)
            .with_context(|| ERROR_QUERY_FAILED)?;
        
        let result = query.execute(&value)
            .with_context(|| ERROR_QUERY_FAILED)?;
        
        // Print the formatted result using the UI manager
        // Convert jaq_interpret::val::Val to a string first, then parse to serde_json::Value
        let result_str = result.to_string();
        match serde_json::from_str::<serde_json::Value>(&result_str) {
            Ok(json_value) => {
                match ui.format_json(&json_value) {
                    Ok(formatted) => {
                        ui.print(&ui.heading(2, "Query Output"));
                        ui.print(&ui.code(&formatted, Some("json")));
                    },
                    Err(_) => {
                        // Fallback to direct printing if JSON formatting fails
                        ui.print(&ui.heading(2, "Query Output (Raw)"));
                        ui.print(&result_str);
                    }
                }
            },
            Err(_) => {
                // If the result cannot be parsed as JSON, display as raw string
                ui.print(&ui.heading(2, "Query Output (Raw)"));
                ui.print(&result_str);
            }
        }
        
        Ok(())
    }

    async fn execute_convert(&self, input_format_str: &str, input_file: &PathBuf, output_format_str: &str, ui: &UIManager) -> Result<()> {
        let input_format = Format::from_str(input_format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let output_format = Format::from_str(output_format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let input = self.get_input(input_file)
            .with_context(|| ERROR_FILE_NOT_FOUND)?;
        
        // Display operation heading
        ui.print(&ui.heading(1, "Format Conversion"));
        ui.print(&ui.key_value_list(&[
            ("From", input_format_str),
            ("To", output_format_str),
            ("File", &input_file.to_string_lossy()),
        ]));
        
        // Convert between formats
        let value: Value = Decoder::decode(&input, input_format)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        ui.print(&ui.heading(2, "Conversion Result"));
        
        match output_format {
            Format::Json | Format::Xml => {
                let output = Encoder::encode_to_string(&value, output_format)
                    .with_context(|| ERROR_DECODE_FAILED)?;
                
                // For text formats, use code block with syntax highlighting
                let language = match output_format {
                    Format::Json => "json",
                    Format::Xml => "xml",
                    _ => "", // Should never happen based on the match condition
                };
                
                ui.print(&ui.code(&output, Some(language)));
            },
            _ => {
                let output = Encoder::encode(&value, output_format)
                    .with_context(|| ERROR_DECODE_FAILED)?;
                
                // For binary formats, just write to stdout directly
                ui.print(&ui.info("Binary data written to stdout"));
                io::stdout().write_all(&output)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_format_parsing() {
        assert!(Format::from_str("json").is_ok());
        assert!(Format::from_str("xml").is_ok());
        assert!(Format::from_str("bson").is_ok());
        assert!(Format::from_str("cbor").is_ok());
        assert!(Format::from_str("invalid").is_err());
    }
} 