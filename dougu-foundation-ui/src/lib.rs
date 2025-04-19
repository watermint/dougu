pub mod resources;

use colored::Colorize;
use log::{debug, trace};
use prettytable::{Table, Row, Cell, format};
use resources::ui_messages;
use serde::Serialize;
use std::fmt::Display;
use textwrap::wrap;

// Theme settings (can be expanded to allow different themes)
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
pub struct UIManager {
    theme: UITheme,
}

impl Default for UIManager {
    fn default() -> Self {
        Self {
            theme: UITheme::default(),
        }
    }
}

impl UIManager {
    /// Create a new UI Manager with custom theme
    pub fn new(theme: UITheme) -> Self {
        Self { theme }
    }
    
    /// Create a heading (Markdown-like # Heading)
    pub fn heading(&self, level: u8, text: &str) -> String {
        let prefix = "#".repeat(std::cmp::min(level as usize, 6));
        let colored_text = match level {
            1 => text.bold().color(&*self.theme.heading_color),
            _ => text.color(&*self.theme.heading_color),
        };
        
        format!("{} {}", prefix, colored_text)
    }
    
    /// Create a text block (simple text output)
    pub fn text(&self, text: &str) -> String {
        text.to_string()
    }
    
    /// Create a success message
    pub fn success(&self, text: &str) -> String {
        text.color(&*self.theme.success_color).to_string()
    }
    
    /// Create an error message
    pub fn error(&self, text: &str) -> String {
        text.color(&*self.theme.error_color).to_string()
    }
    
    /// Create an info message
    pub fn info(&self, text: &str) -> String {
        text.color(&*self.theme.info_color).to_string()
    }
    
    /// Create a warning message
    pub fn warning(&self, text: &str) -> String {
        text.color(&*self.theme.warning_color).to_string()
    }
    
    /// Create a block (indented block of text)
    pub fn block(&self, text: &str) -> String {
        text.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    /// Create a code block (```code```)
    pub fn code(&self, text: &str, language: Option<&str>) -> String {
        let lang = language.unwrap_or("");
        format!("```{}\n{}\n```", lang, text)
    }
    
    /// Create a list with items
    pub fn list<T: Display>(&self, items: &[T], ordered: bool) -> String {
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
    
    /// Create a table from headers and rows
    pub fn table<T: Display>(&self, headers: &[&str], rows: &[Vec<T>]) -> String {
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
    
    /// Horizontal rule (Markdown-like ---)
    pub fn hr(&self) -> String {
        "-".repeat(self.theme.wrapped_width)
    }
    
    /// Wraps text to fit within theme's wrapped_width
    pub fn wrap_text(&self, text: &str) -> String {
        wrap(text, self.theme.wrapped_width)
            .join("\n")
    }
    
    /// Format JSON (or other serializable data) into a pretty string
    pub fn format_json<T: Serialize>(&self, data: &T) -> Result<String, String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| format!("{}: {}", ui_messages::ERROR_JSON_FORMATTING, e))
    }
    
    /// Show key-value pairs in a structured format
    pub fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        pairs
            .iter()
            .map(|(key, value)| format!("{}: {}", key.bold(), value))
            .collect::<Vec<String>>()
            .join("\n")
    }
    
    /// Print to console
    pub fn print(&self, text: &str) {
        println!("{}", text);
    }
}

/// Format a commandlet result in a standardized way
pub fn format_commandlet_result<T: Serialize>(ui: &UIManager, result: &T) -> String {
    trace!("{}", ui_messages::TRACE_FORMATTING_RESULT);
    
    match ui.format_json(result) {
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