// Domain crate - Business logic and domain models
// Re-exports all submodules

// Entity module - Core domain entities
pub mod entity;

// Service module - Business logic and operations
pub mod service;

// Repository module - Data access interfaces
pub mod repository;

// Value module - Value objects
pub mod value;

// Event module - Domain events
pub mod event;

// I18n module - Internationalization support
pub mod i18n;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // Basic test to ensure the module structure is created properly
        assert!(true);
    }
} 