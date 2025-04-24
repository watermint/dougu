use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{ActionError, ActionResult};
use crate::Action;

/// A registry for storing and retrieving actions
#[derive(Default)]
pub struct ActionRegistry {
    actions: RwLock<HashMap<String, Arc<Action>>>,
}

impl ActionRegistry {
    /// Create a new empty action registry
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
        }
    }

    /// Register an action in the registry
    pub fn register(&self, action: Action) -> ActionResult<()> {
        let action_id = action.id().to_string();
        let mut actions = self.actions.write().map_err(|_| {
            ActionError::Other("Failed to acquire write lock on action registry".to_string())
        })?;
        
        actions.insert(action_id, Arc::new(action));
        Ok(())
    }

    /// Get an action from the registry by ID
    pub fn get(&self, id: &str) -> ActionResult<Arc<Action>> {
        let actions = self.actions.read().map_err(|_| {
            ActionError::Other("Failed to acquire read lock on action registry".to_string())
        })?;
        
        actions.get(id)
            .cloned()
            .ok_or_else(|| ActionError::NotFound(id.to_string()))
    }

    /// Check if an action exists in the registry
    pub fn contains(&self, id: &str) -> ActionResult<bool> {
        let actions = self.actions.read().map_err(|_| {
            ActionError::Other("Failed to acquire read lock on action registry".to_string())
        })?;
        
        Ok(actions.contains_key(id))
    }

    /// Remove an action from the registry
    pub fn unregister(&self, id: &str) -> ActionResult<()> {
        let mut actions = self.actions.write().map_err(|_| {
            ActionError::Other("Failed to acquire write lock on action registry".to_string())
        })?;
        
        if actions.remove(id).is_none() {
            return Err(ActionError::NotFound(id.to_string()));
        }
        
        Ok(())
    }

    /// Get all registered actions
    pub fn all(&self) -> ActionResult<Vec<Arc<Action>>> {
        let actions = self.actions.read().map_err(|_| {
            ActionError::Other("Failed to acquire read lock on action registry".to_string())
        })?;
        
        Ok(actions.values().cloned().collect())
    }
} 