use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dougu_essentials_obj::{Decoder, Encoder, Format, Query};
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
        match self.command {
            ObjCommands::Query {
                ref format,
                ref file,
                ref query,
            } => self.execute_query(format, file, query).await,
            ObjCommands::Convert {
                ref input_format,
                ref input_file,
                ref output_format,
            } => self.execute_convert(input_format, input_file, output_format).await,
        }
    }

    async fn execute_query(&self, format_str: &str, file_path: &PathBuf, query_str: &str) -> Result<()> {
        let format = Format::from_str(format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let input = self.get_input(file_path)
            .with_context(|| ERROR_FILE_NOT_FOUND)?;
        
        // Decode the input data
        let value: Value = Decoder::decode(&input, format)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        // Create and execute the query
        let query = Query::compile(query_str)
            .with_context(|| ERROR_QUERY_FAILED)?;
        
        let result = query.execute(&value)
            .with_context(|| ERROR_QUERY_FAILED)?;
        
        // Print the result
        println!("{}", result);
        
        Ok(())
    }

    async fn execute_convert(&self, input_format_str: &str, input_file: &PathBuf, output_format_str: &str) -> Result<()> {
        let input_format = Format::from_str(input_format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let output_format = Format::from_str(output_format_str)
            .with_context(|| ERROR_INVALID_FORMAT)?;
        
        let input = self.get_input(input_file)
            .with_context(|| ERROR_FILE_NOT_FOUND)?;
        
        // Convert between formats
        let value: Value = Decoder::decode(&input, input_format)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        match output_format {
            Format::Json | Format::Xml => {
                let output = Encoder::encode_to_string(&value, output_format)
                    .with_context(|| ERROR_DECODE_FAILED)?;
                println!("{}", output);
            },
            _ => {
                let output = Encoder::encode(&value, output_format)
                    .with_context(|| ERROR_DECODE_FAILED)?;
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