//! Core module containing fundamental types and traits for Memory-MCP
//!
//! This module provides the foundational building blocks used throughout
//! the Memory-MCP server, including address handling, memory values,
//! process information, and error types.

pub mod types;

// Re-export commonly used types for convenience
pub use types::{
    Address,
    MemoryValue,
    ProcessInfo,
    ScanResult,
    MemoryError,
    MemoryResult,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

// Platform verification at compile time
#[cfg(not(target_os = "windows"))]
compile_error!("Memory-MCP only supports Windows platform");

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Memory-MCP requires 64-bit architecture");