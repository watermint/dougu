use crate::obj::notation::{NotationType, NumberVariant};
use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::obj::resources::errors::*;

/// Query provides a wrapper around query operations on NotationType data.
/// It adds a layer of abstraction to provide a simple interface for common query operations.
pub struct Query {
    filter_str: String,
    compiled_filter: Filter,
    select: Option<String>,
    from: Option<String>,
    where_: Option<String>,
    group_by: Option<String>,
    having: Option<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

/// A compiled filter that can be executed against NotationType values
struct Filter {
    path: Vec<FilterStep>,
}

/// A single step in a filter path
enum FilterStep {
    Field(String),
    Index(usize),
    Wildcard,
}

#[derive(Error, Debug)]
enum QueryError {
    #[error("Invalid query path: {0}")]
    InvalidPath(String),
    #[error("Type mismatch at path {path}: expected {expected}, found {found}")]
    TypeMismatch { path: String, expected: String, found: String },
    #[error("Index out of bounds at path {path}: index {index}, size {size}")]
    IndexOutOfBounds { path: String, index: usize, size: usize },
    #[error("Field not found at path {path}: field '{field}'")]
    FieldNotFound { path: String, field: String },
}

impl Query {
    /// Compiles a query string into an executable filter.
    ///
    /// # Arguments
    ///
    /// * `query_str` - The query string to compile
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A compiled Query instance or an error
    pub fn compile(query_str: &str) -> Result<Self> {
        let path = Self::parse_path(query_str)?;

        Ok(Self {
            filter_str: query_str.to_string(),
            compiled_filter: Filter { path },
            select: None,
            from: None,
            where_: None,
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
            offset: None,
        })
    }

    /// Parses a query string into a path of filter steps
    fn parse_path(query_str: &str) -> Result<Vec<FilterStep>> {
        let mut path = Vec::new();
        let mut current = String::new();
        let mut in_brackets = false;

        // Query must start with a dot
        if !query_str.starts_with('.') {
            return Err(anyhow!("{}: Query must start with '.'", ERROR_QUERY_PARSE));
        }

        for c in query_str.chars() {
            match c {
                '.' if !in_brackets => {
                    if !current.is_empty() {
                        path.push(FilterStep::Field(current));
                        current = String::new();
                    }
                }
                '[' if !in_brackets => {
                    if !current.is_empty() {
                        path.push(FilterStep::Field(current));
                        current = String::new();
                    }
                    in_brackets = true;
                }
                ']' if in_brackets => {
                    if current == "*" {
                        path.push(FilterStep::Wildcard);
                    } else if current.is_empty() {
                        path.push(FilterStep::Wildcard);
                    } else if let Ok(index) = current.parse::<usize>() {
                        path.push(FilterStep::Index(index));
                    } else {
                        return Err(anyhow!("{}: Invalid array index: {}", ERROR_QUERY_PARSE, current));
                    }
                    current = String::new();
                    in_brackets = false;
                }
                _ => {
                    if !c.is_whitespace() {
                        // Only allow alphanumeric characters and underscores in field names
                        if !in_brackets && !c.is_alphanumeric() && c != '_' {
                            return Err(anyhow!("{}: Invalid character in field name: {}", ERROR_QUERY_PARSE, c));
                        }
                        current.push(c);
                    }
                }
            }
        }

        if !current.is_empty() {
            // Only allow field names outside brackets
            if in_brackets {
                return Err(anyhow!("{}: Invalid array index: {}", ERROR_QUERY_PARSE, current));
            }
            path.push(FilterStep::Field(current));
        }

        if path.is_empty() {
            return Err(anyhow!("{}: Empty query", ERROR_QUERY_PARSE));
        }

        if in_brackets {
            return Err(anyhow!("{}: Unclosed bracket", ERROR_QUERY_PARSE));
        }

        Ok(path)
    }

    /// Executes the compiled query against a NotationType value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against. Must be Clone + Into<NotationType>.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<NotationType>>` - A vector of NotationType results or an error
    pub fn execute<T>(&self, value: &T) -> Result<Vec<NotationType>>
    where
        T: Clone + Into<NotationType>,
    {
        let value = value.clone().into();
        let mut results = vec![value];

        for step in &self.compiled_filter.path {
            let mut next_results = Vec::new();

            for result in results {
                match step {
                    FilterStep::Field(field) => {
                        if let NotationType::Object(obj) = result {
                            if let Some(v) = obj.get(field) {
                                next_results.push(v.clone());
                            }
                        }
                    }
                    FilterStep::Index(index) => {
                        if let NotationType::Array(arr) = result {
                            if *index < arr.len() {
                                next_results.push(arr[*index].clone());
                            }
                        }
                    }
                    FilterStep::Wildcard => {
                        match result {
                            NotationType::Array(arr) => {
                                next_results.extend(arr.iter().cloned());
                            }
                            NotationType::Object(obj) => {
                                next_results.extend(obj.values().cloned());
                            }
                            _ => (),
                        }
                    }
                }
            }

            results = next_results;
            if results.is_empty() {
                break;
            }
        }

        Ok(results)
    }

