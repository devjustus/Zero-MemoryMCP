//! Integration tests for memory reading functionality

use memory_mcp::core::types::{Address, MemoryValue, ValueType};
use memory_mcp::memory::{BasicMemoryReader, Reader, SafeMemoryReader};
use memory_mcp::process::ProcessHandle;
use std::mem;

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_memory_reading() {
    // Use current process for testing
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Create a test variable
    let test_value: u32 = 0xDEADBEEF;
    let test_address = Address::new(&test_value as *const u32 as usize);

    // Read the value
    let result = reader.read::<u32>(test_address);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_safe_memory_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test with valid memory
    let test_value: i64 = -123456789;
    let test_address = Address::new(&test_value as *const i64 as usize);

    let result = reader.read::<i64>(test_address);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Test with invalid memory
    let invalid_address = Address::new(0);
    let result = reader.read::<u32>(invalid_address);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_array_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Create a test array
    let test_array: [u32; 5] = [1, 2, 3, 4, 5];
    let test_address = Address::new(test_array.as_ptr() as usize);

    // Read the array
    let result = reader.read_array::<u32>(test_address, 5);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_string_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Create a test string
    let test_string = b"Hello, World!\0";
    let test_address = Address::new(test_string.as_ptr() as usize);

    // Read the string
    let result = reader.read_string(test_address, 50);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, World!");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_batch_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Create test values
    let value1: u32 = 100;
    let value2: u32 = 200;
    let value3: u32 = 300;

    let addresses = vec![
        Address::new(&value1 as *const u32 as usize),
        Address::new(&value2 as *const u32 as usize),
        Address::new(&value3 as *const u32 as usize),
    ];

    // Read in batch
    let results = reader.read_batch::<u32>(&addresses);
    assert_eq!(results.len(), 3);

    assert!(results[0].is_ok());
    assert_eq!(results[0].as_ref().unwrap(), &100);

    assert!(results[1].is_ok());
    assert_eq!(results[1].as_ref().unwrap(), &200);

    assert!(results[2].is_ok());
    assert_eq!(results[2].as_ref().unwrap(), &300);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_value_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test different value types
    let u8_val: u8 = 42;
    let f32_val: f32 = 1.23456; // Arbitrary value
    let i32_val: i32 = -1234;

    // Read U8
    let addr = Address::new(&u8_val as *const u8 as usize);
    let result = reader.read_value(addr, ValueType::U8);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), MemoryValue::U8(42)));

    // Read F32
    let addr = Address::new(&f32_val as *const f32 as usize);
    let result = reader.read_value(addr, ValueType::F32);
    assert!(result.is_ok());
    if let MemoryValue::F32(val) = result.unwrap() {
        assert!((val - 1.23456).abs() < 0.00001);
    } else {
        panic!("Expected F32 value");
    }

    // Read I32
    let addr = Address::new(&i32_val as *const i32 as usize);
    let result = reader.read_value(addr, ValueType::I32);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), MemoryValue::I32(-1234)));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_unified_reader() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let mut reader = Reader::new(&handle);

    // Test safe reading
    let test_value: u64 = 0xCAFEBABEDEADBEEF;
    let test_address = Address::new(&test_value as *const u64 as usize);

    let result = reader.read_safe::<u64>(test_address);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Test cached reading
    let result = reader.read_cached::<u64>(test_address);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Clear cache
    reader.clear_cache();
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_validation() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test readable memory
    let valid_value: u32 = 12345;
    let valid_address = Address::new(&valid_value as *const u32 as usize);
    assert!(reader.is_readable(valid_address, mem::size_of::<u32>()));

    // Test unreadable memory
    assert!(!reader.is_readable(Address::new(0), 4));
    assert!(!reader.is_readable(Address::new(0xFFFFFFFFFFFFFFFF), 4));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_wide_string_reading() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Create a UTF-16 test string
    let test_string = "Test\0".encode_utf16().collect::<Vec<u16>>();
    let test_address = Address::new(test_string.as_ptr() as usize);

    // Read the wide string
    let result = reader.read_wide_string(test_address, 50);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Test");
}

#[test]
fn test_edge_cases() {
    // Test conversion logic without FFI

    // Empty string
    let empty = b"\0";
    let len = empty.iter().position(|&b| b == 0).unwrap_or(empty.len());
    let result = String::from_utf8(empty[..len].to_vec());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    // String without null terminator
    let no_null = b"NoNull";
    let len = no_null
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(no_null.len());
    let result = String::from_utf8(no_null[..len].to_vec());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "NoNull");
}
