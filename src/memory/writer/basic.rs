//! Basic memory write operations without additional validation
//!
//! This module provides the core memory writing functionality with minimal overhead.

use super::{BatchWrite, ExtendedWrite, MemoryCopy, MemoryWrite};
use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;
use std::mem;

/// Basic memory writer for raw write operations
pub struct BasicMemoryWriter<'a> {
    handle: &'a ProcessHandle,
}

impl<'a> BasicMemoryWriter<'a> {
    /// Create a new basic memory writer
    pub fn new(handle: &'a ProcessHandle) -> Self {
        BasicMemoryWriter { handle }
    }

    /// Get the process handle
    pub fn handle(&self) -> &ProcessHandle {
        self.handle
    }
}

impl<'a> MemoryWrite for BasicMemoryWriter<'a> {
    /// Write raw bytes to memory
    fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        if data.is_empty() {
            return Ok(());
        }

        let bytes_written = self.handle.write_memory(address.as_usize(), data)?;

        if bytes_written != data.len() {
            return Err(MemoryError::WriteFailed {
                address: format!("0x{:X}", address.as_usize()),
                reason: format!(
                    "Partial write: expected {} bytes, wrote {} bytes",
                    data.len(),
                    bytes_written
                ),
            });
        }

        Ok(())
    }

    /// Write a typed value to memory
    fn write<T: Copy>(&self, address: Address, value: T) -> MemoryResult<()> {
        let size = mem::size_of::<T>();
        let ptr = &value as *const T as *const u8;

        unsafe {
            let data = std::slice::from_raw_parts(ptr, size);
            self.write_bytes(address, data)
        }
    }

    /// Write a memory value to memory
    fn write_value(&self, address: Address, value: &MemoryValue) -> MemoryResult<()> {
        match value {
            MemoryValue::U8(v) => self.write(address, *v),
            MemoryValue::U16(v) => self.write(address, *v),
            MemoryValue::U32(v) => self.write(address, *v),
            MemoryValue::U64(v) => self.write(address, *v),
            MemoryValue::I8(v) => self.write(address, *v),
            MemoryValue::I16(v) => self.write(address, *v),
            MemoryValue::I32(v) => self.write(address, *v),
            MemoryValue::I64(v) => self.write(address, *v),
            MemoryValue::F32(v) => self.write(address, *v),
            MemoryValue::F64(v) => self.write(address, *v),
            MemoryValue::String(s) => {
                let mut bytes = s.as_bytes().to_vec();
                bytes.push(0);
                self.write_bytes(address, &bytes)
            }
            MemoryValue::Bytes(b) => self.write_bytes(address, b),
        }
    }
}

impl<'a> ExtendedWrite for BasicMemoryWriter<'a> {
    /// Write a null-terminated string to memory
    fn write_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        self.write_bytes(address, &bytes)
    }

    /// Write a null-terminated wide string (UTF-16) to memory
    fn write_wide_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes: Vec<u8> = wide.iter().flat_map(|&w| w.to_le_bytes()).collect();
        self.write_bytes(address, &bytes)
    }

    /// Fill memory with a repeated byte value
    fn fill(&self, address: Address, value: u8, count: usize) -> MemoryResult<()> {
        if count == 0 {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 4096;
        let chunk = vec![value; CHUNK_SIZE.min(count)];

        let mut offset = 0;
        while offset < count {
            let write_size = (count - offset).min(CHUNK_SIZE);
            let write_addr = Address::new(address.as_usize() + offset);
            self.write_bytes(write_addr, &chunk[..write_size])?;
            offset += write_size;
        }

        Ok(())
    }
}

impl<'a> BatchWrite for BasicMemoryWriter<'a> {
    /// Write multiple values in a batch
    fn write_batch<T: Copy>(&self, writes: &[(Address, T)]) -> Vec<MemoryResult<()>> {
        writes
            .iter()
            .map(|(addr, value)| self.write(*addr, *value))
            .collect()
    }
}

impl<'a> MemoryCopy for BasicMemoryWriter<'a> {
    /// Copy memory from one location to another within the same process
    fn copy_memory(&self, source: Address, destination: Address, size: usize) -> MemoryResult<()> {
        if size == 0 {
            return Ok(());
        }

        const CHUNK_SIZE: usize = 8192;
        let mut buffer = vec![0u8; CHUNK_SIZE.min(size)];

        let mut offset = 0;
        while offset < size {
            let copy_size = (size - offset).min(CHUNK_SIZE);
            let src_addr = Address::new(source.as_usize() + offset);
            let dst_addr = Address::new(destination.as_usize() + offset);

            self.handle
                .read_memory(src_addr.as_usize(), &mut buffer[..copy_size])?;
            self.write_bytes(dst_addr, &buffer[..copy_size])?;

            offset += copy_size;
        }

        Ok(())
    }

    /// Swap two memory regions
    fn swap_memory(&self, addr1: Address, addr2: Address, size: usize) -> MemoryResult<()> {
        if size == 0 {
            return Ok(());
        }

        let mut buffer1 = vec![0u8; size];
        let mut buffer2 = vec![0u8; size];

        self.handle.read_memory(addr1.as_usize(), &mut buffer1)?;
        self.handle.read_memory(addr2.as_usize(), &mut buffer2)?;

        self.write_bytes(addr1, &buffer2)?;
        self.write_bytes(addr2, &buffer1)?;

        Ok(())
    }
}
