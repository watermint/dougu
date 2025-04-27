use std::sync::Arc;

use crate::core::error::Result;
use crate::log::interface::{LogLevel, LogRecord};

/// Filter trait for filtering log records
pub trait Filter: Send + Sync {
    /// Determine if a log record should be logged
    fn should_log(&self, record: &LogRecord) -> Result<bool>;
}

/// Filter based on log level
pub struct LevelFilter {
    /// Minimum log level to log
    minimum_level: LogLevel,
}

impl LevelFilter {
    /// Create a new level filter
    pub fn new(minimum_level: LogLevel) -> Self {
        Self { minimum_level }
    }

    /// Get the minimum log level
    pub fn minimum_level(&self) -> LogLevel {
        self.minimum_level
    }

    /// Set the minimum log level
    pub fn set_minimum_level(&mut self, level: LogLevel) {
        self.minimum_level = level;
    }
}

impl Filter for LevelFilter {
    fn should_log(&self, record: &LogRecord) -> Result<bool> {
        Ok(record.level >= self.minimum_level)
    }
}

/// Filter based on module name
pub struct ModuleFilter {
    /// Module names to include (if empty, all modules are included)
    include_modules: Vec<String>,
    /// Module names to exclude
    exclude_modules: Vec<String>,
}

impl ModuleFilter {
    /// Create a new module filter
    pub fn new(include_modules: Vec<String>, exclude_modules: Vec<String>) -> Self {
        Self {
            include_modules,
            exclude_modules,
        }
    }

    /// Add a module to include
    pub fn add_include(&mut self, module: String) {
        self.include_modules.push(module);
    }

    /// Add a module to exclude
    pub fn add_exclude(&mut self, module: String) {
        self.exclude_modules.push(module);
    }
}

impl Filter for ModuleFilter {
    fn should_log(&self, record: &LogRecord) -> Result<bool> {
        // Check exclude list first
        for exclude in &self.exclude_modules {
            if record.module.starts_with(exclude) {
                return Ok(false);
            }
        }

        // If include list is empty, include all modules
        if self.include_modules.is_empty() {
            return Ok(true);
        }

        // Check include list
        for include in &self.include_modules {
            if record.module.starts_with(include) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Composite filter that combines multiple filters
pub struct CompositeFilter {
    /// The filters to apply
    filters: Vec<Arc<dyn Filter>>,
    /// Whether all filters must pass (true) or any filter can pass (false)
    require_all: bool,
}

impl CompositeFilter {
    /// Create a new composite filter where all filters must pass
    pub fn all(filters: Vec<Arc<dyn Filter>>) -> Self {
        Self {
            filters,
            require_all: true,
        }
    }

    /// Create a new composite filter where any filter can pass
    pub fn any(filters: Vec<Arc<dyn Filter>>) -> Self {
        Self {
            filters,
            require_all: false,
        }
    }

    /// Add a filter to the composite
    pub fn add_filter(&mut self, filter: Arc<dyn Filter>) {
        self.filters.push(filter);
    }
}

impl Filter for CompositeFilter {
    fn should_log(&self, record: &LogRecord) -> Result<bool> {
        if self.filters.is_empty() {
            return Ok(true);
        }

        for filter in &self.filters {
            let result = filter.should_log(record)?;

            if self.require_all && !result {
                // If all filters must pass and one fails, return false
                return Ok(false);
            } else if !self.require_all && result {
                // If any filter can pass and one does, return true
                return Ok(true);
            }
        }

        // If we get here, either all filters passed (when require_all is true)
        // or all filters failed (when require_all is false)
        Ok(self.require_all)
    }
} 