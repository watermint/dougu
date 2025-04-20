use anyhow::{Context, Result};
use base64::Engine;
use clap::{Parser, Subcommand};
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
                ref input_format,
                ref input_file,
                ref output_format,
            } => self.execute_convert(input_format, input_file, output_format, &ui, use_json_output).await,
        }
    }

    async fn execute_query(&self, format_str: &str, file_path: &PathBuf, query_str: &str, ui: &UIManager, json_mode: bool) -> Result<()> {
        let result: Result<serde_json::Value, anyhow::Error> = (|| {
            let format = Format::from_str(format_str)
                .with_context(|| ERROR_INVALID_FORMAT)?;
            let input = self.get_input(file_path)
                .with_context(|| ERROR_FILE_NOT_FOUND)?;
            let value: Value = Decoder::decode(&input, format)
                .with_context(|| ERROR_DECODE_FAILED)?;
            let query = Query::compile(query_str)
                .with_context(|| ERROR_QUERY_FAILED)?;
            let result = query.execute(&value)
                .with_context(|| ERROR_QUERY_FAILED)?;
            let result_str = result.to_string();
            let output = match serde_json::from_str::<serde_json::Value>(&result_str) {
                Ok(json_value) => json_value,
                Err(_) => serde_json::json!({"raw": result_str}),
            };
            Ok(output)
        })();
        if json_mode {
            match result {
                Ok(val) => {
                    let json_string = serde_json::to_string_pretty(&val).unwrap();
                    ui.print(&json_string);
                },
                Err(e) => {
                    let error_json = serde_json::json!({"error": e.to_string()}).to_string();
                    ui.print(&error_json);
                },
            }
        } else {
            match result {
                Ok(val) => {
                    ui.print(&ui.heading(1, "Query Result"));
                    let formatted = ui.format_json(&val).unwrap_or_else(|_| format!("{}", serde_json::to_string_pretty(&val).unwrap_or_default()));
                    ui.print(&ui.heading(2, "Query Output"));
                    ui.print(&ui.code(&formatted, Some("json")));
                },
                Err(e) => {
                    ui.print(&ui.error(&e.to_string()));
                }
            }
        }
        Ok(())
    }

    async fn execute_convert(&self, input_format_str: &str, input_file: &PathBuf, output_format_str: &str, ui: &UIManager, json_mode: bool) -> Result<()> {
        let result: Result<serde_json::Value, anyhow::Error> = (|| {
            let input_format = Format::from_str(input_format_str)
                .with_context(|| ERROR_INVALID_FORMAT)?;
            let output_format = Format::from_str(output_format_str)
                .with_context(|| ERROR_INVALID_FORMAT)?;
            let input = self.get_input(input_file)
                .with_context(|| ERROR_FILE_NOT_FOUND)?;
            let value: Value = Decoder::decode(&input, input_format)
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
                    ui.print(&json_string);
                },
                Err(e) => {
                    let error_json = serde_json::json!({"error": e.to_string()}).to_string();
                    ui.print(&error_json);
                },
            }
        } else {
            match result {
                Ok(val) => {
                    ui.print(&ui.heading(1, "Format Conversion"));
                    ui.print(&ui.key_value_list(&[
                        ("From", input_format_str),
                        ("To", output_format_str),
                        ("File", &input_file.to_string_lossy()),
                    ]));
                    ui.print(&ui.heading(2, "Conversion Result"));
                    if let Some(result) = val.get("result") {
                        let formatted = ui.format_json(result).unwrap_or_else(|_| format!("{}", result));
                        let language = match output_format_str.to_lowercase().as_str() {
                            "json" => "json",
                            "xml" => "xml",
                            _ => "",
                        };
                        ui.print(&ui.code(&formatted, Some(language)));
                    } else if let Some(info) = val.get("info") {
                        ui.print(&ui.info(info.as_str().unwrap_or("Binary data written to stdout")));
                    }
                },
                Err(e) => {
                    ui.print(&ui.error(&e.to_string()));
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