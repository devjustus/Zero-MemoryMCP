//! Memory reading module with basic and safe implementations

pub mod basic;
pub mod cache;
pub mod safe;

pub use basic::BasicMemoryReader;
pub use cache::{MemoryReader, ReadCache};
pub use safe::SafeMemoryReader;

use crate::core::types::{Address, MemoryResult, MemoryValue, ValueType};
use crate::process::ProcessHandle;

/// Unified memory reader interface
pub struct Reader<'a> {
    handle: &'a ProcessHandle,
    cached: MemoryReader<'a>,
    safe: SafeMemoryReader<'a>,
}

impl<'a> Reader<'a> {
    /// Create a new reader
    pub fn new(handle: &'a ProcessHandle) -> Self {
        Reader {
            handle,
            cached: MemoryReader::new(handle),
            safe: SafeMemoryReader::new(handle),
        }
    }

    /// Read with caching
    pub fn read_cached<T>(&mut self, address: Address) -> MemoryResult<T>
    where
        T: Copy,
    {
        self.cached.read(address)
    }

    /// Read with validation
    pub fn read_safe<T>(&self, address: Address) -> MemoryResult<T>
    where
        T: Copy + Default,
    {
        self.safe.read(address)
    }

    /// Read raw bytes
    pub fn read_bytes(&mut self, address: Address, size: usize) -> MemoryResult<Vec<u8>> {
        self.cached.read_bytes(address, size)
    }

    /// Read a value by type
    pub fn read_value(&self, address: Address, value_type: ValueType) -> MemoryResult<MemoryValue> {
        self.safe.read_value(address, value_type)
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cached.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_unified_reader() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());

        let mut reader = Reader::new(&handle);
        assert_eq!(reader.cached.cache_size(), 0);

        reader.clear_cache();
        assert_eq!(reader.cached.cache_size(), 0);
    }
}
