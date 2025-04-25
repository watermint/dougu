use std::fmt;

/// Case represents different text casing formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    /// Camel case: "camelCase"
    Camel,
    /// Pascal case: "PascalCase"
    Pascal,
    /// Snake case: "snake_case"
    Snake,
    /// Screaming snake case: "SCREAMING_SNAKE_CASE"
    ScreamingSnake,
    /// Kebab case: "kebab-case"
    Kebab,
    /// Delimited case: "custom.delimited.case"
    Delimited(char),
}

impl fmt::Display for Case {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Case::Camel => write!(f, "camel"),
            Case::Pascal => write!(f, "pascal"),
            Case::Snake => write!(f, "snake"),
            Case::ScreamingSnake => write!(f, "screaming_snake"),
            Case::Kebab => write!(f, "kebab"),
            Case::Delimited(c) => write!(f, "delimited({})", c),
        }
    }
}

/// WordExtractor splits text into word components.
struct WordExtractor<'a> {
    text: &'a str,
    current_case: Option<Case>,
}

impl<'a> WordExtractor<'a> {
    /// Create a new WordExtractor
    fn new(text: &'a str) -> Self {
        let current_case = detect_case(text);
        Self { text, current_case }
    }

    /// Extract individual words from the text based on case patterns
    fn extract_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        
        match self.current_case {
            Some(Case::Camel) | Some(Case::Pascal) => {
                let mut current_word = String::new();
                let mut prev_is_upper = false;
                let mut consecutive_uppers = 0;
                let chars: Vec<char> = self.text.chars().collect();

                for i in 0..chars.len() {
                    let c = chars[i];
                    
                    if i == 0 {
                        current_word.push(c.to_lowercase().next().unwrap_or(c));
                        prev_is_upper = c.is_uppercase();
                        if prev_is_upper {
                            consecutive_uppers = 1;
                        }
                        continue;
                    }

                    if c.is_uppercase() {
                        // Handle acronyms (consecutive uppercase letters)
                        if prev_is_upper {
                            consecutive_uppers += 1;
                            
                            // If this is the last uppercase in a sequence before a lowercase,
                            // it belongs to the next word
                            let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
                            if next_is_lower && consecutive_uppers > 1 {
                                // End the current word (acronym)
                                if !current_word.is_empty() {
                                    words.push(current_word);
                                    current_word = String::new();
                                }
                            }
                        } else {
                            // Regular camel/pascal case boundary
                            if !current_word.is_empty() {
                                words.push(current_word);
                                current_word = String::new();
                            }
                            consecutive_uppers = 1;
                        }
                        current_word.push(c.to_lowercase().next().unwrap_or(c));
                        prev_is_upper = true;
                    } else {
                        current_word.push(c);
                        prev_is_upper = false;
                        consecutive_uppers = 0;
                    }
                }

                if !current_word.is_empty() {
                    words.push(current_word);
                }
            },
            Some(Case::Snake) | Some(Case::ScreamingSnake) => {
                for part in self.text.split('_') {
                    if !part.is_empty() {
                        words.push(part.to_lowercase());
                    }
                }
            },
            Some(Case::Kebab) => {
                for part in self.text.split('-') {
                    if !part.is_empty() {
                        words.push(part.to_lowercase());
                    }
                }
            },
            Some(Case::Delimited(delimiter)) => {
                for part in self.text.split(delimiter) {
                    if !part.is_empty() {
                        words.push(part.to_lowercase());
                    }
                }
            },
            None => {
                // If no specific case detected, just use the text as is
                if !self.text.is_empty() {
                    words.push(self.text.to_lowercase());
                }
            }
        }

        words
    }
}

/// Detect the case of the given text
fn detect_case(text: &str) -> Option<Case> {
    if text.is_empty() {
        return None;
    }

    if text.contains('_') {
        if text.to_uppercase() == text {
            return Some(Case::ScreamingSnake);
        } else {
            return Some(Case::Snake);
        }
    }

    if text.contains('-') {
        return Some(Case::Kebab);
    }

    // Check for period as a delimiter
    if text.contains('.') {
        return Some(Case::Delimited('.'));
    }

    // Check for camelCase vs PascalCase
    let first_char = text.chars().next().unwrap();
    if text.chars().any(|c| c.is_uppercase()) {
        if first_char.is_uppercase() {
            return Some(Case::Pascal);
        } else {
            return Some(Case::Camel);
        }
    }

    // If no specific case detected
    None
}

