//! Memory operations module for reading, writing and scanning process memory
//!
//! This module provides safe abstractions for memory manipulation including:
//! - Type-safe memory reading and writing
//! - Batch operations for performance
//! - Memory region validation
//! - Basic pattern scanning

pub mod reader;
pub mod scanner;
pub mod writer;

pub use reader::{BasicMemoryReader, MemoryReader, ReadCache, Reader, SafeMemoryReader};
pub use scanner::{ComparisonType, MemoryScanner, ScanOptions, ScanPattern};
pub use writer::{create_safe_writer, create_writer, BasicMemoryWriter, SafeMemoryWriter};

use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;
use crate::windows::bindings::kernel32;
use std::collections::HashMap;

/// Memory operation context that holds process handle and provides unified interface
pub struct MemoryOperations {
    handle: ProcessHandle,
}

impl MemoryOperations {
    /// Create new memory operations context for a process
    pub fn new(handle: ProcessHandle) -> Self {
        MemoryOperations { handle }
    }

    /// Get a reference to the memory reader
    pub fn reader(&self) -> MemoryReader<'_> {
        MemoryReader::new(&self.handle)
    }

    /// Get a mutable reference to the memory reader
    pub fn reader_mut(&mut self) -> MemoryReader<'_> {
        MemoryReader::new(&self.handle)
    }

    /// Get a reference to the memory writer
    pub fn writer(&self) -> BasicMemoryWriter<'_> {
        BasicMemoryWriter::new(&self.handle)
    }

    /// Get a safe memory writer with validation
    pub fn safe_writer(&self) -> SafeMemoryWriter<'_> {
        SafeMemoryWriter::new(&self.handle)
    }

    /// Get a reference to the memory scanner
    pub fn scanner(&self) -> MemoryScanner<'_> {
        MemoryScanner::new(&self.handle)
    }

    /// Read a value from memory
    pub fn read<T: Copy>(&mut self, address: Address) -> MemoryResult<T> {
        let reader = MemoryReader::new(&self.handle);
        reader.read(address)
    }

    /// Write a value to memory
    pub fn write<T: Copy>(&self, address: Address, value: T) -> MemoryResult<()> {
        use writer::MemoryWrite;
        let writer = BasicMemoryWriter::new(&self.handle);
        writer.write(address, value)
    }

    /// Scan for a pattern in memory
    pub fn scan(&self, pattern: &ScanPattern, options: ScanOptions) -> MemoryResult<Vec<Address>> {
        let scanner = MemoryScanner::new(&self.handle);
        scanner.scan(pattern, options)
    }
}

/// Validate that a memory region is accessible
pub fn validate_region(handle: &ProcessHandle, address: Address, size: usize) -> MemoryResult<()> {
    // Query the memory region to check if it's valid
    unsafe {
        let mbi = kernel32::virtual_query_ex(handle.raw(), address.as_usize())?;

        // Check if the region is committed
        const MEM_COMMIT: u32 = 0x1000;
        if mbi.State != MEM_COMMIT {
            return Err(MemoryError::InvalidAddress(format!(
                "Memory at 0x{:X} is not committed",
                address.as_usize()
            )));
        }

        // Check if the region is large enough
        if mbi.RegionSize < size {
            return Err(MemoryError::InvalidAddress(format!(
                "Memory region at 0x{:X} is too small (requested: {}, available: {})",
                address.as_usize(),
                size,
                mbi.RegionSize
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessHandle;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_operations_creation() {
        // Test with current process handle
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let ops = MemoryOperations::new(handle);
        let reader = ops.reader();
        assert!(reader.cache_size() == 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_validate_region_with_null_handle() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        // Test with an invalid address that should fail
        let result = validate_region(&handle, Address::new(0x0), 100);
        assert!(result.is_err());
    }
}
