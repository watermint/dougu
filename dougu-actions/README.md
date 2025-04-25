# Dougu Actions

This crate provides a flexible action system for the Dougu project. It allows you to define, register, and execute
actions with a clean, type-safe API.

## Features

- **Action Registry**: Register and manage actions through a central registry
- **Action Executor**: Execute actions with proper error handling and async support
- **Typed Interfaces**: Type-safe action definitions with input and output type parameters
- **Error Handling**: Comprehensive error types and results for robust error handling

## Usage

### Defining an Action

```rust
use dougu_actions::{Action, ActionError, ActionResult};

// Create a simple action
let action = Action::new("calculate", "Calculate Value")
.with_description("Performs a calculation on the input value");
```

### Setting up an Action Registry

```rust
use dougu_actions::ActionRegistry;

// Create a registry to hold actions
let registry = ActionRegistry::new();

// Register an action
registry.register(action)?;

// Retrieve an action
let retrieved_action = registry.get("calculate")?;
```

### Implementing an Executor

```rust
use dougu_actions::{ActionExecutor, FunctionExecutor};
use async_trait::async_trait;

// Create a function executor for simple use cases
let calc_executor = FunctionExecutor::new( | input: i32| -> ActionResult<i32> {
Ok(input * 2)
});

// Execute the action
let result = calc_executor.execute(21).await?;
assert_eq!(result, 42);

// Create a custom executor for more complex cases
struct CustomExecutor;

#[async_trait]
impl ActionExecutor for CustomExecutor {
    type Input = String;
    type Output = String;

    async fn execute(&self, input: Self::Input) -> ActionResult<Self::Output> {
        if input.is_empty() {
            return Err(ActionError::InvalidParameters("Input cannot be empty".to_string()));
        }
        Ok(format!("Processed: {}", input))
    }
}
```

## License

This project is part of the Dougu ecosystem and follows the same licensing terms. 