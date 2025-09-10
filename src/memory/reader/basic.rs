//! Basic memory reading operations without safety checks

use crate::core::types::{Address, MemoryError, MemoryResult};
use crate::process::ProcessHandle;
use std::mem;

/// Basic memory reader for raw memory operations
pub struct BasicMemoryReader<'a> {
    handle: &'a ProcessHandle,
}

impl<'a> BasicMemoryReader<'a> {
    /// Create a new basic memory reader
    pub fn new(handle: &'a ProcessHandle) -> Self {
        BasicMemoryReader { handle }
    }

    /// Read raw bytes from memory
    pub fn read_raw(&self, address: Address, size: usize) -> MemoryResult<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        self.handle.read_memory(address.as_usize(), &mut buffer)?;
        Ok(buffer)
    }

    /// Read a typed value from memory
    pub fn read<T>(&self, address: Address) -> MemoryResult<T>
    where
        T: Copy + Default,
    {
        let size = mem::size_of::<T>();
        let mut buffer = vec![0u8; size];

        self.handle.read_memory(address.as_usize(), &mut buffer)?;

        // Safety: We're reading exactly size_of::<T>() bytes
        unsafe {
            let ptr = buffer.as_ptr() as *const T;
            Ok(*ptr)
        }
    }

    /// Read an array of typed values
    pub fn read_array<T>(&self, address: Address, count: usize) -> MemoryResult<Vec<T>>
    where
        T: Copy + Default,
    {
        let element_size = mem::size_of::<T>();
        let total_size = element_size * count;
        let mut buffer = vec![0u8; total_size];

        self.handle.read_memory(address.as_usize(), &mut buffer)?;

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let offset = i * element_size;
            unsafe {
                let ptr = buffer[offset..].as_ptr() as *const T;
                result.push(*ptr);
            }
        }

        Ok(result)
    }

    /// Read a null-terminated string
    pub fn read_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        let buffer = self.read_raw(address, max_len)?;

        // Find null terminator
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(max_len);

        String::from_utf8(buffer[..len].to_vec()).map_err(MemoryError::Utf8Error)
    }

    /// Read a wide string (UTF-16)
    pub fn read_wide_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        let byte_size = max_len * 2;
        let buffer = self.read_raw(address, byte_size)?;

        let mut u16_buffer = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let low = buffer[i * 2];
            let high = buffer[i * 2 + 1];
            let value = u16::from_le_bytes([low, high]);
            if value == 0 {
                break;
            }
            u16_buffer.push(value);
        }

        String::from_utf16(&u16_buffer)
            .map_err(|_| MemoryError::InvalidValueType("Invalid UTF-16 string".to_string()))
    }

    /// Read multiple values in batch
    pub fn read_batch<T>(&self, addresses: &[Address]) -> Vec<MemoryResult<T>>
    where
        T: Copy + Default,
    {
        addresses.iter().map(|&addr| self.read(addr)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_basic_reader_creation() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let _reader = BasicMemoryReader::new(&handle);
        // Just verify creation works
    }

    #[test]
    fn test_string_conversion() {
        // Test UTF-8 string parsing logic (no FFI needed)
        let test_data = b"Hello\0World";
        let len = test_data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(test_data.len());
        let result = String::from_utf8(test_data[..len].to_vec());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
    }

    #[test]
    fn test_wide_string_conversion() {
        // Test UTF-16 conversion logic
        let hello_utf16 = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in UTF-16 code points
        let result = String::from_utf16(&hello_utf16);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
    }
}
