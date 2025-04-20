pub mod resources;

use colored::Colorize;
use log::{debug, trace, info};
use prettytable::{Table, Row, Cell, format};
use resources::ui_messages;
use serde::Serialize;
use std::fmt::Display;
use std::str::FromStr;
use textwrap::wrap;

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

/// UI Manager for standardized output rendering
#[derive(Debug, Clone)]
pub struct UIManager {
    theme: UITheme,
    format: OutputFormat,
}

impl Default for UIManager {
    fn default() -> Self {
        Self {
            theme: UITheme::default(),
            format: OutputFormat::Default,
        }
    }
}

impl UIManager {
    /// Create a new UI Manager with custom theme
    pub fn new(theme: UITheme) -> Self {
        Self { 
            theme,
            format: OutputFormat::Default,
        }
    }
    
    /// Create a UIManager with a specific output format
    pub fn with_format(format: OutputFormat) -> Self {
        Self {
            theme: UITheme::default(),
            format,
        }
    }
    
    /// Create a UIManager that outputs only JSON (no formatting)
    pub fn json_mode() -> Self {
        Self { 
            theme: UITheme::default(),
            format: OutputFormat::JsonLines,
        }
    }
    
    /// Create a UIManager that outputs markdown
    pub fn markdown_mode() -> Self {
        Self {
            theme: UITheme::default(),
            format: OutputFormat::Markdown,
        }
    }
    
