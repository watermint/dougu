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

/// Formatter trait for UI output (non-generic methods only)
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

/// Default formatter implementation
#[derive(Clone)]
pub struct DefaultFormatter {
    theme: UITheme,
}

impl DefaultFormatter {
    pub fn new(theme: UITheme) -> Self {
        Self { theme }
    }
}

impl UIFormatter for DefaultFormatter {
    fn title(&self, text: &str) -> String {
        let prefix = "#";
        let colored_text = text.bold().color(&*self.theme.heading_color);
        format!("{} {}", prefix, colored_text)
    }

    fn subtitle(&self, text: &str) -> String {
        let prefix = "##";
        let colored_text = text.color(&*self.theme.heading_color);
        format!("{} {}", prefix, colored_text)
    }

    fn heading(&self, level: u8, text: &str) -> String {
        let prefix = "#".repeat(std::cmp::min(level as usize, 6));
        let colored_text = match level {
            1 => text.bold().color(&*self.theme.heading_color),
            _ => text.color(&*self.theme.heading_color),
        };
        format!("{} {}", prefix, colored_text)
    }

    fn text(&self, text: &str) -> String {
        text.to_string()
    }

    fn success(&self, text: &str) -> String {
        text.color(&*self.theme.success_color).to_string()
    }

    fn error(&self, text: &str) -> String {
        text.color(&*self.theme.error_color).to_string()
    }

    fn info(&self, text: &str) -> String {
        text.color(&*self.theme.info_color).to_string()
    }

    fn warning(&self, text: &str) -> String {
        text.color(&*self.theme.warning_color).to_string()
    }

    fn block(&self, text: &str) -> String {
        text.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn code(&self, text: &str, language: Option<&str>) -> String {
        let lang = language.unwrap_or("");
        format!("```{}\n{}\n```", lang, text)
    }

    fn hr(&self) -> String {
        "-".repeat(self.theme.wrapped_width)
    }

    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        pairs
            .iter()
            .map(|(key, value)| format!("{}: {}", key.bold(), value))
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
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
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
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
                .map(|cell| Cell::new(cell))
                .collect();
            table.add_row(Row::new(cells));
        }
        
        format!("{}", table)
    }
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string_pretty(value)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string(value)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSONL_FORMATTING, e))
    }
}

/// JSON Lines formatter implementation
#[derive(Clone)]
pub struct JsonLinesFormatter;

impl JsonLinesFormatter {
    pub fn new() -> Self {
        Self {}
    }

    fn json_wrap(&self, json_type: &str, data: serde_json::Value) -> String {
        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), serde_json::Value::String(json_type.to_string()));
        
        match data {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    obj.insert(k, v);
                }
            },
            _ => {
                obj.insert("data".to_string(), data);
            }
        }
        
        serde_json::to_string(&serde_json::Value::Object(obj))
            .unwrap_or_else(|_| format!("{{\"type\":\"{}\"}}", json_type))
    }
}

impl UIFormatter for JsonLinesFormatter {
    fn title(&self, text: &str) -> String {
        self.json_wrap("title", serde_json::json!({ "text": text }))
    }

    fn subtitle(&self, text: &str) -> String {
        self.json_wrap("subtitle", serde_json::json!({ "text": text }))
    }

    fn heading(&self, level: u8, text: &str) -> String {
        self.json_wrap("heading", serde_json::json!({
            "level": level,
            "text": text
        }))
    }

    fn text(&self, text: &str) -> String {
        self.json_wrap("text", serde_json::json!({ "text": text }))
    }

    fn success(&self, text: &str) -> String {
        self.json_wrap("success", serde_json::json!({ "text": text }))
    }

    fn error(&self, text: &str) -> String {
        self.json_wrap("error", serde_json::json!({ "text": text }))
    }

    fn info(&self, text: &str) -> String {
        self.json_wrap("info", serde_json::json!({ "text": text }))
    }

    fn warning(&self, text: &str) -> String {
        self.json_wrap("warning", serde_json::json!({ "text": text }))
    }

    fn block(&self, text: &str) -> String {
        self.json_wrap("block", serde_json::json!({ "text": text }))
    }

