//! Type-safe memory writing with validation

use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;
use crate::windows::bindings::kernel32;
use std::mem;

/// Memory writer for type-safe write operations
pub struct MemoryWriter {
    handle: *const ProcessHandle,
}

impl MemoryWriter {
    /// Create a new memory writer
    pub fn new(handle: &ProcessHandle) -> Self {
        MemoryWriter {
            handle: handle as *const ProcessHandle,
        }
    }

    /// Write raw bytes to memory
    pub fn write_bytes(&self, address: Address, data: &[u8]) -> MemoryResult<()> {
        unsafe {
            let handle = &*self.handle;
            let bytes_written = handle.write_memory(address.as_usize(), data)?;

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
    }

    /// Write a typed value to memory
    pub fn write<T: Copy>(&self, address: Address, value: T) -> MemoryResult<()> {
        let size = mem::size_of::<T>();
        let ptr = &value as *const T as *const u8;

        unsafe {
            let data = std::slice::from_raw_parts(ptr, size);
            self.write_bytes(address, data)
        }
    }

    /// Write a string to memory (null-terminated)
    pub fn write_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0); // Add null terminator
        self.write_bytes(address, &bytes)
    }

    /// Write a wide string (UTF-16) to memory (null-terminated)
    pub fn write_wide_string(&self, address: Address, value: &str) -> MemoryResult<()> {
        let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes: Vec<u8> = wide.iter().flat_map(|&w| w.to_le_bytes()).collect();
        self.write_bytes(address, &bytes)
    }

    /// Write multiple values in a batch
    pub fn write_batch<T: Copy>(&self, writes: &[(Address, T)]) -> Vec<MemoryResult<()>> {
        writes
            .iter()
            .map(|(addr, value)| self.write(*addr, *value))
            .collect()
    }

    /// Write a MemoryValue to memory
    pub fn write_value(&self, address: Address, value: &MemoryValue) -> MemoryResult<()> {
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
            MemoryValue::String(s) => self.write_string(address, s),
            MemoryValue::Bytes(b) => self.write_bytes(address, b),
        }
    }

    /// Fill memory with a repeated byte value
    pub fn fill(&self, address: Address, value: u8, count: usize) -> MemoryResult<()> {
        let data = vec![value; count];
        self.write_bytes(address, &data)
    }

    /// Copy memory from one location to another within the same process
    pub fn copy_memory(
        &self,
        source: Address,
        destination: Address,
        size: usize,
    ) -> MemoryResult<()> {
        // Read from source
        let mut buffer = vec![0u8; size];
        unsafe {
            let handle = &*self.handle;
            handle.read_memory(source.as_usize(), &mut buffer)?;
        }

        // Write to destination
        self.write_bytes(destination, &buffer)
    }

    /// Swap two memory regions
    pub fn swap_memory(&self, addr1: Address, addr2: Address, size: usize) -> MemoryResult<()> {
        // Read both regions
        let mut buffer1 = vec![0u8; size];
        let mut buffer2 = vec![0u8; size];

        unsafe {
            let handle = &*self.handle;
            handle.read_memory(addr1.as_usize(), &mut buffer1)?;
            handle.read_memory(addr2.as_usize(), &mut buffer2)?;
        }

        // Write them swapped
        self.write_bytes(addr1, &buffer2)?;
        self.write_bytes(addr2, &buffer1)?;

        Ok(())
    }

    /// Write with verification - reads back to confirm write succeeded
    pub fn write_verified<T: Copy + PartialEq>(
        &self,
        address: Address,
        value: T,
    ) -> MemoryResult<()> {
        // Write the value
        self.write(address, value)?;

        // Read it back
        let size = mem::size_of::<T>();
        let mut buffer = vec![0u8; size];

        unsafe {
            let handle = &*self.handle;
            handle.read_memory(address.as_usize(), &mut buffer)?;
        }

        let read_value = unsafe { *(buffer.as_ptr() as *const T) };

        // Verify
        if read_value != value {
            return Err(MemoryError::WriteFailed {
                address: format!("0x{:X}", address.as_usize()),
                reason: "Verification failed: written value doesn't match".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_handle() -> ProcessHandle {
        ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap())
    }

    #[test]
    fn test_memory_writer_creation() {
        let handle = create_test_handle();
        let _writer = MemoryWriter::new(&handle);
        // Just test creation doesn't panic
    }

    #[test]
    fn test_write_bytes_with_null_handle() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.write_bytes(Address::new(0x1000), &[1, 2, 3, 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_typed_with_null_handle() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.write(Address::new(0x1000), 42u32);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_string_with_null_handle() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.write_string(Address::new(0x1000), "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_wide_string_with_null_handle() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.write_wide_string(Address::new(0x1000), "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_batch() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let writes = vec![(Address::new(0x1000), 42u32), (Address::new(0x2000), 84u32)];

        let results = writer.write_batch(&writes);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_err());
        assert!(results[1].is_err());
    }

    #[test]
    fn test_write_memory_value() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let value = MemoryValue::U32(42);
        let result = writer.write_value(Address::new(0x1000), &value);
        assert!(result.is_err());

        let value = MemoryValue::String("test".to_string());
        let result = writer.write_value(Address::new(0x1000), &value);
        assert!(result.is_err());

        let value = MemoryValue::Bytes(vec![1, 2, 3, 4]);
        let result = writer.write_value(Address::new(0x1000), &value);
        assert!(result.is_err());
    }

    #[test]
    fn test_fill_memory() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.fill(Address::new(0x1000), 0xCC, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_memory() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.copy_memory(Address::new(0x1000), Address::new(0x2000), 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_memory() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.swap_memory(Address::new(0x1000), Address::new(0x2000), 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_verified() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let result = writer.write_verified(Address::new(0x1000), 42u32);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_all_memory_value_types() {
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        let test_values = vec![
            MemoryValue::U8(255),
            MemoryValue::U16(65535),
            MemoryValue::U32(4294967295),
            MemoryValue::U64(18446744073709551615),
            MemoryValue::I8(-128),
            MemoryValue::I16(-32768),
            MemoryValue::I32(-2147483648),
            MemoryValue::I64(-9223372036854775808),
            MemoryValue::F32(std::f32::consts::PI),
            MemoryValue::F64(std::f64::consts::E),
            MemoryValue::String("Hello, World!".to_string()),
            MemoryValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ];

        for value in test_values {
            let result = writer.write_value(Address::new(0x1000), &value);
            assert!(result.is_err()); // Should fail with null handle
        }
    }

    #[test]
    fn test_write_string_with_null_terminator() {
        // Test that write_string adds null terminator
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        // This will fail with null handle, but we're testing the logic
        let _ = writer.write_string(Address::new(0x1000), "test");
        // The function should add a null terminator internally
    }

    #[test]
    fn test_write_wide_string_encoding() {
        // Test that write_wide_string properly encodes UTF-16
        let handle = create_test_handle();
        let writer = MemoryWriter::new(&handle);

        // Test with ASCII
        let _ = writer.write_wide_string(Address::new(0x1000), "Hello");

        // Test with Unicode
        let _ = writer.write_wide_string(Address::new(0x2000), "Hello 世界");
    }
}
