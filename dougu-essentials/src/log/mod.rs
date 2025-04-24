use anyhow::Result;
use log::{debug, error, info, warn, LevelFilter};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the logger with specified log level
pub fn init(level: LevelFilter) -> Result<()> {
    INIT.call_once(|| {
        env_logger::Builder::new()
            .filter_level(level)
            .init();
        
        debug!("Logger initialized with level: {}", level);
    });
    Ok(())
}

/// Convenience function to log errors
pub fn log_error<E: std::fmt::Display>(err: E) {
    error!("{}", err);
}

/// Convenience function to log warnings
pub fn log_warning<W: std::fmt::Display>(warning: W) {
    warn!("{}", warning);
}

/// Convenience function to log info messages
pub fn log_info<I: std::fmt::Display>(info_msg: I) {
    info!("{}", info_msg);
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
} 