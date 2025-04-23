use std::fmt;

mod error;
mod executor;
mod registry;

pub use error::{ActionError, ActionResult};
pub use executor::{ActionExecutor, FunctionExecutor};
pub use registry::ActionRegistry;

/// Represents an action that can be performed within the Dougu system
#[derive(Debug, Clone)]
pub struct Action {
    id: String,
    name: String,
    description: Option<String>,
}

impl Action {
    /// Creates a new action with the specified ID and name
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
        }
    }

    /// Adds a description to the action
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns the ID of the action
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the name of the action
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description of the action, if available
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let action = Action::new("test_id", "Test Action")
            .with_description("This is a test action");
        
        assert_eq!(action.id(), "test_id");
        assert_eq!(action.name(), "Test Action");
        assert_eq!(action.description(), Some("This is a test action"));
    }

    #[test]
    fn test_action_registry() {
        let registry = ActionRegistry::new();
        let action = Action::new("test_action", "Test Action")
            .with_description("Test action description");
        
        // Register action
        assert!(registry.register(action.clone()).is_ok());
        
        // Check contains
        assert!(registry.contains("test_action").unwrap());
        assert!(!registry.contains("non_existent").unwrap());
        
        // Get action
        let retrieved = registry.get("test_action").unwrap();
        assert_eq!(retrieved.id(), "test_action");
        assert_eq!(retrieved.name(), "Test Action");
        
        // All actions
        let all_actions = registry.all().unwrap();
        assert_eq!(all_actions.len(), 1);
        
        // Unregister
        assert!(registry.unregister("test_action").is_ok());
        assert!(!registry.contains("test_action").unwrap());
    }

    #[tokio::test]
    async fn test_function_executor() {
        // Define a simple function to execute
        let executor = FunctionExecutor::new(|input: i32| -> ActionResult<String> {
            Ok(format!("Result: {}", input * 2))
        });
        
        // Execute the function
        let result = executor.execute(21).await.unwrap();
        assert_eq!(result, "Result: 42");
        
        // Test error case
        let error_executor = FunctionExecutor::new(|_: i32| -> ActionResult<String> {
            Err(error::ActionError::ExecutionFailed("Test error".to_string()))
        });
        
        assert!(error_executor.execute(0).await.is_err());
    }
} 