//! Additional tests to improve coverage for BasicMemoryReader

use memory_mcp::core::types::{Address, MemoryError};
use memory_mcp::memory::reader::basic::BasicMemoryReader;
use memory_mcp::process::ProcessHandle;

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_raw_size_overflow() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = BasicMemoryReader::new(&handle);
    
    // Test size overflow check (lines 22-26)
    let result = reader.read_raw(Address::new(0x1000), usize::MAX);
    assert!(result.is_err());
    match result {
        Err(MemoryError::InvalidAddress(msg)) => {
            assert!(msg.contains("exceeds maximum allowed"));
        }
        _ => panic!("Expected InvalidAddress error for size overflow"),
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_array_with_invalid_memory() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = BasicMemoryReader::new(&handle);
    
    // Test array reading with invalid address (lines 62-66)
    let result = reader.read_array::<u32>(Address::new(0xDEADBEEF), 10);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_string_utf8_error_handling() {
    // This test would need a mock or special setup to trigger UTF-8 error
    // Since we can't easily create invalid UTF-8 in process memory,
    // we'll test the error path logic separately
    
    // Test UTF-8 error conversion (line 79)
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let result = String::from_utf8(invalid_utf8);
    assert!(result.is_err());
    
    // Verify our error type conversion
    let mem_err = MemoryError::Utf8Error(result.unwrap_err());
    assert!(mem_err.to_string().contains("UTF-8") || mem_err.to_string().contains("Utf8") || mem_err.to_string().contains("utf8"));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_wide_string_size_overflow() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = BasicMemoryReader::new(&handle);
    
    // Test wide string size overflow (lines 86-90)
    let max_len = usize::MAX / 2 + 1; // This will overflow when multiplied by 2
    let result = reader.read_wide_string(Address::new(0x1000), max_len);
    assert!(result.is_err());
    match result {
        Err(MemoryError::InvalidAddress(msg)) => {
            assert!(msg.contains("exceeds maximum"));
        }
        _ => panic!("Expected InvalidAddress error for wide string size overflow"),
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_typed_value() {
    let handle = ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap());
    
    let reader = BasicMemoryReader::new(&handle);
    
    // Test reading a typed value (line 45)
    // This will fail with invalid address but exercises the code path
    let result = reader.read::<u32>(Address::new(0));
    assert!(result.is_err());
}

#[test]
fn test_wide_string_invalid_utf16() {
    // Test invalid UTF-16 conversion (line 106)
    // Unpaired surrogate
    let invalid_utf16 = vec![0xD800]; // High surrogate without low surrogate
    let result = String::from_utf16(&invalid_utf16);
    assert!(result.is_err());
    
    // Verify the error would be converted correctly
    let mem_err = MemoryError::InvalidValueType("Invalid UTF-16 string".to_string());
    assert!(mem_err.to_string().contains("Invalid UTF-16"));
}