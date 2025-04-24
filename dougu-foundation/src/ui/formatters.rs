use crate::ui::{UIFormatter, UITheme};
use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use dougu_essentials::obj::{Notation, NotationType};
use std::collections::HashMap;

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
    
    fn json_value(&self, value: &dyn Notation) -> Result<String, String> {
        Ok(value.encode_to_string()?)
    }
    
    fn jsonl_value(&self, value: &dyn Notation) -> Result<String, String> {
        Ok(value.encode_to_string()?)
    }
}

pub struct JsonLinesFormatter;

impl JsonLinesFormatter {
    pub fn new() -> Self {
        Self {}
    }
    
    fn json_wrap(&self, json_type: &str, data: impl Notation) -> String {
        let mut wrapper = HashMap::new();
        wrapper.insert("type".to_string(), json_type.to_string());
        wrapper.insert("data".to_string(), data.encode_to_string().unwrap());
        
        let wrapper_value = NotationType::Json.encode_to_string(&wrapper).unwrap();
        wrapper_value
    }
}

impl UIFormatter for JsonLinesFormatter {
    fn title(&self, text: &str) -> String {
        self.json_wrap("title", text)
    }
    
    fn subtitle(&self, text: &str) -> String {
        self.json_wrap("subtitle", text)
    }
    
    fn heading(&self, level: u32, text: &str) -> String {
        let mut heading_data = HashMap::new();
        heading_data.insert("level".to_string(), level.to_string());
        heading_data.insert("text".to_string(), text.to_string());
        
        self.json_wrap("heading", heading_data)
    }
    
    fn text(&self, text: &str) -> String {
        self.json_wrap("text", text)
    }
    
    fn success(&self, text: &str) -> String {
        self.json_wrap("success", text)
    }
    
    fn error(&self, text: &str) -> String {
        self.json_wrap("error", text)
    }
    
    fn info(&self, text: &str) -> String {
        self.json_wrap("info", text)
    }
    
    fn warning(&self, text: &str) -> String {
        self.json_wrap("warning", text)
    }
    
    fn block(&self, text: &str) -> String {
        self.json_wrap("block", text)
    }
    
    fn code(&self, text: &str, language: Option<&str>) -> String {
        let mut code_data = HashMap::new();
        code_data.insert("text".to_string(), text.to_string());
        code_data.insert("language".to_string(), language.map(|l| l.to_string()).unwrap_or_default());
        
        self.json_wrap("code", code_data)
    }
    
    fn hr(&self) -> String {
        self.json_wrap("hr", NotationType::Null)
    }
    
    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        let list: Vec<String> = pairs.iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect();
        
        self.json_wrap("key_value_list", list.join("\n"))
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        let items_array: Vec<String> = items.iter().map(|item| item.to_string()).collect();
        
        let mut list_data = HashMap::new();
        list_data.insert("ordered".to_string(), ordered.to_string());
        list_data.insert("items".to_string(), items_array.join(","));
        
        self.json_wrap("list", list_data)
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let headers_array: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        
        let rows_array: Vec<String> = rows.iter()
            .map(|row| row.iter().map(|cell| cell.to_string()).collect::<Vec<String>>().join(","))
            .collect();
        
        let mut table_data = HashMap::new();
        table_data.insert("headers".to_string(), headers_array.join(","));
        table_data.insert("rows".to_string(), rows_array.join("\n"));
        
        self.json_wrap("table", table_data)
    }
    
    fn json_value(&self, value: &dyn Notation) -> Result<String, String> {
        Ok(self.json_wrap("json", value.clone()))
    }
    
    fn jsonl_value(&self, value: &dyn Notation) -> Result<String, String> {
        Ok(self.json_wrap("jsonl", value.clone()))
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
    
    fn json_value(&self, value: &dyn Notation) -> Result<String, String> {
        let json_str = value.encode_to_string()?;
        Ok(format!("```json\n{}\n```", json_str))
    }
    
    fn jsonl_value(&self, value: &dyn Notation) -> Result<String, String> {
        let json_str = value.encode_to_string()?;
        Ok(format!("```json\n{}\n```", json_str))
    }
} 