    /// Get the current output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Create a title (equivalent to a level 1 heading) and print it
    pub fn title(&self, text: &str) -> String {
        let prefix = "#";
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "title",
                    "text": text
                })) {
                    json
                } else {
                    format!("{} {}", prefix, text)
                }
            },
            OutputFormat::Markdown => format!("{} {}", prefix, text),
            OutputFormat::Default => {
                let colored_text = text.bold().color(&*self.theme.heading_color);
                format!("{} {}", prefix, colored_text)
            }
        };
        println!("{}", output);
        output
    }
    
    /// Create a subtitle (equivalent to a level 2 heading) and print it
    pub fn subtitle(&self, text: &str) -> String {
        let prefix = "##";
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "subtitle",
                    "text": text
                })) {
                    json
                } else {
                    format!("{} {}", prefix, text)
                }
            },
            OutputFormat::Markdown => format!("{} {}", prefix, text),
            OutputFormat::Default => {
                let colored_text = text.color(&*self.theme.heading_color);
                format!("{} {}", prefix, colored_text)
            }
        };
        println!("{}", output);
        output
    }
    
    /// Create a heading (Markdown-like # Heading) and print it
    #[deprecated(since = "0.2.0", note = "Use title() or subtitle() instead")]
    pub fn heading(&self, level: u8, text: &str) -> String {
        let prefix = "#".repeat(std::cmp::min(level as usize, 6));
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "heading",
                    "level": level,
                    "text": text
                })) {
                    json
                } else {
                    format!("{} {}", prefix, text)
                }
            },
            OutputFormat::Markdown => format!("{} {}", prefix, text),
            OutputFormat::Default => {
                let colored_text = match level {
                    1 => text.bold().color(&*self.theme.heading_color),
                    _ => text.color(&*self.theme.heading_color),
                };
                format!("{} {}", prefix, colored_text)
            }
        };
        println!("{}", output);
        output
    }
    
    /// Create a text block (simple text output) and print it
    pub fn text(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "text",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            _ => text.to_string()
        };
        println!("{}", output);
        output
    }
    
    /// Create and print a success message
    pub fn success(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "success",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("✅ {}", text),
            OutputFormat::Default => text.color(&*self.theme.success_color).to_string()
        };
        println!("{}", output);
        output
    }
    
    /// Create and print an error message
    pub fn error(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "error",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("❌ {}", text),
            OutputFormat::Default => text.color(&*self.theme.error_color).to_string()
        };
        println!("{}", output);
        output
    }
    
    /// Create and print an info message
    pub fn info(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "info",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("ℹ️ {}", text),
            OutputFormat::Default => text.color(&*self.theme.info_color).to_string()
        };
        println!("{}", output);
        output
    }
    
    /// Create and print a warning message
    pub fn warning(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "warning",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("⚠️ {}", text),
            OutputFormat::Default => text.color(&*self.theme.warning_color).to_string()
        };
        println!("{}", output);
        output
    }
    
    /// Create a block (indented block of text) and print it
    pub fn block(&self, text: &str) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "block",
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("> {}", text.replace("\n", "\n> ")),
            OutputFormat::Default => {
                text.lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        };
        println!("{}", output);
        output
    }
    
    /// Create and print a code block (```code```)
    pub fn code(&self, text: &str, language: Option<&str>) -> String {
        let lang = language.unwrap_or("");
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "code",
                    "language": lang,
                    "text": text
                })) {
                    json
                } else {
                    text.to_string()
                }
            },
            OutputFormat::Markdown => format!("```{}\n{}\n```", lang, text),
            OutputFormat::Default => format!("```{}\n{}\n```", lang, text)
        };
        println!("{}", output);
        output
    }
    
    /// Create a list with items and print it
    pub fn list<T: Display>(&self, items: &[T], ordered: bool) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                let items_vec: Vec<String> = items.iter().map(|item| format!("{}", item)).collect();
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "list",
                    "ordered": ordered,
                    "items": items_vec
                })) {
                    json
                } else {
                    items.iter()
                        .enumerate()
                        .map(|(i, item)| {
                            if ordered {
                                format!("{}. {}", i + 1, item)
                            } else {
                                format!("• {}", item)
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            },
            OutputFormat::Markdown | OutputFormat::Default => {
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        if ordered {
                            format!("{}. {}", i + 1, item)
                        } else {
                            format!("• {}", item)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        };
        println!("{}", output);
        output
    }
    
    /// Create a table from headers and rows and print it
    pub fn table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                let headers_vec: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
                let rows_vec: Vec<Vec<String>> = rows.iter()
                    .map(|row| row.iter().map(|cell| format!("{}", cell)).collect())
                    .collect();
                
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "table",
                    "headers": headers_vec,
                    "rows": rows_vec
                })) {
                    json
                } else {
                    format!("{}", Table::new()) // Empty table as fallback
                }
            },
            OutputFormat::Markdown => {
                let mut md_table = String::new();
                
                // Add headers
                md_table.push_str("|");
                for header in headers {
                    md_table.push_str(&format!(" {} |", header));
                }
                md_table.push_str("\n|");
                
                // Add separator row
                for _ in headers {
                    md_table.push_str(" --- |");
                }
                md_table.push_str("\n");
                
                // Add data rows
                for row in rows {
                    md_table.push_str("|");
                    for cell in row {
                        md_table.push_str(&format!(" {} |", cell));
                    }
                    md_table.push_str("\n");
                }
                
                md_table
            },
            OutputFormat::Default => {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                
                // Add header row
                let header_cells: Vec<Cell> = headers
                    .iter()
                    .map(|h| Cell::new(h).with_style(prettytable::Attr::ForegroundColor(
                        match self.theme.table_header_color.as_str() {
                            "blue" => prettytable::color::BLUE,
                            "green" => prettytable::color::GREEN,
                            "red" => prettytable::color::RED,
                            "cyan" => prettytable::color::CYAN,
                            "yellow" => prettytable::color::YELLOW,
                            "magenta" => prettytable::color::MAGENTA,
                            _ => prettytable::color::CYAN,
                        }
                    )))
                    .collect();
                
                table.add_row(Row::new(header_cells));
                
                // Add data rows
                for row in rows {
                    let cells: Vec<Cell> = row
                        .iter()
                        .map(|cell| Cell::new(&format!("{}", cell)))
                        .collect();
                    table.add_row(Row::new(cells));
                }
                
                format!("{}", table)
            }
        };
        println!("{}", output);
        output
    }
    
    /// Print a horizontal rule (Markdown-like ---)
    pub fn hr(&self) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "hr"
                })) {
                    json
                } else {
                    "-".repeat(self.theme.wrapped_width)
                }
            },
            OutputFormat::Markdown => "---".to_string(),
            OutputFormat::Default => "-".repeat(self.theme.wrapped_width)
        };
        println!("{}", output);
        output
    }
    
    /// Print a blank line
    pub fn line_break(&self) {
        match self.format {
            OutputFormat::JsonLines => {
                if let Ok(json) = serde_json::to_string(&serde_json::json!({
                    "type": "line_break"
                })) {
                    println!("{}", json);
                } else {
                    println!();
                }
            },
            _ => println!()
        }
    }
    
    /// Format JSON (or other serializable data) into a pretty string and print it
    pub fn json<T: Serialize>(&self, data: &T) -> Result<String, String> {
        let output = match self.format {
            OutputFormat::JsonLines => {
                serde_json::to_string(data)
                    .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))
            },
            _ => serde_json::to_string_pretty(data)
                .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))
        }?;
        println!("{}", output);
        Ok(output)
    }
    
    /// Format data as a single JSON line (compact format, no indentation) and print it
    pub fn jsonl<T: Serialize>(&self, data: &T) -> Result<String, String> {
        let output = serde_json::to_string(data)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSONL_FORMATTING, e))?;
        println!("{}", output);
        Ok(output)
    }
    
    /// Show key-value pairs in a structured format and print them
    pub fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        let output = match self.format {
            OutputFormat::JsonLines => {
                let mut map = serde_json::Map::new();
                for (key, value) in pairs {
                    map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                }
                if let Ok(json) = serde_json::to_string(&serde_json::Value::Object(map)) {
                    json
                } else {
                    pairs
                        .iter()
                        .map(|(key, value)| format!("{}: {}", key, value))
                        .collect::<Vec<String>>()
                        .join("\n")
                }
            },
            OutputFormat::Markdown => {
                pairs
                    .iter()
                    .map(|(key, value)| format!("**{}**: {}", key, value))
                    .collect::<Vec<String>>()
                    .join("\n")
            },
            OutputFormat::Default => {
                pairs
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key.bold(), value))
                    .collect::<Vec<String>>()
                    .join("\n")
            }
        };
        println!("{}", output);
        output
    }
    
    /// Wraps text to fit within theme's wrapped_width
    pub fn wrap_text(&self, text: &str) -> String {
        wrap(text, self.theme.wrapped_width)
            .join("\n")
    }
}

