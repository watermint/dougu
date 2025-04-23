use std::fmt;

/// Represents errors that can occur in action processing
#[derive(Debug)]
pub enum ActionError {
    /// Error when an action fails to execute
    ExecutionFailed(String),
    /// Error when an action is not found
    NotFound(String),
    /// Error when an action has invalid parameters
    InvalidParameters(String),
    /// Error when an action execution is interrupted
    Interrupted,
    /// Other errors that don't fit the above categories
    Other(String),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::ExecutionFailed(msg) => write!(f, "Action execution failed: {}", msg),
            ActionError::NotFound(id) => write!(f, "Action not found: {}", id),
            ActionError::InvalidParameters(msg) => write!(f, "Invalid action parameters: {}", msg),
            ActionError::Interrupted => write!(f, "Action execution was interrupted"),
            ActionError::Other(msg) => write!(f, "Action error: {}", msg),
        }
    }
}

impl std::error::Error for ActionError {}

pub type ActionResult<T> = Result<T, ActionError>; 