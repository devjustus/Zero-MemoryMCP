//! Safe memory write operations with validation and verification
//!
//! This module provides memory writing functionality with additional safety checks
//! including verification, bounds checking, and permission validation.

use super::{BasicMemoryWriter, BatchWrite, ExtendedWrite, MemoryCopy, MemoryWrite};
use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;
use std::mem;

/// Safe memory writer with validation and verification
pub struct SafeMemoryWriter<'a> {
    basic_writer: BasicMemoryWriter<'a>,
    verify_writes: bool,
    check_permissions: bool,
}

impl<'a> SafeMemoryWriter<'a> {
    /// Create a new safe memory writer
    pub fn new(handle: &'a ProcessHandle) -> Self {
        SafeMemoryWriter {
            basic_writer: BasicMemoryWriter::new(handle),
            verify_writes: true,
            check_permissions: true,
        }
    }

    /// Enable or disable write verification
    pub fn set_verify_writes(&mut self, verify: bool) {
        self.verify_writes = verify;
    }

    /// Enable or disable permission checking
    pub fn set_check_permissions(&mut self, check: bool) {
        self.check_permissions = check;
    }

    /// Write with verification - reads back to confirm write succeeded
    pub fn write_verified<T: Copy + PartialEq>(
        &self,
        address: Address,
        value: T,
    ) -> MemoryResult<()> {
        self.basic_writer.write(address, value)?;

        let size = mem::size_of::<T>();
        let mut buffer = vec![0u8; size];

        self.basic_writer
            .handle()
            .read_memory(address.as_usize(), &mut buffer)?;

        let read_value = unsafe { *(buffer.as_ptr() as *const T) };

        if read_value != value {
            return Err(MemoryError::WriteFailed {
                address: format!("0x{:X}", address.as_usize()),
                reason: "Verification failed: written value doesn't match".to_string(),
            });
        }

        Ok(())
    }

    /// Write with automatic backup
    pub fn write_with_backup<T: Copy>(&self, address: Address, value: T) -> MemoryResult<Vec<u8>> {
        let size = mem::size_of::<T>();
        let mut backup = vec![0u8; size];

        self.basic_writer
            .handle()
            .read_memory(address.as_usize(), &mut backup)?;
        self.basic_writer.write(address, value)?;

        Ok(backup)
    }

    /// Restore from backup
    pub fn restore_from_backup(&self, address: Address, backup: &[u8]) -> MemoryResult<()> {
        self.basic_writer.write_bytes(address, backup)
    }

    /// Check if address is writable
    fn check_writable(&self, address: Address, size: usize) -> MemoryResult<()> {
        if !self.check_permissions {
            return Ok(());
        }

        // For now, we'll just validate the address range
        // In a full implementation, we'd check memory protection flags
        if address.as_usize() == 0 {
            return Err(MemoryError::InvalidAddress(format!(
                "0x{:X} - Null pointer",
                address.as_usize()
            )));
        }

        // Check for potential overflow
        if address.as_usize().saturating_add(size) < address.as_usize() {
            return Err(MemoryError::InvalidAddress(format!(
                "0x{:X} - Address overflow",
                address.as_usize()
            )));
        }

        Ok(())
    }
}

impl<'a> MemoryWrite for SafeMemoryWriter<'a> {
    fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        self.check_writable(address, data.len())?;

        if self.verify_writes && !data.is_empty() {
            self.basic_writer.write_bytes(address, data)?;

            let mut verify_buffer = vec![0u8; data.len()];
            self.basic_writer
                .handle()
                .read_memory(address.as_usize(), &mut verify_buffer)?;

            if verify_buffer != data {
                return Err(MemoryError::WriteFailed {
                    address: format!("0x{:X}", address.as_usize()),
                    reason: "Verification failed: written data doesn't match".to_string(),
                });
            }

            Ok(())
        } else {
            self.basic_writer.write_bytes(address, data)
        }
    }

    fn write<T: Copy>(&self, address: Address, value: T) -> MemoryResult<()> {
        let size = mem::size_of::<T>();
        self.check_writable(address, size)?;
        self.basic_writer.write(address, value)
    }

    fn write_value(&self, address: Address, value: &MemoryValue) -> MemoryResult<()> {
        let size = value.size();
        self.check_writable(address, size)?;
        self.basic_writer.write_value(address, value)
    }
}

impl<'a> ExtendedWrite for SafeMemoryWriter<'a> {
    fn write_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let size = value.len() + 1; // +1 for null terminator
        self.check_writable(address, size)?;
        self.basic_writer.write_string(address, value)
    }

    fn write_wide_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let size = (value.encode_utf16().count() + 1) * 2; // +1 for null terminator, *2 for wide chars
        self.check_writable(address, size)?;
        self.basic_writer.write_wide_string(address, value)
    }

    fn fill(&self, address: Address, value: u8, count: usize) -> MemoryResult<()> {
        self.check_writable(address, count)?;
        self.basic_writer.fill(address, value, count)
    }
}

impl<'a> BatchWrite for SafeMemoryWriter<'a> {
    fn write_batch<T: Copy>(&self, writes: &[(Address, T)]) -> Vec<MemoryResult<()>> {
        let size = mem::size_of::<T>();
        writes
            .iter()
            .map(|(addr, value)| {
                self.check_writable(*addr, size)?;
                self.write(*addr, *value)
            })
            .collect()
    }
}

impl<'a> MemoryCopy for SafeMemoryWriter<'a> {
    fn copy_memory(&self, source: Address, destination: Address, size: usize) -> MemoryResult<()> {
        self.check_writable(destination, size)?;
        self.basic_writer.copy_memory(source, destination, size)
    }

    fn swap_memory(&self, addr1: Address, addr2: Address, size: usize) -> MemoryResult<()> {
        self.check_writable(addr1, size)?;
        self.check_writable(addr2, size)?;
        self.basic_writer.swap_memory(addr1, addr2, size)
    }
}
