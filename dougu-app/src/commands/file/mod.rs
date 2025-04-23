// Re-export everything from the original lib.rs
pub use self::lib::*;
pub mod lib {
    include!("lib.rs");
} 