//! Memory-MCP library for Windows memory manipulation

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod core;

// Re-export main types from core module
pub use core::types::{
    Address, MemoryError, MemoryResult, MemoryValue, ModuleInfo, ProcessArchitecture, ProcessId,
    ProcessInfo, ThreadId, ValueType,
};

// Re-export core directly for full access
pub use core::*;
