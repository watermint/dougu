use crate::ui::{UIFormatter, UITheme, OutputFormat};
use crate::ui::formatters::{DefaultFormatter, JsonLinesFormatter, MarkdownFormatter};
use serde::Serialize;
use std::fmt::Display;

pub struct UIManager {
    theme: UITheme,
    format: OutputFormat,
    formatter: Box<dyn UIFormatter>,
}

impl Clone for UIManager {
    fn clone(&self) -> Self {
        // We need to recreate the formatter based on the format
        let formatter: Box<dyn UIFormatter> = match self.format {
            OutputFormat::Default => Box::new(DefaultFormatter::new(self.theme.clone())),
            OutputFormat::JsonLines => Box::new(JsonLinesFormatter::new()),
            OutputFormat::Markdown => Box::new(MarkdownFormatter::new()),
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
            format: OutputFormat::Default,
            formatter: Box::new(DefaultFormatter::new(theme)),
        }
    }
}

impl UIManager {
    /// Create a new UIManager with the given theme
    pub fn new(theme: UITheme) -> Self {
        Self {
            theme: theme.clone(),
            format: OutputFormat::Default,
            formatter: Box::new(DefaultFormatter::new(theme)),
        }
    }
    
    /// Create a new UIManager with the given output format
    pub fn with_format(format: OutputFormat) -> Self {
        let theme = UITheme::default();
        let formatter: Box<dyn UIFormatter> = match format {
            OutputFormat::Default => Box::new(DefaultFormatter::new(theme.clone())),
            OutputFormat::JsonLines => Box::new(JsonLinesFormatter::new()),
            OutputFormat::Markdown => Box::new(MarkdownFormatter::new()),
        };
        
        Self { theme, format, formatter }
    }
    
    /// Create a UIManager for JSON output
    pub fn json_mode() -> Self {
        let theme = UITheme::default();
        Self {
            theme,
            format: OutputFormat::JsonLines,
            formatter: Box::new(JsonLinesFormatter::new()),
        }
    }
    
    /// Create a UIManager for Markdown output
    pub fn markdown_mode() -> Self {
        let theme = UITheme::default();
        Self {
            theme,
            format: OutputFormat::Markdown,
            formatter: Box::new(MarkdownFormatter::new()),
        }
    }
    
    /// Get the current output format
    pub fn format(&self) -> OutputFormat {
        self.format
    }
    
    /// Get a reference to the formatter
    fn formatter(&self) -> &dyn UIFormatter {
        self.formatter.as_ref()
    }
    
    /// Print a title
    pub fn title(&self, text: &str) {
        println!("{}", self.format_title(text));
    }
    
    /// Print a subtitle
    pub fn subtitle(&self, text: &str) {
        println!("{}", self.format_subtitle(text));
    }
    
    /// Print a heading with specified level
    pub fn heading(&self, level: u8, text: &str) {
        println!("{}", self.format_heading(level, text));
    }
    
    /// Print text
    pub fn text(&self, text: &str) {
        println!("{}", self.format_text(text));
    }
    
    /// Print success message
    pub fn success(&self, text: &str) {
        println!("{}", self.format_success(text));
    }
    
    /// Print error message
    pub fn error(&self, text: &str) {
        println!("{}", self.format_error(text));
    }
    
    /// Print info message
    pub fn info(&self, text: &str) {
        println!("{}", self.format_info(text));
    }
    
    /// Print warning message
    pub fn warning(&self, text: &str) {
        println!("{}", self.format_warning(text));
    }
    
    /// Print block of text
    pub fn block(&self, text: &str) {
        println!("{}", self.format_block(text));
    }
    
    /// Print code block with optional language
    pub fn code(&self, text: &str, language: Option<&str>) {
        println!("{}", self.format_code(text, language));
    }
    
    /// Print a list of items
    pub fn list<T: Display>(&self, items: &[T], ordered: bool) {
        // Convert items to strings
        let items_str: Vec<String> = items.iter().map(|i| i.to_string()).collect();
        println!("{}", self.formatter().list_string(&items_str, ordered));
    }
    
    /// Print a table
    pub fn table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) {
        // Convert rows to strings
        let rows_str: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect();
        
        println!("{}", self.formatter().table_string(headers, &rows_str));
    }
    
    /// Print a horizontal rule
    pub fn hr(&self) {
        println!("{}", self.format_hr());
    }
    
    /// Print a line break
    pub fn line_break(&self) {
        println!();
    }
    
    /// Serialize to JSON
    pub fn json<T: Serialize>(&self, data: &T) -> Result<String, String> {
        // Convert to Value first
        let value = match serde_json::to_value(data) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to serialize to JSON: {}", e)),
        };
        
        // Format using the current formatter
        self.formatter().json_value(&value)
    }
    
    /// Serialize to JSON Lines
    pub fn jsonl<T: Serialize>(&self, data: &T) -> Result<String, String> {
        // Convert to Value first
        let value = match serde_json::to_value(data) {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to serialize to JSON: {}", e)),
        };
        
        // Format using the current formatter
        self.formatter().jsonl_value(&value)
    }
    
    /// Print a key-value list
    pub fn key_value_list(&self, pairs: &[(&str, &str)]) {
        println!("{}", self.format_key_value_list(pairs));
    }
    
    /// Wrap text to the configured width
    pub fn wrap_text(&self, text: &str) -> String {
        textwrap::wrap(text, self.theme.wrapped_width).join("\n")
    }
}

