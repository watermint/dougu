pub mod error;
pub mod constants;
pub mod utils;

pub use error::Error;
pub type Result<T> = anyhow::Result<T>;

/// Initialize logging for the application
pub fn init_logging() {
    use tracing_subscriber::fmt::format::FmtSpan;
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
} 