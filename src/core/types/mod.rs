//! Core type definitions for Memory-MCP
//!
//! This module contains all fundamental types used throughout the application,
//! including address wrappers, memory values, process information, and error types.

mod address;
mod value;
mod process_info;
mod scan_result;
mod error;

// Re-export all public types
pub use address::Address;
pub use value::MemoryValue;
pub use process_info::{ProcessInfo, ProcessArchitecture, ModuleInfo};
pub use scan_result::{ScanResult, ScanSession, ScanType, ValueType};
pub use error::{MemoryError, MemoryResult};

// Common type aliases
pub type ProcessId = u32;
pub type ThreadId = u32;
pub type Offset = usize;
pub type Size = usize;