use anyhow::{anyhow, Result};
use crate::obj::notation::{Notation, NotationType, NumberVariant};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::writer::Writer;
use std::collections::HashMap;
use std::io::Cursor;
use serde::{Deserialize, Serialize};
use crate::fs::resources::error_messages;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct XmlNotation;

impl XmlNotation {
    pub fn new() -> Self {
        XmlNotation
    }
}

impl Default for XmlNotation {
    fn default() -> Self {
        Self::new()
    }
}

impl Notation for XmlNotation {
    fn encode<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        let xml_str = notation_type_to_xml_string(&notation_type)?;
        Ok(xml_str.into_bytes())
    }

    fn decode(&self, data: &[u8]) -> Result<NotationType> {
        let xml_str = String::from_utf8(data.to_vec())?;
        xml_string_to_notation_type(&xml_str)
    }

    fn encode_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Into<NotationType> + Clone,
    {
        let notation_type: NotationType = value.clone().into();
        notation_type_to_xml_string(&notation_type)
    }
}

fn notation_type_to_xml_string(notation_type: &NotationType) -> Result<String> {
    match notation_type {
        NotationType::Object(obj) => {
            let mut xml = String::new();
            xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<root>\n");
            for (key, value) in obj {
                xml.push_str(&format!("  <{}>{}</{}>\n", key, value, key));
            }
            xml.push_str("</root>");
            Ok(xml)
        }
        _ => Err(anyhow!("XML root must be an object")),
    }
}

fn xml_string_to_notation_type(xml_str: &str) -> Result<NotationType> {
    #[derive(Debug, serde::Deserialize)]
    struct XmlElement {
        #[serde(rename = "$value")]
        value: String,
    }

    #[derive(Debug, serde::Deserialize)]
    struct XmlRoot {
        #[serde(rename = "$value")]
        elements: Vec<XmlElement>,
    }

    let root: XmlRoot = from_str(xml_str)?;
    let mut map = std::collections::HashMap::new();
    
    for element in root.elements {
        let value_str = element.value;
        let value = if value_str == "true" {
            NotationType::Boolean(true)
        } else if value_str == "false" {
            NotationType::Boolean(false)
        } else if let Ok(i) = value_str.parse::<i64>() {
            NotationType::Number(NumberVariant::Int(i))
        } else if let Ok(f) = value_str.parse::<f64>() {
            NotationType::Number(NumberVariant::Float(f))
        } else if value_str == "null" {
            NotationType::Null
        } else {
            NotationType::String(value_str.clone())
        };
        
        map.insert(value_str, value);
    }

    Ok(NotationType::Object(map))
}

// Simplified XML Event to NotationType conversion
fn xml_event_to_notation_type(reader: &mut quick_xml::Reader<&[u8]>) -> Result<NotationType> {
    loop {
        match reader.read_event()?.into_owned() {
            Event::Start(_) => continue,
            Event::End(_) => continue,
            Event::Text(e) => {
                let text = e.unescape()?.to_string();
                if let Ok(i) = text.parse::<i64>() {
                    return Ok(NotationType::Number(NumberVariant::Int(i)))
                } else if let Ok(u) = text.parse::<u64>() {
                    return Ok(NotationType::Number(NumberVariant::Uint(u)))
                } else if let Ok(f) = text.parse::<f64>() {
                    return Ok(NotationType::Number(NumberVariant::Float(f)))
                } else {
                    return Ok(NotationType::String(text))
                }
            },
            Event::Eof => break,
            _ => continue,
        }
    }
    Err(anyhow!(error_messages::XML_PARSING_ERROR))
}

// Intermediate structure for simplified XML mapping
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum SimpleXml {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<SimpleXml>),
    Object(HashMap<String, SimpleXml>),
}

// Conversion from this intermediate structure to NotationType
fn simple_xml_to_notation_type(value: SimpleXml) -> NotationType {
    match value {
        SimpleXml::String(s) => NotationType::String(s),
        SimpleXml::Integer(i) => NotationType::Number(NumberVariant::Int(i)),
        SimpleXml::Float(f) => NotationType::Number(NumberVariant::Float(f)),
        SimpleXml::Boolean(b) => NotationType::Boolean(b),
        SimpleXml::Array(arr) => {
            NotationType::Array(arr.into_iter().map(simple_xml_to_notation_type).collect())
        }
        SimpleXml::Object(obj) => {
            NotationType::Object(obj.into_iter().map(|(k, v)| (k, simple_xml_to_notation_type(v))).collect())
        }
    }
} 