impl UIManager {
    /// Format a title
    pub fn format_title(&self, text: &str) -> String {
        self.formatter().title(text)
    }
    
    /// Format a subtitle
    pub fn format_subtitle(&self, text: &str) -> String {
        self.formatter().subtitle(text)
    }
    
    /// Format a heading
    pub fn format_heading(&self, level: u8, text: &str) -> String {
        self.formatter().heading(level, text)
    }
    
    /// Format text
    pub fn format_text(&self, text: &str) -> String {
        self.formatter().text(text)
    }
    
    /// Format success message
    pub fn format_success(&self, text: &str) -> String {
        self.formatter().success(text)
    }
    
    /// Format error message
    pub fn format_error(&self, text: &str) -> String {
        self.formatter().error(text)
    }
    
    /// Format info message
    pub fn format_info(&self, text: &str) -> String {
        self.formatter().info(text)
    }
    
    /// Format warning message
    pub fn format_warning(&self, text: &str) -> String {
        self.formatter().warning(text)
    }
    
    /// Format block of text
    pub fn format_block(&self, text: &str) -> String {
        self.formatter().block(text)
    }
    
    /// Format code block
    pub fn format_code(&self, text: &str, language: Option<&str>) -> String {
        self.formatter().code(text, language)
    }
    
    /// Format a list of items
    pub fn format_list<T: Display>(&self, items: &[T], ordered: bool) -> String {
        let items_str: Vec<String> = items.iter().map(|i| i.to_string()).collect();
        self.formatter().list_string(&items_str, ordered)
    }
    
    /// Format a table
    pub fn format_table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) -> String {
        let rows_str: Vec<Vec<String>> = rows
            .iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect())
            .collect();
        
        self.formatter().table_string(headers, &rows_str)
    }
    
    /// Format a horizontal rule
    pub fn format_hr(&self) -> String {
        self.formatter().hr()
    }
    
    /// Format a key-value list
    pub fn format_key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        self.formatter().key_value_list(pairs)
    }
}

// Utility functions for formatting commandlet results
pub fn format_commandlet_result<T: Serialize>(ui: &UIManager, result: &T) -> String {
    match ui.format() {
        OutputFormat::JsonLines => format_commandlet_result_json_lines(ui, result),
        OutputFormat::Markdown => format_commandlet_result_markdown(ui, result),
        OutputFormat::Default => format_commandlet_result_default(ui, result),
    }
}

fn format_commandlet_result_json_lines<T: Serialize>(ui: &UIManager, result: &T) -> String {
    match ui.jsonl(result) {
        Ok(json) => json,
        Err(e) => format!("{{ \"error\": \"Failed to serialize result: {}\" }}", e),
    }
}

fn format_commandlet_result_markdown<T: Serialize>(ui: &UIManager, result: &T) -> String {
    // Try to serialize as JSON for display
    match ui.json(result) {
        Ok(json) => {
            // For markdown mode, we'll make a more readable output
            let mut output = String::new();
            
            output.push_str(&ui.format_title("Command Result"));
            output.push_str("\n\n");
            
            // If the result is a complex object, display as JSON
            if json.contains("{") || json.contains("[") {
                // Try to parse the JSON to prettify it
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&value) {
                        output.push_str(&ui.format_code(&pretty, Some("json")));
                        return output;
                    }
                }
                
                // Fallback to regular JSON
                output.push_str(&ui.format_code(&json, Some("json")));
            } else {
                // For simple values, just display them directly
                output.push_str(&ui.format_text(&json));
            }
            
            output
        },
        Err(e) => {
            let mut output = String::new();
            output.push_str(&ui.format_title("Error"));
            output.push_str("\n\n");
            output.push_str(&ui.format_error(&format!("Failed to serialize result: {}", e)));
            output
        }
    }
}

fn format_commandlet_result_default<T: Serialize>(ui: &UIManager, result: &T) -> String {
    // Try to serialize as JSON for inspection
    match serde_json::to_value(result) {
        Ok(value) => {
            match value {
                serde_json::Value::Object(map) => {
                    // Handle object type results
                    let mut output = String::new();
                    
                    // Check if it's an error response
                    if let Some(error) = map.get("error") {
                        if let Some(err_str) = error.as_str() {
                            output.push_str(&ui.format_error(err_str));
                            
                            // Add details if available
                            if let Some(details) = map.get("details") {
                                if let Some(details_str) = details.as_str() {
                                    output.push_str("\n");
                                    output.push_str(&ui.format_block(details_str));
                                }
                            }
                            
                            return output;
                        }
                    }
                    
                    // For regular objects, format as key-value pairs
                    for (key, val) in map.iter() {
                        let val_str = match val {
                            serde_json::Value::String(s) => s.clone(),
                            _ => val.to_string(),
                        };
                        
                        output.push_str(&format!("{}: {}\n", key, val_str));
                    }
                    
                    output
                },
                serde_json::Value::Array(arr) => {
                    // Handle array type results
                    let items: Vec<String> = arr.iter()
                        .map(|v| v.to_string())
                        .collect();
                    
                    ui.format_list(&items, false)
                },
                _ => {
                    // Handle primitive values
                    value.to_string()
                }
            }
        },
        Err(e) => {
            ui.format_error(&format!("Failed to serialize result: {}", e))
        }
    }
} 