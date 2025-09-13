//! Memory writing module for safe and efficient memory operations
//!
//! This module provides type-safe memory writing functionality with support for:
//! - Basic memory writing operations
//! - Safe writing with validation
//! - Batch operations
//! - Memory manipulation utilities
//! - Automatic backup and restore

pub mod backup;
pub mod basic;
pub mod safe;

pub use backup::{BackupConfig, BackupEntry, MemoryBackup};
pub use basic::BasicMemoryWriter;
pub use safe::SafeMemoryWriter;

use crate::core::types::{Address, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;

/// Common trait for memory write operations
pub trait MemoryWrite {
    /// Write raw bytes to memory
    fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()>;

    /// Write a typed value to memory
    fn write<T: Copy>(&self, address: Address, value: T) -> MemoryResult<()>;

    /// Write a memory value to memory
    fn write_value(&self, address: Address, value: &MemoryValue) -> MemoryResult<()>;
}

/// Extended write operations
pub trait ExtendedWrite: MemoryWrite {
    /// Write a string to memory
    fn write_string(&self, address: Address, value: &str) -> MemoryResult<()>;

    /// Write a wide string to memory
    fn write_wide_string(&self, address: Address, value: &str) -> MemoryResult<()>;

    /// Fill memory with a repeated byte value
    fn fill(&self, address: Address, value: u8, count: usize) -> MemoryResult<()>;
}

/// Batch write operations
pub trait BatchWrite: MemoryWrite {
    /// Write multiple values in a batch
    fn write_batch<T: Copy>(&self, writes: &[(Address, T)]) -> Vec<MemoryResult<()>>;
}

/// Memory copy operations
pub trait MemoryCopy: MemoryWrite {
    /// Copy memory from one location to another
    fn copy_memory(&self, source: Address, destination: Address, size: usize) -> MemoryResult<()>;

    /// Swap two memory regions
    fn swap_memory(&self, addr1: Address, addr2: Address, size: usize) -> MemoryResult<()>;
}

/// Create a memory writer for the given process handle
pub fn create_writer<'a>(handle: &'a ProcessHandle) -> BasicMemoryWriter<'a> {
    BasicMemoryWriter::new(handle)
}

/// Create a safe memory writer with validation
pub fn create_safe_writer<'a>(handle: &'a ProcessHandle) -> SafeMemoryWriter<'a> {
    SafeMemoryWriter::new(handle)
}
