use anyhow::{anyhow, Context, Result};
use std::str;

use crate::obj::resources::errors::*;
use crate::obj::notation::NotationType;

/// Query provides a wrapper around query operations on NotationType data.
/// It adds a layer of abstraction to provide a simple interface for common query operations.
pub struct Query {
    filter_str: String,
    compiled_filter: Filter,
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
        })
    }
    
    /// Parses a query string into a path of filter steps
    fn parse_path(query_str: &str) -> Result<Vec<FilterStep>> {
        let mut path = Vec::new();
        let mut current = String::new();
        let mut in_brackets = false;
        
        for c in query_str.chars() {
            match c {
                '.' if !in_brackets => {
                    if !current.is_empty() {
                        path.push(FilterStep::Field(current));
                        current = String::new();
                    }
                },
                '[' if !in_brackets => {
                    if !current.is_empty() {
                        path.push(FilterStep::Field(current));
                        current = String::new();
                    }
                    in_brackets = true;
                },
                ']' if in_brackets => {
                    if current == "*" {
                        path.push(FilterStep::Wildcard);
                    } else if let Ok(index) = current.parse::<usize>() {
                        path.push(FilterStep::Index(index));
                    } else {
                        return Err(anyhow!("{}: Invalid array index: {}", ERROR_QUERY_PARSE, current));
                    }
                    current = String::new();
                    in_brackets = false;
                },
                _ => current.push(c),
            }
        }
        
        if !current.is_empty() {
            path.push(FilterStep::Field(current));
        }
        
        if path.is_empty() {
            return Err(anyhow!("{}: Empty query", ERROR_QUERY_PARSE));
        }
        
        Ok(path)
    }
    
    /// Executes the compiled query against a NotationType value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against
    ///
    /// # Returns
    ///
    /// * `Result<Vec<NotationType>>` - A vector of resulting values or an error
    pub fn execute<T>(&self, value: &T) -> Result<Vec<NotationType>> 
    where
        T: Into<NotationType>,
    {
        let value = value.into();
        let mut results = vec![value];
        
        for step in &self.compiled_filter.path {
            let mut next_results = Vec::new();
            
            for result in results {
                match step {
                    FilterStep::Field(field) => {
                        if let NotationType::Object(obj) = result {
                            for (k, v) in obj {
                                if k == field {
                                    next_results.push(v.clone());
                                }
                            }
                        }
                    },
                    FilterStep::Index(index) => {
                        if let NotationType::Array(arr) = result {
                            if *index < arr.len() {
                                next_results.push(arr[*index].clone());
                            }
                        }
                    },
                    FilterStep::Wildcard => {
                        match result {
                            NotationType::Array(arr) => {
                                next_results.extend(arr.iter().cloned());
                            },
                            NotationType::Object(obj) => {
                                next_results.extend(obj.iter().map(|(_, v)| v.clone()));
                            },
                            _ => (),
                        }
                    },
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
    /// * `value` - The value to query against
    ///
    /// # Returns
    ///
    /// * `Result<NotationType>` - A single NotationType result or an error
    pub fn execute_to_single<T>(&self, value: &T) -> Result<NotationType> 
    where
        T: Into<NotationType>,
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
    /// * `value` - The value to query against
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The JSON string result or an error
    pub fn execute_to_string<T>(&self, value: &T) -> Result<String> 
    where
        T: Into<NotationType>,
    {
        let results = self.execute(value)?;
        
        if results.is_empty() {
            return Err(anyhow!(
                "{}: Query produced no results", 
                ERROR_QUERY_EXECUTION
            ));
        }
        
        Ok(results[0].to_string())
    }
    
    /// Returns the original query string used to compile this query.
    pub fn query_string(&self) -> &str {
        &self.filter_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_simple_query() {
        let data = json!({
            "name": "John Doe",
            "age": 42,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            }
        });
        
        let query = Query::compile(".name").unwrap();
        let result = query.execute_to_string(&data).unwrap();
        assert_eq!("\"John Doe\"", result);
    }
    
    #[test]
    fn test_array_query() {
        let data = json!({
            "items": [1, 2, 3, 4, 5]
        });
        
        let query = Query::compile(".items[]").unwrap();
        let results = query.execute(&data).unwrap();
        assert_eq!(5, results.len());
        assert_eq!(json!(1), results[0]);
        assert_eq!(json!(5), results[4]);
    }
    
    #[test]
    fn test_filter_query() {
        let data = json!({
            "people": [
                {"name": "Alice", "age": 25},
                {"name": "Bob", "age": 30},
                {"name": "Charlie", "age": 35}
            ]
        });
        
        // Access the last person in the array directly
        let query = Query::compile(".people[2]").unwrap();
        let results = query.execute(&data).unwrap();
        assert_eq!(1, results.len());
        assert_eq!("Charlie", results[0]["name"].as_str().unwrap());
        assert_eq!(35, results[0]["age"].as_u64().unwrap());
    }
    
    #[test]
    fn test_invalid_query() {
        let result = Query::compile("invalid query syntax");
        assert!(result.is_err());
    }
}