    /// Executes the compiled query and returns a single NotationType result.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against. Must be Clone + Into<NotationType>.
    ///
    /// # Returns
    ///
    /// * `Result<NotationType>` - A single NotationType result or an error
    pub fn execute_to_single<T>(&self, value: &T) -> Result<NotationType>
    where
        T: Clone + Into<NotationType>,
    {
        let results = self.execute(value)?;

        if results.is_empty() {
            return Err(anyhow!(
                "{}: Query produced no results", 
                ERROR_QUERY_EXECUTION
            ));
        }

        Ok(results[0].clone())
    }

    /// Executes the compiled query and returns the result as a JSON string.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against. Must be Clone + Into<NotationType>.
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The JSON string result or an error
    pub fn execute_to_string<T>(&self, value: &T) -> Result<String>
    where
        T: Clone + Into<NotationType>,
    {
        let results = self.execute(value)?;

        if results.is_empty() {
            return Err(anyhow!(
                "{}: Query produced no results", 
                ERROR_QUERY_EXECUTION
            ));
        }

        match &results[0] {
            NotationType::String(s) => Ok(s.clone()),
            _ => Ok(results[0].to_string()),
        }
    }

    /// Returns the original query string used to compile this query.
    pub fn query_string(&self) -> &str {
        &self.filter_str
    }

    fn build_json_value(&self) -> NotationType {
        let mut obj = Vec::new();

        if let Some(select) = &self.select {
            obj.push(("select".to_string(), NotationType::String(select.clone())));
        }

        if let Some(from) = &self.from {
            obj.push(("from".to_string(), NotationType::String(from.clone())));
        }

        if let Some(where_) = &self.where_ {
            obj.push(("where".to_string(), NotationType::String(where_.clone())));
        }

        if let Some(group_by) = &self.group_by {
            obj.push(("group_by".to_string(), NotationType::String(group_by.clone())));
        }

        if let Some(having) = &self.having {
            obj.push(("having".to_string(), NotationType::String(having.clone())));
        }

        if let Some(order_by) = &self.order_by {
            obj.push(("order_by".to_string(), NotationType::String(order_by.clone())));
        }

        if let Some(limit) = self.limit {
            obj.push(("limit".to_string(), NotationType::Number(NumberVariant::Float(limit as f64))));
        }

        if let Some(offset) = self.offset {
            obj.push(("offset".to_string(), NotationType::Number(NumberVariant::Float(offset as f64))));
        }

        obj.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj::notation::{NotationType, NumberVariant};

    // Helper to create NotationType::Object from Vec<(String, NotationType)>
    fn create_object(data: Vec<(String, NotationType)>) -> NotationType {
        data.into()
    }

    // Helper to create NotationType::Array from Vec<NotationType>
    fn create_array(data: Vec<NotationType>) -> NotationType {
        NotationType::Array(data)
    }

    #[test]
    fn test_simple_query() {
        let data = create_object(vec![
            ("name".to_string(), "John Doe".into()),
            ("age".to_string(), 42.into()),
            ("address".to_string(), create_object(vec![
                ("street".to_string(), "123 Main St".into()),
                ("city".to_string(), "Anytown".into()),
            ])),
        ]);

        let query = Query::compile(".name").unwrap();
        let result = query.execute_to_string(&data).unwrap();
        // execute_to_string now uses fmt::Display for NotationType
        assert_eq!("John Doe", result);
    }

    #[test]
    fn test_array_query() {
        let data = create_object(vec![
            ("items".to_string(), create_array(vec![
                NotationType::Number(NumberVariant::Int(1)),
                NotationType::Number(NumberVariant::Int(2)),
                NotationType::Number(NumberVariant::Int(3)),
                NotationType::Number(NumberVariant::Int(4)),
                NotationType::Number(NumberVariant::Int(5))
            ]))
        ]);

        let query = Query::compile(".items[]").unwrap();
        let results = query.execute(&data).unwrap();
        assert_eq!(5, results.len());
        assert_eq!(NotationType::Number(NumberVariant::Int(1)), results[0]);
        assert_eq!(NotationType::Number(NumberVariant::Int(5)), results[4]);
    }

    #[test]
    fn test_filter_query() {
        let data = create_object(vec![
            ("people".to_string(), create_array(vec![
                create_object(vec![("name".to_string(), "Alice".into()), ("age".to_string(), NotationType::Number(NumberVariant::Int(25)))]),
                create_object(vec![("name".to_string(), "Bob".into()), ("age".to_string(), NotationType::Number(NumberVariant::Int(30)))]),
                create_object(vec![("name".to_string(), "Charlie".into()), ("age".to_string(), NotationType::Number(NumberVariant::Int(35)))]),
            ]))
        ]);

        // Access the last person in the array directly
        let query = Query::compile(".people[2]").unwrap();
        let results = query.execute(&data).unwrap();
        assert_eq!(1, results.len());
        // Access fields using helper methods or pattern matching
        assert_eq!(results[0].get("name").unwrap(), &NotationType::String("Charlie".to_string()));
        assert_eq!(results[0].get("age").unwrap(), &NotationType::Number(NumberVariant::Int(35)));
    }

    #[test]
    fn test_invalid_query() {
        let result = Query::compile("invalid query syntax");
        assert!(result.is_err());
    }
}