/// Format a commandlet result in a standardized way
pub fn format_commandlet_result<T: Serialize>(ui: &UIManager, result: &T) -> String {
    trace!("{}", ui_messages::TRACE_FORMATTING_RESULT);
    
    // Check the format and handle accordingly
    match ui.format() {
        OutputFormat::JsonLines => {
            // For JSON Lines format, use compact JSON formatting (no indentation)
            match ui.jsonl(result) {
                Ok(formatted_json) => {
                    debug!("{}", ui_messages::DEBUG_RESULT_FORMATTED);
                    formatted_json
                },
                Err(e) => {
                    debug!("{}: {}", ui_messages::ERROR_FORMATTING_RESULT, e);
                    format!("{}", ui_messages::ERROR_RESULT_FALLBACK)
                }
            }
        },
        OutputFormat::Markdown => {
            // For markdown format, create a more human-readable output
            // First serialize to a value to examine the structure
            if let Ok(json_value) = serde_json::to_value(result) {
                if let Some(obj) = json_value.as_object() {
                    // Create a markdown table from the object
                    let mut markdown = String::new();
                    markdown.push_str("# Result\n\n");
                    
                    // Create a table with key-value pairs
                    markdown.push_str("| Property | Value |\n");
                    markdown.push_str("|----------|-------|\n");
                    
                    for (key, value) in obj {
                        let value_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            _ => value.to_string(),
                        };
                        markdown.push_str(&format!("| **{}** | {} |\n", key, value_str));
                    }
                    
                    markdown
                } else {
                    // If it's not an object, just format as JSON with markdown code block
                    match ui.json(result) {
                        Ok(formatted_json) => format!("```json\n{}\n```", formatted_json),
                        Err(e) => {
                            debug!("{}: {}", ui_messages::ERROR_FORMATTING_RESULT, e);
                            format!("{}", ui_messages::ERROR_RESULT_FALLBACK)
                        }
                    }
                }
            } else {
                // Fallback to JSON if we can't convert to a value
                match ui.json(result) {
                    Ok(formatted_json) => format!("```json\n{}\n```", formatted_json),
                    Err(e) => {
                        debug!("{}: {}", ui_messages::ERROR_FORMATTING_RESULT, e);
                        format!("{}", ui_messages::ERROR_RESULT_FALLBACK)
                    }
                }
            }
        },
        OutputFormat::Default => {
            // For default format, also use JSON but with a more human-readable output if possible
            if let Ok(json_value) = serde_json::to_value(result) {
                if let Some(obj) = json_value.as_object() {
                    // Create a simple text output
                    let mut text = String::new();
                    
                    for (key, value) in obj {
                        let value_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            _ => value.to_string(),
                        };
                        text.push_str(&format!("{}: {}\n", key, value_str));
                    }
                    
                    text
                } else {
                    // If it's not an object, just format as JSON
                    match ui.json(result) {
                        Ok(formatted_json) => formatted_json,
                        Err(e) => {
                            debug!("{}: {}", ui_messages::ERROR_FORMATTING_RESULT, e);
                            format!("{}", ui_messages::ERROR_RESULT_FALLBACK)
                        }
                    }
                }
            } else {
                // Fallback to JSON if we can't convert to a value
                match ui.json(result) {
                    Ok(formatted_json) => formatted_json,
                    Err(e) => {
                        debug!("{}: {}", ui_messages::ERROR_FORMATTING_RESULT, e);
                        format!("{}", ui_messages::ERROR_RESULT_FALLBACK)
                    }
                }
            }
        }
    }
}

/// Common UI helper for displaying a success message with details
pub fn format_success(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut output = vec![ui.success(message)];
    
    if let Some(detail_text) = details {
        output.push(ui.block(detail_text));
    }
    
    output.join("\n")
}

/// Common UI helper for displaying an error message with details
pub fn format_error(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut output = vec![ui.error(message)];
    
    if let Some(detail_text) = details {
        output.push(ui.block(detail_text));
    }
    
    output.join("\n")
} 