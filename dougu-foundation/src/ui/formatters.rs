use crate::ui::{UIFormatter, UITheme};
use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use serde_json;

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
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string_pretty(value).map_err(|e| e.to_string())
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        serde_json::to_string(value).map_err(|e| e.to_string())
    }
}

pub struct JsonLinesFormatter;

impl JsonLinesFormatter {
    pub fn new() -> Self {
        Self {}
    }
    
    fn json_wrap(&self, json_type: &str, data: serde_json::Value) -> String {
        let mut wrapper = serde_json::Map::new();
        wrapper.insert("type".to_string(), serde_json::Value::String(json_type.to_string()));
        wrapper.insert("data".to_string(), data);
        
        let wrapper_value = serde_json::Value::Object(wrapper);
        serde_json::to_string(&wrapper_value)
            .unwrap_or_else(|_| "{}".to_string())
    }
}

impl UIFormatter for JsonLinesFormatter {
    fn title(&self, text: &str) -> String {
        self.json_wrap("title", serde_json::Value::String(text.to_string()))
    }
    
    fn subtitle(&self, text: &str) -> String {
        self.json_wrap("subtitle", serde_json::Value::String(text.to_string()))
    }
    
    fn heading(&self, level: u8, text: &str) -> String {
        let mut heading_data = serde_json::Map::new();
        heading_data.insert("level".to_string(), serde_json::Value::Number(serde_json::Number::from(level)));
        heading_data.insert("text".to_string(), serde_json::Value::String(text.to_string()));
        
        self.json_wrap("heading", serde_json::Value::Object(heading_data))
    }
    
    fn text(&self, text: &str) -> String {
        self.json_wrap("text", serde_json::Value::String(text.to_string()))
    }
    
    fn success(&self, text: &str) -> String {
        self.json_wrap("success", serde_json::Value::String(text.to_string()))
    }
    
    fn error(&self, text: &str) -> String {
        self.json_wrap("error", serde_json::Value::String(text.to_string()))
    }
    
    fn info(&self, text: &str) -> String {
        self.json_wrap("info", serde_json::Value::String(text.to_string()))
    }
    
    fn warning(&self, text: &str) -> String {
        self.json_wrap("warning", serde_json::Value::String(text.to_string()))
    }
    
    fn block(&self, text: &str) -> String {
        self.json_wrap("block", serde_json::Value::String(text.to_string()))
    }
    
    fn code(&self, text: &str, language: Option<&str>) -> String {
        let mut code_data = serde_json::Map::new();
        code_data.insert("text".to_string(), serde_json::Value::String(text.to_string()));
        code_data.insert("language".to_string(), match language {
            Some(lang) => serde_json::Value::String(lang.to_string()),
            None => serde_json::Value::Null,
        });
        
        self.json_wrap("code", serde_json::Value::Object(code_data))
    }
    
    fn hr(&self) -> String {
        self.json_wrap("hr", serde_json::Value::Null)
    }
    
    fn key_value_list(&self, pairs: &[(&str, &str)]) -> String {
        let list: Vec<serde_json::Value> = pairs
            .iter()
            .map(|(key, value)| {
                let mut item = serde_json::Map::new();
                item.insert("key".to_string(), serde_json::Value::String(key.to_string()));
                item.insert("value".to_string(), serde_json::Value::String(value.to_string()));
                serde_json::Value::Object(item)
            })
            .collect();
        
        self.json_wrap("key_value_list", serde_json::Value::Array(list))
    }
    
    fn list_string(&self, items: &[String], ordered: bool) -> String {
        let items_array: Vec<serde_json::Value> = items
            .iter()
            .map(|item| serde_json::Value::String(item.to_string()))
            .collect();
        
        let mut list_data = serde_json::Map::new();
        list_data.insert("ordered".to_string(), serde_json::Value::Bool(ordered));
        list_data.insert("items".to_string(), serde_json::Value::Array(items_array));
        
        self.json_wrap("list", serde_json::Value::Object(list_data))
    }
    
    fn table_string(&self, headers: &[&str], rows: &[Vec<String>]) -> String {
        let headers_array: Vec<serde_json::Value> = headers
            .iter()
            .map(|h| serde_json::Value::String(h.to_string()))
            .collect();
        
        let rows_array: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let row_array: Vec<serde_json::Value> = row
                    .iter()
                    .map(|cell| serde_json::Value::String(cell.to_string()))
                    .collect();
                serde_json::Value::Array(row_array)
            })
            .collect();
        
        let mut table_data = serde_json::Map::new();
        table_data.insert("headers".to_string(), serde_json::Value::Array(headers_array));
        table_data.insert("rows".to_string(), serde_json::Value::Array(rows_array));
        
        self.json_wrap("table", serde_json::Value::Object(table_data))
    }
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        self.json_wrap("json", value.clone());
        Ok(serde_json::to_string(value).map_err(|e| e.to_string())?)
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        Ok(serde_json::to_string(value).map_err(|e| e.to_string())?)
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
    
    fn json_value(&self, value: &serde_json::Value) -> Result<String, String> {
        let json_str = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
        Ok(format!("```json\n{}\n```", json_str))
    }
    
    fn jsonl_value(&self, value: &serde_json::Value) -> Result<String, String> {
        let json_str = serde_json::to_string(value).map_err(|e| e.to_string())?;
        Ok(format!("```json\n{}\n```", json_str))
    }
} 