/// CaseConverter converts text between different case formats
pub struct CaseConverter;

impl CaseConverter {
    /// Convert text to a specified case format
    pub fn convert(text: &str, to_case: Case) -> String {
        let extractor = WordExtractor::new(text);
        let words = extractor.extract_words();
        
        if words.is_empty() {
            return String::new();
        }

        match to_case {
            Case::Camel => {
                let mut result = words[0].clone();
                for word in words.iter().skip(1) {
                    if let Some(first_char) = word.chars().next() {
                        result.push_str(&first_char.to_uppercase().to_string());
                        result.push_str(&word[first_char.len_utf8()..]);
                    }
                }
                result
            },
            Case::Pascal => {
                let mut result = String::new();
                for word in &words {
                    if let Some(first_char) = word.chars().next() {
                        result.push_str(&first_char.to_uppercase().to_string());
                        result.push_str(&word[first_char.len_utf8()..]);
                    }
                }
                result
            },
            Case::Snake => words.join("_"),
            Case::ScreamingSnake => {
                words.iter()
                    .map(|w| w.to_uppercase())
                    .collect::<Vec<_>>()
                    .join("_")
            },
            Case::Kebab => words.join("-"),
            Case::Delimited(delimiter) => words.join(&delimiter.to_string()),
        }
    }

    /// Detect case of a string
    pub fn detect(text: &str) -> Option<Case> {
        detect_case(text)
    }
}

/// Trait providing case conversion methods for strings
pub trait CaseExt {
    /// Convert to camel case
    fn to_camel_case(&self) -> String;
    
    /// Convert to pascal case
    fn to_pascal_case(&self) -> String;
    
    /// Convert to snake case
    fn to_snake_case(&self) -> String;
    
    /// Convert to screaming snake case
    fn to_screaming_snake_case(&self) -> String;
    
    /// Convert to kebab case
    fn to_kebab_case(&self) -> String;
    
    /// Convert to delimited case with the provided delimiter
    fn to_delimited_case(&self, delimiter: char) -> String;
    
    /// Detect case of the string
    fn detect_case(&self) -> Option<Case>;
}

impl CaseExt for str {
    fn to_camel_case(&self) -> String {
        CaseConverter::convert(self, Case::Camel)
    }
    
    fn to_pascal_case(&self) -> String {
        CaseConverter::convert(self, Case::Pascal)
    }
    
    fn to_snake_case(&self) -> String {
        CaseConverter::convert(self, Case::Snake)
    }
    
    fn to_screaming_snake_case(&self) -> String {
        CaseConverter::convert(self, Case::ScreamingSnake)
    }
    
    fn to_kebab_case(&self) -> String {
        CaseConverter::convert(self, Case::Kebab)
    }
    
    fn to_delimited_case(&self, delimiter: char) -> String {
        CaseConverter::convert(self, Case::Delimited(delimiter))
    }
    
    fn detect_case(&self) -> Option<Case> {
        CaseConverter::detect(self)
    }
}

impl CaseExt for String {
    fn to_camel_case(&self) -> String {
        self.as_str().to_camel_case()
    }
    
    fn to_pascal_case(&self) -> String {
        self.as_str().to_pascal_case()
    }
    
    fn to_snake_case(&self) -> String {
        self.as_str().to_snake_case()
    }
    
    fn to_screaming_snake_case(&self) -> String {
        self.as_str().to_screaming_snake_case()
    }
    
    fn to_kebab_case(&self) -> String {
        self.as_str().to_kebab_case()
    }
    
    fn to_delimited_case(&self, delimiter: char) -> String {
        self.as_str().to_delimited_case(delimiter)
    }
    
