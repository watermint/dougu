use anyhow::{Context, Result};
use std::str;

use crate::obj::resources::errors::*;
use super::{Notation, NotationType};

#[derive(Debug, Clone)]
pub struct JsonNotation;

impl Notation for JsonNotation {
    fn decode<T>(&self, input: &[u8]) -> Result<T>
    where
        T: From<NotationType>,
    {
        let s = str::from_utf8(input)
            .with_context(|| ERROR_DECODE_FAILED)?;
        
        let value = Self::parse_json(s)?;
        Ok(T::from(value))
    }
    
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let s = Self::format_json(&value)?;
        Ok(s.into_bytes())
    }
    
    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        Self::format_json(&value)
    }
}

impl JsonNotation {
    fn parse_json(s: &str) -> Result<NotationType> {
        let mut chars = s.chars().peekable();
        Self::parse_value(&mut chars)
    }
    
    fn parse_value<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        Self::skip_whitespace(chars);
        
        match chars.peek() {
            Some('"') => Self::parse_string(chars),
            Some('{') => Self::parse_object(chars),
            Some('[') => Self::parse_array(chars),
            Some('t') => Self::parse_true(chars),
            Some('f') => Self::parse_false(chars),
            Some('n') => Self::parse_null(chars),
            Some(c) if c.is_ascii_digit() || *c == '-' => Self::parse_number(chars),
            _ => Err(anyhow!("{}: Invalid JSON value", ERROR_DECODE_FAILED)),
        }
    }
    
    fn parse_string<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        chars.next(); // Skip opening quote
        let mut s = String::new();
        let mut escape = false;
        
        while let Some(c) = chars.next() {
            if escape {
                match c {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    _ => s.push(c),
                }
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                return Ok(NotationType::String(s));
            } else {
                s.push(c);
            }
        }
        
        Err(anyhow!("{}: Unterminated string", ERROR_DECODE_FAILED))
    }
    
    fn parse_object<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        chars.next(); // Skip opening brace
        let mut obj = Vec::new();
        
        loop {
            Self::skip_whitespace(chars);
            
            if let Some('}') = chars.peek() {
                chars.next();
                return Ok(NotationType::Object(obj));
            }
            
            if !obj.is_empty() {
                Self::expect_char(chars, ',')?;
                Self::skip_whitespace(chars);
            }
            
            let key = if let NotationType::String(s) = Self::parse_string(chars)? {
                s
            } else {
                return Err(anyhow!("{}: Object key must be a string", ERROR_DECODE_FAILED));
            };
            
            Self::skip_whitespace(chars);
            Self::expect_char(chars, ':')?;
            Self::skip_whitespace(chars);
            
            let value = Self::parse_value(chars)?;
            obj.push((key, value));
        }
    }
    
    fn parse_array<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        chars.next(); // Skip opening bracket
        let mut arr = Vec::new();
        
        loop {
            Self::skip_whitespace(chars);
            
            if let Some(']') = chars.peek() {
                chars.next();
                return Ok(NotationType::Array(arr));
            }
            
            if !arr.is_empty() {
                Self::expect_char(chars, ',')?;
                Self::skip_whitespace(chars);
            }
            
            let value = Self::parse_value(chars)?;
            arr.push(value);
        }
    }
    
    fn parse_true<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        Self::expect_str(chars, "true")?;
        Ok(NotationType::Boolean(true))
    }
    
    fn parse_false<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        Self::expect_str(chars, "false")?;
        Ok(NotationType::Boolean(false))
    }
    
    fn parse_null<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        Self::expect_str(chars, "null")?;
        Ok(NotationType::Null)
    }
    
    fn parse_number<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Result<NotationType> {
        let mut s = String::new();
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        if let Some('-') = chars.peek() {
            s.push(chars.next().unwrap());
        }
        
        while let Some(c) = chars.peek() {
            match c {
                '0'..='9' => {
                    s.push(chars.next().unwrap());
                },
                '.' if !has_decimal && !has_exponent => {
                    s.push(chars.next().unwrap());
                    has_decimal = true;
                },
                'e' | 'E' if !has_exponent => {
                    s.push(chars.next().unwrap());
                    has_exponent = true;
                    
                    if let Some('+' | '-') = chars.peek() {
                        s.push(chars.next().unwrap());
                    }
                },
                _ => break,
            }
        }
        
        s.parse::<f64>()
            .map(NotationType::Number)
            .map_err(|_| anyhow!("{}: Invalid number: {}", ERROR_DECODE_FAILED, s))
    }
    
    fn skip_whitespace<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) {
        while let Some(c) = chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            chars.next();
        }
    }
    
    fn expect_char<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>, expected: char) -> Result<()> {
        match chars.next() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(anyhow!("{}: Expected '{}', found '{}'", ERROR_DECODE_FAILED, expected, c)),
            None => Err(anyhow!("{}: Expected '{}', found end of input", ERROR_DECODE_FAILED, expected)),
        }
    }
    
    fn expect_str<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>, expected: &str) -> Result<()> {
        for c in expected.chars() {
            Self::expect_char(chars, c)?;
        }
        Ok(())
    }
    
    fn format_json(value: &NotationType) -> Result<String> {
        let mut s = String::new();
        Self::format_value(value, &mut s)?;
        Ok(s)
    }
    
    fn format_value(value: &NotationType, s: &mut String) -> Result<()> {
        match value {
            NotationType::String(str) => {
                s.push('"');
                for c in str.chars() {
                    match c {
                        '\n' => s.push_str("\\n"),
                        '\r' => s.push_str("\\r"),
                        '\t' => s.push_str("\\t"),
                        '\\' => s.push_str("\\\\"),
                        '"' => s.push_str("\\\""),
                        _ => s.push(c),
                    }
                }
                s.push('"');
            },
            NotationType::Number(n) => {
                s.push_str(&n.to_string());
            },
            NotationType::Boolean(b) => {
                s.push_str(if *b { "true" } else { "false" });
            },
            NotationType::Null => {
                s.push_str("null");
            },
            NotationType::Array(arr) => {
                s.push('[');
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    Self::format_value(item, s)?;
                }
                s.push(']');
            },
            NotationType::Object(obj) => {
                s.push('{');
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        s.push(',');
                    }
                    Self::format_value(&NotationType::String(k.clone()), s)?;
                    s.push(':');
                    Self::format_value(v, s)?;
                }
                s.push('}');
            },
        }
        Ok(())
    }
} 