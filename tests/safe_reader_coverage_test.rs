//! Additional tests to improve coverage for SafeMemoryReader

use memory_mcp::core::types::{Address, MemoryError, ValueType, MemoryValue};
use memory_mcp::memory::reader::safe::SafeMemoryReader;
use memory_mcp::process::ProcessHandle;

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_value_all_types() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test all value types to cover lines 103-119
    let test_address = Address::new(0xDEADBEEF); // Invalid address for error cases
    
    // Test U8 type
    let result = reader.read_value(test_address, ValueType::U8);
    assert!(result.is_err());
    
    // Test U16 type
    let result = reader.read_value(test_address, ValueType::U16);
    assert!(result.is_err());
    
    // Test U32 type
    let result = reader.read_value(test_address, ValueType::U32);
    assert!(result.is_err());
    
    // Test U64 type
    let result = reader.read_value(test_address, ValueType::U64);
    assert!(result.is_err());
    
    // Test I8 type
    let result = reader.read_value(test_address, ValueType::I8);
    assert!(result.is_err());
    
    // Test I16 type
    let result = reader.read_value(test_address, ValueType::I16);
    assert!(result.is_err());
    
    // Test I32 type
    let result = reader.read_value(test_address, ValueType::I32);
    assert!(result.is_err());
    
    // Test I64 type
    let result = reader.read_value(test_address, ValueType::I64);
    assert!(result.is_err());
    
    // Test F32 type
    let result = reader.read_value(test_address, ValueType::F32);
    assert!(result.is_err());
    
    // Test F64 type
    let result = reader.read_value(test_address, ValueType::F64);
    assert!(result.is_err());
    
    // Test String type
    let result = reader.read_value(test_address, ValueType::String);
    assert!(result.is_err());
    
    // Test Bytes type
    let result = reader.read_value(test_address, ValueType::Bytes);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_validate_region_uncommitted_memory() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test validation with uncommitted memory (lines 30-35)
    // Most invalid addresses will be uncommitted
    let result = reader.validate_region(Address::new(0xFFFFFFFF00000000), 4);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_validate_region_insufficient_size() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test region size validation (lines 38-45)
    // Try to read a very large region from a valid address
    let result = reader.validate_region(Address::new(0x10000), usize::MAX);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_validate_region_protected_memory() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test protected memory validation (lines 48-56)
    // Null page is typically protected
    let result = reader.validate_region(Address::new(0), 4);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_batch() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test batch reading (lines 123-128)
    let addresses = vec![
        Address::new(0),
        Address::new(0xDEADBEEF),
        Address::new(0xFFFFFFFF),
    ];
    
    let results: Vec<_> = reader.read_batch::<u32>(&addresses);
    
    // All should fail with invalid addresses
    assert_eq!(results.len(), 3);
    for result in results {
        assert!(result.is_err());
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_with_validation() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test read with validation (lines 63-69)
    let result = reader.read::<u64>(Address::new(0));
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_raw_with_validation() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test read_raw with validation (lines 72-75)
    let result = reader.read_raw(Address::new(0), 256);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_array_with_validation() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test read_array with validation (lines 78-85)
    let result = reader.read_array::<i32>(Address::new(0), 10);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_string_with_validation() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test read_string with validation (lines 88-92)
    let result = reader.read_string(Address::new(0), 256);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_wide_string_with_validation() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = SafeMemoryReader::new(&handle);
    
    // Test read_wide_string with validation (lines 95-99)
    let result = reader.read_wide_string(Address::new(0), 256);
    assert!(result.is_err());
}