    fn code(&self, text: &str, language: Option<&str>) -> String {
        self.json_wrap("code", serde_json::json!({
            "text": text,
            "language": language
        }))
    }

    fn hr(&self) -> String {
        self.json_wrap("hr", serde_json::json!({}))
    }

    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        let mut map = serde_json::Map::new();
        for (key, value) in pairs {
            map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        }
        self.json_wrap("key_value_list", serde_json::Value::Object(map))
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        self.json_wrap("list", serde_json::json!({
            "ordered": ordered,
            "items": items
        }))
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let headers_vec: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        
        self.json_wrap("table", serde_json::json!({
            "headers": headers_vec,
            "rows": rows
        }))
    }
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string(value)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        self.json_value(value)
    }
}

/// Markdown formatter implementation
#[derive(Clone)]
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl UIFormatter for MarkdownFormatter {
    fn title(&self, text: &str) -> String {
        format!("# {}", text)
    }

    fn subtitle(&self, text: &str) -> String {
        format!("## {}", text)
    }

    fn heading(&self, level: u8, text: &str) -> String {
        let level = std::cmp::min(level as usize, 6);
        format!("{} {}", "#".repeat(level), text)
    }

    fn text(&self, text: &str) -> String {
        text.to_string()
    }

    fn success(&self, text: &str) -> String {
        format!("> ✅ {}", text)
    }

    fn error(&self, text: &str) -> String {
        format!("> ❌ {}", text)
    }

    fn info(&self, text: &str) -> String {
        format!("> ℹ️ {}", text)
    }

    fn warning(&self, text: &str) -> String {
        format!("> ⚠️ {}", text)
    }

    fn block(&self, text: &str) -> String {
        format!("```\n{}\n```", text)
    }

    fn code(&self, text: &str, language: Option<&str>) -> String {
        let lang = language.unwrap_or("");
        format!("```{}\n{}\n```", lang, text)
    }

    fn hr(&self) -> String {
        "---".to_string()
    }

    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        pairs
            .iter()
            .map(|(key, value)| format!("**{}**: {}", key, value))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if ordered {
                    format!("{}. {}", i + 1, item)
                } else {
                    format!("- {}", item)
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
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
    }
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        Ok(format!("```json\n{}\n```", 
            serde_json::to_string_pretty(value)
                .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))?
        ))
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string(value)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSONL_FORMATTING, e))
    }
}

/// UI Manager for standardized output rendering
pub struct UIManager {
    theme: UITheme,
    format: OutputFormat,
    formatter: Box<dyn UIFormatter>,
}

impl Clone for UIManager {
    fn clone(&self) -> Self {
        let formatter: Box<dyn UIFormatter> = match self.format {
            OutputFormat::JsonLines => Box::new(JsonLinesFormatter::new()),
            OutputFormat::Markdown => Box::new(MarkdownFormatter::new()),
            _ => Box::new(DefaultFormatter::new(self.theme.clone())),
        };
        
        Self {
            theme: self.theme.clone(),
            format: self.format,
            formatter,
        }
    }
}

impl Default for UIManager {
    fn default() -> Self {
        let theme = UITheme::default();
        Self {
            theme: theme.clone(),
            format: OutputFormat::default(),
            formatter: Box::new(DefaultFormatter::new(theme)),
        }
    }
}

impl UIManager {
    /// Create a new UI Manager with the given theme
    pub fn new(theme: UITheme) -> Self {
        Self {
            theme: theme.clone(),
            format: OutputFormat::Default,
            formatter: Box::new(DefaultFormatter::new(theme)),
        }
    }
    
    /// Create a new UI Manager with the given format
    pub fn with_format(format: OutputFormat) -> Self {
        match format {
            OutputFormat::JsonLines => Self::json_mode(),
            OutputFormat::Markdown => Self::markdown_mode(),
            _ => Self::default(),
        }
    }
    
    /// Create a new UI Manager in JSON Lines mode
    pub fn json_mode() -> Self {
        Self {
            theme: UITheme::default(),
            format: OutputFormat::JsonLines,
            formatter: Box::new(JsonLinesFormatter::new()),
        }
    }
    
