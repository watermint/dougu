use anyhow::{anyhow, Context, Result};
use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
use jaq_parse;
use serde::Serialize;
use serde_json::Value;

use crate::obj::resources::errors::*;

/// JaqQuery provides a wrapper around the jaq-parse library for executing JQ-like queries on JSON data.
/// It adds a layer of abstraction to hide the complexity of the underlying library and provide
/// a simple interface for common query operations.
pub struct Query {
    filter_str: String,
    compiled_filter: jaq_interpret::Filter,
}

impl Query {
    /// Compiles a JQ query string into an executable filter.
    ///
    /// # Arguments
    ///
    /// * `query_str` - The JQ query string to compile
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A compiled JaqQuery instance or an error
    pub fn compile(query_str: &str) -> Result<Self> {
        // Parse the filter using jaq_parse
        let (parsed_filter, parse_errors) = jaq_parse::parse(query_str, jaq_parse::main());
        
        if !parse_errors.is_empty() {
            return Err(anyhow!(
                "{}: Failed to parse query: {} errors occurred", 
                ERROR_QUERY_PARSE, 
                parse_errors.len()
            ));
        }
        
        let parsed_filter = parsed_filter.ok_or_else(|| anyhow!("{}: No filter parsed", ERROR_QUERY_PARSE))?;
        
        // Compile the filter in the context
        let mut ctx = ParseCtx::new(Vec::new());
        let compiled_filter = ctx.compile(parsed_filter);
        
        if !ctx.errs.is_empty() {
            return Err(anyhow!(
                "{}: Failed to compile query: {} errors occurred", 
                ERROR_QUERY_PARSE, 
                ctx.errs.len()
            ));
        }
        
        Ok(Self {
            filter_str: query_str.to_string(),
            compiled_filter,
        })
    }
    
    /// Executes the compiled query against a JSON-serializable value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Value>>` - A vector of resulting JSON values or an error
    pub fn execute<T>(&self, value: &T) -> Result<Vec<Value>> 
    where
        T: Serialize,
    {
        // Convert the input value to serde_json Value
        let json_value = serde_json::to_value(value)
            .with_context(|| ERROR_VALUE_CONVERSION)?;
        
        // Set up the execution context
        let inputs = RcIter::new(std::iter::empty());
        let ctx = Ctx::new([], &inputs);
        
        // Run the filter
        let results: Vec<Result<Val, jaq_interpret::Error>> = 
            self.compiled_filter
                .run((ctx, Val::from(json_value)))
                .collect();
        
        // Convert the results to serde_json Values
        let mut output = Vec::new();
        for result in results {
            match result {
                Ok(val) => {
                    let value: Value = val.into();
                    output.push(value);
                },
                Err(err) => {
                    return Err(anyhow!(
                        "{}: Failed to execute query: {}", 
                        ERROR_QUERY_EXECUTION, 
                        err
                    ));
                }
            }
        }
        
        Ok(output)
    }
    
    /// Executes the compiled query and returns a single JSON value result.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to query against
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - A single JSON value result or an error
    pub fn execute_to_single<T>(&self, value: &T) -> Result<Value> 
    where
        T: Serialize,
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
        T: Serialize,
    {
        let results = self.execute(value)?;
        
        if results.is_empty() {
            return Err(anyhow!(
                "{}: Query produced no results", 
                ERROR_QUERY_EXECUTION
            ));
        }
        
        serde_json::to_string(&results[0])
            .with_context(|| ERROR_VALUE_CONVERSION)
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
