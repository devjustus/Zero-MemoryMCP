//! Safe memory reading with validation and error handling

use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue, ValueType};
use crate::memory::reader::basic::BasicMemoryReader;
use crate::process::ProcessHandle;
use crate::windows::bindings::kernel32;

/// Safe memory reader with validation
pub struct SafeMemoryReader<'a> {
    handle: &'a ProcessHandle,
    basic_reader: BasicMemoryReader<'a>,
}

impl<'a> SafeMemoryReader<'a> {
    /// Create a new safe memory reader
    pub fn new(handle: &'a ProcessHandle) -> Self {
        SafeMemoryReader {
            handle,
            basic_reader: BasicMemoryReader::new(handle),
        }
    }

    /// Validate memory region before reading
    pub fn validate_region(&self, address: Address, size: usize) -> MemoryResult<()> {
        unsafe {
            let mbi = kernel32::virtual_query_ex(self.handle.raw(), address.as_usize())?;

            // Check if memory is committed
            const MEM_COMMIT: u32 = 0x1000;
            if mbi.State != MEM_COMMIT {
                return Err(MemoryError::InvalidAddress(format!(
                    "Memory at 0x{:X} is not committed",
                    address.as_usize()
                )));
            }

            // Check if region is large enough
            if mbi.RegionSize < size {
                return Err(MemoryError::InvalidAddress(format!(
                    "Memory region at 0x{:X} is too small (requested: {}, available: {})",
                    address.as_usize(),
                    size,
                    mbi.RegionSize
                )));
            }

            // Check read permissions
            const PAGE_NOACCESS: u32 = 0x01;
            const PAGE_GUARD: u32 = 0x100;
            if mbi.Protect & PAGE_NOACCESS != 0 || mbi.Protect & PAGE_GUARD != 0 {
                return Err(MemoryError::InvalidAddress(format!(
                    "Memory at 0x{:X} is not readable (protection: 0x{:X})",
                    address.as_usize(),
                    mbi.Protect
                )));
            }

            Ok(())
        }
    }

    /// Read with validation
    pub fn read<T>(&self, address: Address) -> MemoryResult<T>
    where
        T: Copy + Default,
    {
        self.validate_region(address, std::mem::size_of::<T>())?;
        self.basic_reader.read(address)
    }

    /// Read raw bytes with validation
    pub fn read_raw(&self, address: Address, size: usize) -> MemoryResult<Vec<u8>> {
        self.validate_region(address, size)?;
        self.basic_reader.read_raw(address, size)
    }

    /// Read array with validation
    pub fn read_array<T>(&self, address: Address, count: usize) -> MemoryResult<Vec<T>>
    where
        T: Copy + Default,
    {
        let total_size = std::mem::size_of::<T>() * count;
        self.validate_region(address, total_size)?;
        self.basic_reader.read_array(address, count)
    }

    /// Read string with validation
    pub fn read_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        // Validate at least first byte
        self.validate_region(address, 1)?;
        self.basic_reader.read_string(address, max_len)
    }

    /// Read wide string with validation
    pub fn read_wide_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        // Validate at least first 2 bytes
        self.validate_region(address, 2)?;
        self.basic_reader.read_wide_string(address, max_len)
    }

    /// Read a MemoryValue with type information
    pub fn read_value(&self, address: Address, value_type: ValueType) -> MemoryResult<MemoryValue> {
        match value_type {
            ValueType::U8 => Ok(MemoryValue::U8(self.read::<u8>(address)?)),
            ValueType::U16 => Ok(MemoryValue::U16(self.read::<u16>(address)?)),
            ValueType::U32 => Ok(MemoryValue::U32(self.read::<u32>(address)?)),
            ValueType::U64 => Ok(MemoryValue::U64(self.read::<u64>(address)?)),
            ValueType::I8 => Ok(MemoryValue::I8(self.read::<i8>(address)?)),
            ValueType::I16 => Ok(MemoryValue::I16(self.read::<i16>(address)?)),
            ValueType::I32 => Ok(MemoryValue::I32(self.read::<i32>(address)?)),
            ValueType::I64 => Ok(MemoryValue::I64(self.read::<i64>(address)?)),
            ValueType::F32 => Ok(MemoryValue::F32(self.read::<f32>(address)?)),
            ValueType::F64 => Ok(MemoryValue::F64(self.read::<f64>(address)?)),
            ValueType::String => Ok(MemoryValue::String(self.read_string(address, 256)?)),
            ValueType::Bytes => {
                let buffer = self.read_raw(address, 256)?;
                Ok(MemoryValue::Bytes(buffer))
            }
        }
    }

    /// Batch read with validation
    pub fn read_batch<T>(&self, addresses: &[Address]) -> Vec<MemoryResult<T>>
    where
        T: Copy + Default,
    {
        addresses.iter().map(|&addr| self.read(addr)).collect()
    }

    /// Check if address is readable
    pub fn is_readable(&self, address: Address, size: usize) -> bool {
        self.validate_region(address, size).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_reader_creation() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let _reader = SafeMemoryReader::new(&handle);
        // Just verify creation works
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_is_readable() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let reader = SafeMemoryReader::new(&handle);

        // Null address should not be readable
        assert!(!reader.is_readable(Address::new(0), 4));

        // Very high address should not be readable
        assert!(!reader.is_readable(Address::new(0xFFFFFFFFFFFFFFFF), 4));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_validation_errors() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let reader = SafeMemoryReader::new(&handle);

        // Should fail on invalid address
        let result = reader.read::<u32>(Address::new(0));
        assert!(result.is_err());

        // Should fail on inaccessible memory
        let result = reader.read::<u32>(Address::new(0xDEADBEEF));
        assert!(result.is_err());
    }
}
