pub mod resources;

use colored::Colorize;
use log::{debug, trace, info};
use prettytable::{Table, Row, Cell, format};
use resources::ui_messages;
use serde::Serialize;
use std::fmt::Display;
use std::str::FromStr;
use textwrap::wrap;

mod formatters;
mod manager;

// Re-export main types
pub use formatters::*;
pub use manager::*;

// Define a format enum to represent the available output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Default,
    JsonLines,
    Markdown,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Default
    }
}

impl FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(OutputFormat::Default),
            "json" | "jsonl" | "jsonlines" => Ok(OutputFormat::JsonLines),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// UI Theme that controls colors and display preferences
#[derive(Debug, Clone)]
pub struct UITheme {
    pub heading_color: String,
    pub success_color: String,
    pub error_color: String,
    pub info_color: String,
    pub warning_color: String,
    pub table_header_color: String,
    pub wrapped_width: usize,
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            heading_color: "blue".to_string(),
            success_color: "green".to_string(),
            error_color: "red".to_string(),
            info_color: "cyan".to_string(),
            warning_color: "yellow".to_string(),
            table_header_color: "cyan".to_string(),
            wrapped_width: 80,
        }
    }
}

/// Formatter trait for UI output
pub trait UIFormatter: Send + Sync {
    fn title(&self, text: &str) -> String;
    fn subtitle(&self, text: &str) -> String;
    fn heading(&self, level: u8, text: &str) -> String;
    fn text(&self, text: &str) -> String;
    fn success(&self, text: &str) -> String;
    fn error(&self, text: &str) -> String;
    fn info(&self, text: &str) -> String;
    fn warning(&self, text: &str) -> String;
    fn block(&self, text: &str) -> String;
    fn code(&self, text: &str, language: Option<&str>) -> String;
    fn hr(&self) -> String;
    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String;
    
    // Methods for handling generic types with type erasure - using String for Display
    fn list_string(&self, items: &[String], ordered: bool) -> String;
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String;
    
    // Methods for serialization using serde_json::Value
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String>;
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String>;
}

// Helper functions for UI
pub fn display_success(ui: &UIManager, message: &str, details: Option<&str>) {
    ui.success(message);
    if let Some(details) = details {
        ui.text(details);
    }
}

pub fn display_error(ui: &UIManager, message: &str, details: Option<&str>) {
    ui.error(message);
    if let Some(details) = details {
        ui.block(details);
    }
}

pub fn format_success(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut result = ui.format_success(message);
    if let Some(details) = details {
        result = format!("{}\n{}", result, ui.format_text(details));
    }
    result
}

pub fn format_error(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut result = ui.format_error(message);
    if let Some(details) = details {
        result = format!("{}\n{}", result, ui.format_block(details));
    }
    result
} 