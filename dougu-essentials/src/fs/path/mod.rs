// This module is responsible for handling file paths in a way that's
// compatible with different file systems, both local and remote.

pub mod core;
pub mod local;
mod essential;
pub mod default;
mod resolver;
pub mod utils;

#[cfg(test)]
mod tests;

pub use core::*;
pub use default::*;
pub use essential::*;
pub use local::*;
pub use resolver::*;
pub use utils::*;
