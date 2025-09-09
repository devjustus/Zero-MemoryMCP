//! Type-safe memory reading with caching support

use crate::core::types::{Address, MemoryError, MemoryResult, MemoryValue};
use crate::process::ProcessHandle;
use crate::windows::bindings::kernel32;
use std::collections::HashMap;
use std::mem;

/// Cache entry for read operations
#[derive(Debug, Clone)]
pub struct CacheEntry {
    data: Vec<u8>,
    address: Address,
    timestamp: std::time::Instant,
}

/// Read cache for frequently accessed memory regions
pub struct ReadCache {
    entries: HashMap<Address, CacheEntry>,
    max_age_ms: u128,
    max_entries: usize,
}

impl ReadCache {
    /// Create a new read cache
    pub fn new(max_entries: usize, max_age_ms: u128) -> Self {
        ReadCache {
            entries: HashMap::new(),
            max_age_ms,
            max_entries,
        }
    }

    /// Get cached data if available and not expired
    pub fn get(&self, address: Address, size: usize) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get(&address) {
            if entry.data.len() >= size {
                let age = entry.timestamp.elapsed().as_millis();
                if age < self.max_age_ms {
                    return Some(entry.data[..size].to_vec());
                }
            }
        }
        None
    }

    /// Store data in cache
    pub fn put(&mut self, address: Address, data: Vec<u8>) {
        // Evict oldest entry if cache is full
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&address) {
            if let Some(oldest) = self.find_oldest_entry() {
                self.entries.remove(&oldest);
            }
        }

        self.entries.insert(
            address,
            CacheEntry {
                data,
                address,
                timestamp: std::time::Instant::now(),
            },
        );
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    fn find_oldest_entry(&self) -> Option<Address> {
        self.entries
            .values()
            .min_by_key(|e| e.timestamp)
            .map(|e| e.address)
    }
}

/// Memory reader with type-safe operations
pub struct MemoryReader {
    handle: *const ProcessHandle,
    cache: ReadCache,
}

impl MemoryReader {
    /// Create a new memory reader
    pub fn new(handle: &ProcessHandle) -> Self {
        MemoryReader {
            handle: handle as *const ProcessHandle,
            cache: ReadCache::new(100, 1000), // 100 entries, 1 second max age
        }
    }

    /// Read raw bytes from memory
    pub fn read_bytes(&mut self, address: Address, size: usize) -> MemoryResult<Vec<u8>> {
        // Check cache first
        if let Some(cached) = self.cache.get(address, size) {
            return Ok(cached);
        }

        // Read from process
        let mut buffer = vec![0u8; size];
        unsafe {
            let handle = &*self.handle;
            handle.read_memory(address.as_usize(), &mut buffer)?;
        }

        // Store in cache
        self.cache.put(address, buffer.clone());
        Ok(buffer)
    }

    /// Read a typed value from memory
    pub fn read<T: Copy>(&self, address: Address) -> MemoryResult<T> {
        let size = mem::size_of::<T>();
        let mut buffer = vec![0u8; size];

        unsafe {
            let handle = &*self.handle;
            handle.read_memory(address.as_usize(), &mut buffer)?;
            
            // Safety: We're reading exactly size_of::<T>() bytes
            Ok(*(buffer.as_ptr() as *const T))
        }
    }

    /// Read a null-terminated string from memory
    pub fn read_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        let mut buffer = vec![0u8; max_len];
        
        unsafe {
            let handle = &*self.handle;
            handle.read_memory(address.as_usize(), &mut buffer)?;
        }

        // Find null terminator
        let len = buffer.iter().position(|&b| b == 0).unwrap_or(max_len);
        