    /// Create a new UI Manager in Markdown mode
    pub fn markdown_mode() -> Self {
        Self {
            theme: UITheme::default(),
            format: OutputFormat::Markdown,
            formatter: Box::new(MarkdownFormatter::new()),
        }
    }
    
    /// Get the current output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Get the formatter based on current format
    fn formatter(&self) -> &dyn UIFormatter {
        &*self.formatter
    }
    
    /// Create a title (H1) heading and print it
    pub fn title(&self, text: &str) {
        let output = self.formatter().title(text);
        println!("{}", output);
    }
    
    /// Create a subtitle (H2) heading and print it
    pub fn subtitle(&self, text: &str) {
        let output = self.formatter().subtitle(text);
        println!("{}", output);
    }
    
    /// Create a heading with the specified level and print it
    pub fn heading(&self, level: u8, text: &str) {
        let output = self.formatter().heading(level, text);
        println!("{}", output);
    }
    
    /// Format and print plain text
    pub fn text(&self, text: &str) {
        let output = self.formatter().text(text);
        println!("{}", output);
    }
    
    /// Format and print a success message
    pub fn success(&self, text: &str) {
        let output = self.formatter().success(text);
        println!("{}", output);
    }
    
    /// Format and print an error message
    pub fn error(&self, text: &str) {
        let output = self.formatter().error(text);
        println!("{}", output);
    }
    
    /// Format and print an info message
    pub fn info(&self, text: &str) {
        let output = self.formatter().info(text);
        println!("{}", output);
    }
    
    /// Format and print a warning message
    pub fn warning(&self, text: &str) {
        let output = self.formatter().warning(text);
        println!("{}", output);
    }
    
    /// Format text as a block/code block and print it
    pub fn block(&self, text: &str) {
        let output = self.formatter().block(text);
        println!("{}", output);
    }
    
    /// Format text as a code block with optional language and print it
    pub fn code(&self, text: &str, language: Option<&str>) {
        let output = self.formatter().code(text, language);
        println!("{}", output);
    }
    
    /// Create a list with items and print it
    pub fn list<T: Display>(&self, items: &[T], ordered: bool) {
        // Convert items to strings first
        let string_items: Vec<String> = items.iter().map(|item| format!("{}", item)).collect();
        let output = self.formatter().list_string(&string_items, ordered);
        println!("{}", output);
    }
    
    /// Create a table from headers and rows and print it
    pub fn table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) {
        // Convert rows to strings first
        let string_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|cell| format!("{}", cell)).collect())
            .collect();
        
        let output = self.formatter().table_string(headers, &string_rows);
        println!("{}", output);
    }
    
    /// Print a horizontal rule
    pub fn hr(&self) {
        let output = self.formatter().hr();
        println!("{}", output);
    }
    
    /// Print a line break (just a newline)
    pub fn line_break(&self) {
        println!();
    }
    
    /// Format JSON (or other serializable data) into a pretty string and print it
    pub fn json<T: Serialize>(&self, data: &T) -> Result<String, String> {
        // Convert to serde_json::Value first
        let value = serde_json::to_value(data)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_CONVERSION, e))?;
        
        let output = self.formatter().json_value(&value)?;
        println!("{}", output);
        Ok(output)
    }
    
    /// Format data as a single JSON line (compact format, no indentation) and print it
    pub fn jsonl<T: Serialize>(&self, data: &T) -> Result<String, String> {
        // Convert to serde_json::Value first
        let value = serde_json::to_value(data)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_CONVERSION, e))?;
        
        let output = self.formatter().jsonl_value(&value)?;
        println!("{}", output);
        Ok(output)
    }
    
    /// Format and print a key-value list
    pub fn key_value_list(&self, pairs: &[(&str, &str)]) {
        let output = self.formatter().key_value_list(pairs);
        println!("{}", output);
    }
    
    /// Wrap text to the configured width
    pub fn wrap_text(&self, text: &str) -> String {
        wrap(text, self.theme.wrapped_width).join("\n")
    }
}

/// Add formatter methods that just return formatted strings without printing
impl UIManager {
    /// Format text as a title (H1) heading without printing
    fn format_title(&self, text: &str) -> String {
        self.formatter().title(text)
    }
    