    fn detect_case(&self) -> Option<Case> {
        self.as_str().detect_case()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_case() {
        assert_eq!(CaseConverter::detect("camelCase"), Some(Case::Camel));
        assert_eq!(CaseConverter::detect("PascalCase"), Some(Case::Pascal));
        assert_eq!(CaseConverter::detect("snake_case"), Some(Case::Snake));
        assert_eq!(CaseConverter::detect("SCREAMING_SNAKE_CASE"), Some(Case::ScreamingSnake));
        assert_eq!(CaseConverter::detect("kebab-case"), Some(Case::Kebab));
        assert_eq!(CaseConverter::detect("delimited.case"), Some(Case::Delimited('.')));
        assert_eq!(CaseConverter::detect("plaintext"), None);
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!("camel_case".to_camel_case(), "camelCase");
        assert_eq!("PascalCase".to_camel_case(), "pascalCase");
        assert_eq!("kebab-case".to_camel_case(), "kebabCase");
        assert_eq!("SCREAMING_SNAKE_CASE".to_camel_case(), "screamingSnakeCase");
        assert_eq!("delimited.case".to_camel_case(), "delimitedCase");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!("camel_case".to_pascal_case(), "CamelCase");
        assert_eq!("camelCase".to_pascal_case(), "CamelCase");
        assert_eq!("kebab-case".to_pascal_case(), "KebabCase");
        assert_eq!("SCREAMING_SNAKE_CASE".to_pascal_case(), "ScreamingSnakeCase");
        assert_eq!("delimited.case".to_pascal_case(), "DelimitedCase");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!("camelCase".to_snake_case(), "camel_case");
        assert_eq!("PascalCase".to_snake_case(), "pascal_case");
        assert_eq!("kebab-case".to_snake_case(), "kebab_case");
        assert_eq!("SCREAMING_SNAKE_CASE".to_snake_case(), "screaming_snake_case");
        assert_eq!("delimited.case".to_snake_case(), "delimited_case");
    }

    #[test]
    fn test_to_screaming_snake_case() {
        assert_eq!("camelCase".to_screaming_snake_case(), "CAMEL_CASE");
        assert_eq!("PascalCase".to_screaming_snake_case(), "PASCAL_CASE");
        assert_eq!("kebab-case".to_screaming_snake_case(), "KEBAB_CASE");
        assert_eq!("snake_case".to_screaming_snake_case(), "SNAKE_CASE");
        assert_eq!("delimited.case".to_screaming_snake_case(), "DELIMITED_CASE");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!("camelCase".to_kebab_case(), "camel-case");
        assert_eq!("PascalCase".to_kebab_case(), "pascal-case");
        assert_eq!("snake_case".to_kebab_case(), "snake-case");
        assert_eq!("SCREAMING_SNAKE_CASE".to_kebab_case(), "screaming-snake-case");
        assert_eq!("delimited.case".to_kebab_case(), "delimited-case");
    }

    #[test]
    fn test_to_delimited_case() {
        assert_eq!("camelCase".to_delimited_case('.'), "camel.case");
        assert_eq!("PascalCase".to_delimited_case('.'), "pascal.case");
        assert_eq!("snake_case".to_delimited_case('.'), "snake.case");
        assert_eq!("SCREAMING_SNAKE_CASE".to_delimited_case(':'), "screaming:snake:case");
        assert_eq!("kebab-case".to_delimited_case('/'), "kebab/case");
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!("".to_camel_case(), "");
        assert_eq!("".to_pascal_case(), "");
        assert_eq!("".to_snake_case(), "");
        assert_eq!("".to_kebab_case(), "");
        
        // Single word
        assert_eq!("word".to_camel_case(), "word");
        assert_eq!("word".to_pascal_case(), "Word");
        assert_eq!("word".to_snake_case(), "word");
        assert_eq!("word".to_kebab_case(), "word");
        
        // Multi-word with consecutive uppercase letters
        assert_eq!("JSONParser".to_camel_case(), "jsonParser");
        assert_eq!("JSONParser".to_snake_case(), "json_parser");
        assert_eq!("HTTPRequest".to_kebab_case(), "http-request");
    }
} 