        String::from_utf8(buffer[..len].to_vec())
            .map_err(|e| MemoryError::Utf8Error(e))
    }

    /// Read a wide string (UTF-16) from memory
    pub fn read_wide_string(&self, address: Address, max_len: usize) -> MemoryResult<String> {
        let mut buffer = vec![0u16; max_len];
        let byte_size = max_len * 2;
        let mut byte_buffer = vec![0u8; byte_size];

        unsafe {
            let handle = &*self.handle;
            handle.read_memory(address.as_usize(), &mut byte_buffer)?;
            
            // Convert bytes to u16 array
            for i in 0..max_len {
                buffer[i] = u16::from_le_bytes([byte_buffer[i * 2], byte_buffer[i * 2 + 1]]);
            }
        }

        // Find null terminator
        let len = buffer.iter().position(|&w| w == 0).unwrap_or(max_len);
        
        String::from_utf16(&buffer[..len])
            .map_err(|_| MemoryError::InvalidValueType("Invalid UTF-16 string".to_string()))
    }

    /// Read multiple values in a batch
    pub fn read_batch<T: Copy>(&self, addresses: &[Address]) -> Vec<MemoryResult<T>> {
        addresses.iter().map(|&addr| self.read(addr)).collect()
    }

    /// Read a MemoryValue from memory
    pub fn read_value(&self, address: Address, value_type: crate::core::types::ValueType) -> MemoryResult<MemoryValue> {
        use crate::core::types::ValueType;
        
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
                // For bytes, read a default size
                let mut buffer = vec![0u8; 256];
                unsafe {
                    let handle = &*self.handle;
                    handle.read_memory(address.as_usize(), &mut buffer)?;
                }
                Ok(MemoryValue::Bytes(buffer))
            }
        }
    }

    /// Clear the read cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_cache_operations() {
        let mut cache = ReadCache::new(2, 100); // 2 entries, 100ms max age
        
        // Test put and get
        cache.put(Address::new(0x1000), vec![1, 2, 3, 4]);
        assert_eq!(cache.get(Address::new(0x1000), 4), Some(vec![1, 2, 3, 4]));
        assert_eq!(cache.get(Address::new(0x1000), 2), Some(vec![1, 2]));
        assert_eq!(cache.get(Address::new(0x2000), 4), None);
        
        // Test cache size
        assert_eq!(cache.size(), 1);
        
        // Test eviction
        cache.put(Address::new(0x2000), vec![5, 6, 7, 8]);
        assert_eq!(cache.size(), 2);
        
        cache.put(Address::new(0x3000), vec![9, 10, 11, 12]);
        assert_eq!(cache.size(), 2); // Oldest should be evicted
        
        // Test expiration
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(Address::new(0x2000), 4), None); // Should be expired
        
        // Test clear
        cache.clear();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_find_oldest() {
        let mut cache = ReadCache::new(3, 1000);
        
        cache.put(Address::new(0x1000), vec![1]);
        thread::sleep(Duration::from_millis(10));
        cache.put(Address::new(0x2000), vec![2]);
        thread::sleep(Duration::from_millis(10));
        cache.put(Address::new(0x3000), vec![3]);
        
        assert_eq!(cache.find_oldest_entry(), Some(Address::new(0x1000)));
    }

    #[test]
    fn test_memory_reader_creation() {
        // Try to open current process for testing
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| {
                // If that fails, just test that creation works with any handle
                // The actual operations will fail but that's expected in tests
                return ProcessHandle::open_for_read(4).unwrap_or_else(|_| {
                    panic!("Cannot create test handle");
                });
            });
        
        let reader = MemoryReader::new(&handle);
        assert_eq!(reader.cache_size(), 0);
    }

    #[test]
    fn test_read_batch() {
        let handle = ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
        
        let reader = MemoryReader::new(&handle);
        let addresses = vec![Address::new(0x1000), Address::new(0x2000)];
        let results: Vec<MemoryResult<u32>> = reader.read_batch(&addresses);
        
        assert_eq!(results.len(), 2);
        // Both might fail depending on memory protection
        // Just check that we get results back
    }

    #[test]
    fn test_cache_with_partial_data() {
        let mut cache = ReadCache::new(10, 1000);
        
        // Put 10 bytes
        cache.put(Address::new(0x1000), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        
        // Request 5 bytes - should succeed
        assert_eq!(cache.get(Address::new(0x1000), 5), Some(vec![0, 1, 2, 3, 4]));
        
        // Request 15 bytes - should fail (not enough cached data)
        assert_eq!(cache.get(Address::new(0x1000), 15), None);
    }

    #[test]
    fn test_cache_replacement() {
        let mut cache = ReadCache::new(1, 1000); // Only 1 entry allowed
        
        cache.put(Address::new(0x1000), vec![1, 2, 3]);
        assert_eq!(cache.size(), 1);
        
        cache.put(Address::new(0x2000), vec![4, 5, 6]);
        assert_eq!(cache.size(), 1);
        
        // First entry should be gone
        assert_eq!(cache.get(Address::new(0x1000), 3), None);
        // Second entry should be present
        assert_eq!(cache.get(Address::new(0x2000), 3), Some(vec![4, 5, 6]));
    }
}