    /// Format text as a subtitle (H2) heading without printing
    fn format_subtitle(&self, text: &str) -> String {
        self.formatter().subtitle(text)
    }
    
    /// Format text as a heading with specified level without printing
    fn format_heading(&self, level: u8, text: &str) -> String {
        self.formatter().heading(level, text)
    }
    
    /// Format plain text without printing
    fn format_text(&self, text: &str) -> String {
        self.formatter().text(text)
    }
    
    /// Format a success message without printing
    fn format_success(&self, text: &str) -> String {
        self.formatter().success(text)
    }
    
    /// Format an error message without printing
    fn format_error(&self, text: &str) -> String {
        self.formatter().error(text)
    }
    
    /// Format an info message without printing
    fn format_info(&self, text: &str) -> String {
        self.formatter().info(text)
    }
    
    /// Format a warning message without printing
    fn format_warning(&self, text: &str) -> String {
        self.formatter().warning(text)
    }
    
    /// Format text as a block without printing
    fn format_block(&self, text: &str) -> String {
        self.formatter().block(text)
    }
    
    /// Format text as a code block with optional language without printing
    fn format_code(&self, text: &str, language: Option<&str>) -> String {
        self.formatter().code(text, language)
    }
    
    /// Create a list with items without printing
    fn format_list<T: Display>(&self, items: &[T], ordered: bool) -> String {
        // Convert items to strings first
        let string_items: Vec<String> = items.iter().map(|item| format!("{}", item)).collect();
        self.formatter().list_string(&string_items, ordered)
    }
    
    /// Create a table from headers and rows without printing
    fn format_table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) -> String {
        // Convert rows to strings first
        let string_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|cell| format!("{}", cell)).collect())
            .collect();
        
        self.formatter().table_string(headers, &string_rows)
    }
    
    /// Format a horizontal rule without printing
    fn format_hr(&self) -> String {
        self.formatter().hr()
    }
    
    /// Format a key-value list without printing
    fn format_key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        self.formatter().key_value_list(pairs)
    }
}

/// Format a commandlet result for JSON Lines output
fn format_commandlet_result_json_lines<T: Serialize>(ui: &UIManager, result: &T) -> String {
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
}

/// Format a commandlet result for Markdown output
fn format_commandlet_result_markdown<T: Serialize>(ui: &UIManager, result: &T) -> String {
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

/// Format a commandlet result for Default output
fn format_commandlet_result_default<T: Serialize>(ui: &UIManager, result: &T) -> String {
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

/// Format a commandlet result in a standardized way
pub fn format_commandlet_result<T: Serialize>(ui: &UIManager, result: &T) -> String {
    trace!("{}", ui_messages::TRACE_FORMATTING_RESULT);
    
    // Check the format and handle accordingly
    match ui.format() {
        OutputFormat::JsonLines => format_commandlet_result_json_lines(ui, result),
        OutputFormat::Markdown => format_commandlet_result_markdown(ui, result),
        OutputFormat::Default => format_commandlet_result_default(ui, result),
    }
}

/// Common UI helper for displaying a success message with details
pub fn display_success(ui: &UIManager, message: &str, details: Option<&str>) {
    ui.success(message);
    
    if let Some(detail_text) = details {
        ui.block(detail_text);
    }
}

/// Common UI helper for displaying an error message with details
pub fn display_error(ui: &UIManager, message: &str, details: Option<&str>) {
    ui.error(message);
    
    if let Some(detail_text) = details {
        ui.block(detail_text);
    }
}

// Keep the old format functions for compatibility, but mark them as deprecated
#[deprecated(since = "1.1.0", note = "Use display_success instead")]
pub fn format_success(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut output = vec![ui.format_success(message)];
    
    if let Some(detail_text) = details {
        output.push(ui.format_block(detail_text));
    }
    
    output.join("\n")
}

#[deprecated(since = "1.1.0", note = "Use display_error instead")]
pub fn format_error(ui: &UIManager, message: &str, details: Option<&str>) -> String {
    let mut output = vec![ui.format_error(message)];
    
    if let Some(detail_text) = details {
        output.push(ui.format_block(detail_text));
    }
    
    output.join("\n")
} 