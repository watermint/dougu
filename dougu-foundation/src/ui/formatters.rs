use crate::ui::{UIFormatter, UITheme};
use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use dougu_essentials::obj::{Notation, NotationType};
use dougu_essentials::obj::notation::{JsonNotation, JsonlNotation};
use std::collections::HashMap;
use anyhow::Result;

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
            let cells: Vec<Cell> = row.iter().map(|r| Cell::new(r)).collect();
            table.add_row(Row::new(cells));
        }
        
        table.to_string()
    }
    
    fn json_value(&self, value: &NotationType) -> Result<String, String> {
        let json_notation = JsonNotation::new();
        match json_notation.encode_to_string(value) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
    
    fn jsonl_value(&self, value: &NotationType) -> Result<String, String> {
        let jsonl_notation = JsonlNotation::new();
        match jsonl_notation.encode_to_string(value) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct JsonLinesFormatter;

impl JsonLinesFormatter {
    pub fn new() -> Self {
        Self {}
    }
    
    fn json_wrap(&self, json_type: &str, data: impl Into<NotationType> + Clone) -> Result<String, String> {
        let json_notation = JsonNotation::new();
        let wrapper = NotationType::Object(vec![
            (json_type.to_string(), data.into())
        ].into_iter().collect());
        
        json_notation.encode_to_string(&wrapper)
            .map_err(|e| e.to_string())
    }
}

impl UIFormatter for JsonLinesFormatter {
    fn title(&self, text: &str) -> String {
        self.json_wrap("title", text).unwrap_or_default()
    }
    
    fn subtitle(&self, text: &str) -> String {
        self.json_wrap("subtitle", text).unwrap_or_default()
    }
    
    fn heading(&self, level: u8, text: &str) -> String {
        let mut heading_data = HashMap::new();
        heading_data.insert("level".to_string(), level.to_string());
        heading_data.insert("text".to_string(), text.to_string());
        
        self.json_wrap("heading", heading_data).unwrap_or_default()
    }
    
    fn text(&self, text: &str) -> String {
        self.json_wrap("text", text).unwrap_or_default()
    }
    
    fn success(&self, text: &str) -> String {
        self.json_wrap("success", text).unwrap_or_default()
    }
    
    fn error(&self, text: &str) -> String {
        self.json_wrap("error", text).unwrap_or_default()
    }
    
    fn info(&self, text: &str) -> String {
        self.json_wrap("info", text).unwrap_or_default()
    }
    
    fn warning(&self, text: &str) -> String {
        self.json_wrap("warning", text).unwrap_or_default()
    }
    
    fn block(&self, text: &str) -> String {
        self.json_wrap("block", text).unwrap_or_default()
    }
    
    fn code(&self, text: &str, language: Option<&str>) -> String {
        let mut code_data = HashMap::new();
        code_data.insert("text".to_string(), text.to_string());
        code_data.insert("language".to_string(), language.map(|l| l.to_string()).unwrap_or_default());
        
        self.json_wrap("code", code_data).unwrap_or_default()
    }
    
    fn hr(&self) -> String {
        self.json_wrap("hr", NotationType::Null).unwrap_or_default()
    }
    
    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        let list: Vec<String> = pairs.iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect();
        
        self.json_wrap("key_value_list", list.join("\n")).unwrap_or_default()
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        let items_array: Vec<String> = items.iter().map(|item| item.to_string()).collect();
        
        let mut list_data = HashMap::new();
        list_data.insert("ordered".to_string(), ordered.to_string());
        list_data.insert("items".to_string(), items_array.join(","));
        
        self.json_wrap("list", list_data).unwrap_or_default()
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let headers_array: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        
        let rows_array: Vec<String> = rows.iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect::<Vec<String>>().join(","))
            .collect();
        
        let mut table_data = HashMap::new();
        table_data.insert("headers".to_string(), headers_array.join(","));
        table_data.insert("rows".to_string(), rows_array.join("\n"));
        
        self.json_wrap("table", table_data).unwrap_or_default()
    }
    
    fn json_value(&self, value: &NotationType) -> Result<String, String> {
        self.json_wrap("json", value.clone())
            .map_err(|e| e.to_string())
            .map(|s| s)
    }
    
    fn jsonl_value(&self, value: &NotationType) -> Result<String, String> {
        self.json_wrap("jsonl", value.clone())
            .map_err(|e| e.to_string())
            .map(|s| s)
    }
}

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
        let prefix = "#".repeat(std::cmp::min(level as usize, 6));
        format!("{} {}", prefix, text)
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
        text.lines()
            .map(|line| format!("> {}", line))
            .collect::<Vec<String>>()
            .join("\n")
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
            .join("\n")
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        if ordered {
            items
                .iter()
                .enumerate()
                .map(|(i, item)| format!("{}. {}", i + 1, item))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            items
                .iter()
                .map(|item| format!("- {}", item))
                .collect::<Vec<String>>()
                .join("\n")
        }
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        // Create header row
        let header_row = headers
            .iter()
            .map(|h| format!("{}", h))
            .collect::<Vec<String>>()
            .join(" | ");
        
        // Create separator row
        let separator_row = headers
            .iter()
            .map(|_| "---")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        
        // Create data rows
        let data_rows = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<String>>()
                    .join(" | ")
            })
            .collect::<Vec<String>>()
            .join("\n");
        
        // Combine all parts
        format!("| {} |\n| {} |\n| {} |", header_row, separator_row.join(" | "), data_rows.replace("\n", " |\n| "))
    }
    
    fn json_value(&self, value: &NotationType) -> Result<String, String> {
        let json_notation = JsonNotation::new();
        match json_notation.encode_to_string(value) {
            Ok(json_str) => {
                Ok(format!("```json\n{}\n```", json_str))
            },
            Err(e) => Err(format!("Error serializing to JSON: {}", e))
        }
    }
    
    fn jsonl_value(&self, value: &NotationType) -> Result<String, String> {
        let jsonl_notation = JsonlNotation::default();
        match jsonl_notation.encode_to_string(value) {
            Ok(json_str) => {
                Ok(format!("```json\n{}\n```", json_str))
            },
            Err(e) => Err(format!("Error serializing to JSONL: {}", e))
        }
    }
}

pub struct JsonlFormatter {
    jsonl_notation: JsonlNotation,
}

impl JsonlFormatter {
    pub fn new() -> Self {
        let jsonl_notation = JsonlNotation::default();
        Self { jsonl_notation }
    }

    fn jsonl_value(&self, value: &NotationType) -> Result<String, String> {
        match self.jsonl_notation.encode_to_string(value) {
            Ok(json_str) => Ok(json_str),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl JsonlNotation {
    pub fn new() -> Self {
        Self {}
    }
} 