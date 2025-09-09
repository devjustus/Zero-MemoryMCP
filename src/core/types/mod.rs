//! Core type definitions for Memory-MCP
//!
//! This module contains all fundamental types used throughout the application,
//! including address wrappers, memory values, process information, and error types.

mod address;
mod error;
mod process_info;
mod scan_result;
mod value;

// Re-export all public types
pub use address::Address;
pub use error::{MemoryError, MemoryResult};
pub use process_info::{ModuleInfo, ProcessArchitecture, ProcessInfo};
pub use scan_result::{RegionInfo, ScanResult, ScanSession, ScanType};
pub use value::{MemoryValue, ValueType};

// Common type aliases
pub type ProcessId = u32;
pub type ThreadId = u32;
pub type Offset = usize;
pub type Size = usize;
