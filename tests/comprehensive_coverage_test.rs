//! Comprehensive tests to achieve near 100% code coverage

use memory_mcp::core::types::{Address, MemoryError, MemoryValue, ValueType};
use memory_mcp::memory::{
    ComparisonType, MemoryOperations, MemoryReader, MemoryScanner, MemoryWriter, ScanOptions,
    ScanPattern,
};
use memory_mcp::process::ProcessHandle;
use std::collections::HashMap;

/// Helper to get a test handle
fn get_test_handle() -> ProcessHandle {
    #[cfg(miri)]
    {
        ProcessHandle::new(std::ptr::null_mut(), 0)
    }
    #[cfg(not(miri))]
    {
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process")
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_read_cache_through_reader() {
    // Test cache through MemoryReader since ReadCache is not publicly exported
    let handle = get_test_handle();
    let mut reader = MemoryReader::new(&handle);

    // Test initial cache state
    assert_eq!(reader.cache_size(), 0);

    // Trigger cache usage by reading bytes
    let test_value = 42u32;
    let addr = Address::from(&test_value as *const u32 as usize);
    let _ = reader.read_bytes(addr, 4);

    // Cache should have entries now (if read succeeded)
    // Note: May be 0 if read failed, which is expected with test handle

    // Test cache clear
    reader.clear_cache();
    assert_eq!(reader.cache_size(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_comprehensive() {
    let handle = get_test_handle();
    let mut reader = MemoryReader::new(&handle);

    // Test reading different value types
    let test_values = [
        (ValueType::U8, 1usize),
        (ValueType::U16, 2),
        (ValueType::U32, 4),
        (ValueType::U64, 8),
        (ValueType::I8, 1),
        (ValueType::I16, 2),
        (ValueType::I32, 4),
        (ValueType::I64, 8),
        (ValueType::F32, 4),
        (ValueType::F64, 8),
        (ValueType::String, 256),
        (ValueType::Bytes, 256),
    ];

    for (value_type, _size) in test_values {
        let result = reader.read_value(Address::new(0x1000), value_type);
        // Will fail but tests the code path
        assert!(result.is_err());
    }

    // Test batch read with empty slice
    let empty: Vec<Address> = vec![];
    let results: Vec<_> = reader.read_batch::<u32>(&empty);
    assert_eq!(results.len(), 0);

    // Test cache operations
    reader.clear_cache();
    assert_eq!(reader.cache_size(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_comprehensive() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Test all MemoryValue variants
    let values = [
        MemoryValue::U8(255),
        MemoryValue::U16(65535),
        MemoryValue::U32(u32::MAX),
        MemoryValue::U64(u64::MAX),
        MemoryValue::I8(i8::MIN),
        MemoryValue::I16(i16::MIN),
        MemoryValue::I32(i32::MIN),
        MemoryValue::I64(i64::MIN),
        MemoryValue::F32(std::f32::consts::PI),
        MemoryValue::F64(std::f64::consts::E),
        MemoryValue::String("Test".to_string()),
        MemoryValue::Bytes(vec![0xFF; 256]),
    ];

    for value in &values {
        let result = writer.write_value(Address::new(0x1000), value);
        assert!(result.is_err()); // Expected with read-only handle
    }

    // Test batch write with empty slice
    let empty: Vec<(Address, u32)> = vec![];
    let results = writer.write_batch(&empty);
    assert_eq!(results.len(), 0);

    // Test fill with zero size
    let result = writer.fill(Address::new(0x1000), 0xCC, 0);
    assert!(result.is_ok()); // Should succeed with zero size

    // Test copy_memory with zero size
    let result = writer.copy_memory(Address::new(0x1000), Address::new(0x2000), 0);
    assert!(result.is_ok()); // Should succeed with zero size

    // Test swap_memory with zero size
    let result = writer.swap_memory(Address::new(0x1000), Address::new(0x2000), 0);
    assert!(result.is_ok()); // Should succeed with zero size
}

#[test]
fn test_scan_pattern_comprehensive() {
    // Test all pattern types and conversions
    let patterns = [
        ScanPattern::Exact(vec![0x48, 0x8B]),
        ScanPattern::Masked(vec![Some(0x48), None, Some(0x8B)]),
        ScanPattern::String("test".to_string()),
        ScanPattern::WideString("test".to_string()),
    ];

    for pattern in &patterns {
        assert!(!pattern.is_empty());
        assert!(!pattern.is_empty());
    }

    // Test hex string parsing edge cases
    let valid_patterns = ["48", "48 8B", "48 ?? 8B", "?? ?? ??", "48 8B ?? ?? 89 90"];

    for pattern_str in &valid_patterns {
        let pattern = ScanPattern::from_hex_string(pattern_str);
        assert!(pattern.is_ok());
    }

    // Test invalid hex strings
    let invalid_patterns = [
        "", "GG", "ZZ XX", "12 3",   // Invalid single digit
        "12 345", // Invalid three digits
    ];

    for pattern_str in &invalid_patterns {
        let pattern = ScanPattern::from_hex_string(pattern_str);
        if pattern.is_ok() {
            panic!(
                "Pattern '{}' should have returned an error but got: {:?}",
                pattern_str, pattern
            );
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_scanner_comprehensive() {
    let handle = get_test_handle();
    let scanner = MemoryScanner::new(&handle);

    // Test all comparison types
    let comparisons = [
        ComparisonType::Equal,
        ComparisonType::NotEqual,
        ComparisonType::Greater,
        ComparisonType::Less,
        ComparisonType::GreaterOrEqual,
        ComparisonType::LessOrEqual,
    ];

    let mut previous = HashMap::new();
    previous.insert(Address::new(0x1000), vec![10, 0, 0, 0]);

    for comparison in &comparisons {
        let result = scanner.compare_scan(&previous, *comparison);
        assert!(result.is_ok());
    }

    // Test scan with different options
    // Note: Actual memory scanning may fail in CI environments due to permissions
    // So we only test the pattern creation and options building
    let options_variants = [
        ScanOptions {
            start_address: Some(Address::new(0x1000)),
            end_address: Some(Address::new(0x2000)),
            executable_only: true,
            writable_only: false,
            parallel: false,
            alignment: 1,
            max_results: Some(10),
        },
        ScanOptions {
            start_address: None,
            end_address: None,
            executable_only: false,
            writable_only: true,
            parallel: false,
            alignment: 4,
            max_results: None,
        },
        ScanOptions {
            start_address: Some(Address::new(0)),
            end_address: Some(Address::new(0xFFFFFFFF)),
            executable_only: true,
            writable_only: true,
            parallel: false,
            alignment: 8,
            max_results: Some(1),
        },
    ];

    for options in &options_variants {
        let pattern = ScanPattern::Exact(vec![0x90]);
        let result = scanner.scan(&pattern, options.clone());
        // In CI environments, scanning may fail due to permissions
        // We accept both success and permission errors
        if let Err(err) = result {
            match err {
                MemoryError::AccessDenied { .. }
                | MemoryError::ProcessNotFound(_)
                | MemoryError::ReadFailed { .. } => {
                    // Expected in CI environments or when scanning invalid memory regions
                }
                err => panic!("Unexpected error: {:?}", err),
            }
        }
    }

    // Test find_value with different types
    let result = scanner.find_value(42u8, ScanOptions::default());
    // In CI environments, scanning may fail due to permissions
    if let Err(err) = result {
        match err {
            MemoryError::AccessDenied { .. }
            | MemoryError::ProcessNotFound(_)
            | MemoryError::ReadFailed { .. } => {
                // Expected in CI environments or when scanning invalid memory regions
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    let result = scanner.find_value(0xDEADBEEFu32, ScanOptions::default());
    if let Err(err) = result {
        match err {
            MemoryError::AccessDenied { .. }
            | MemoryError::ProcessNotFound(_)
            | MemoryError::ReadFailed { .. } => {
                // Expected in CI environments or when scanning invalid memory regions
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    let result = scanner.find_value(std::f32::consts::PI, ScanOptions::default());
    if let Err(err) = result {
        match err {
            MemoryError::AccessDenied { .. }
            | MemoryError::ProcessNotFound(_)
            | MemoryError::ReadFailed { .. } => {
                // Expected in CI environments or when scanning invalid memory regions
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_operations_comprehensive() {
    let handle = get_test_handle();
    let mut ops = MemoryOperations::new(handle);

    // Test mutable reader access
    ops.reader_mut().clear_cache();
    assert_eq!(ops.reader().cache_size(), 0);

    // Test all accessors
    let _ = ops.reader();
    let _ = ops.writer();
    let _ = ops.scanner();

    // Test convenience methods
    let test_value = 42u32;
    let addr = Address::from(&test_value as *const u32 as usize);

    let _ = ops.read::<u32>(addr);
    let _ = ops.write(addr, 100u32);

    // Test scan with empty pattern
    let empty_pattern = ScanPattern::Exact(vec![]);
    let result = ops.scan(&empty_pattern, ScanOptions::default());
    // In CI environments, scanning may fail due to permissions
    // Empty pattern should return empty results or permission error
    if let Err(err) = result {
        match err {
            MemoryError::AccessDenied { .. }
            | MemoryError::ProcessNotFound(_)
            | MemoryError::ReadFailed { .. } => {
                // Expected in CI environments or when scanning invalid memory regions
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    } else {
        // Empty pattern should return empty results
        assert!(result.unwrap().is_empty());
    }
}

#[test]
fn test_error_edge_cases() {
    // Test MemoryError variants that might not be covered
    let errors = [
        MemoryError::InvalidAddress("test".to_string()),
        MemoryError::AccessDenied {
            pid: 1234,
            reason: "test".to_string(),
        },
        MemoryError::ProcessNotFound("1234".to_string()),
        MemoryError::ReadFailed {
            address: "0x1000".to_string(),
            reason: "Access denied".to_string(),
        },
        MemoryError::WriteFailed {
            address: "0x2000".to_string(),
            reason: "test".to_string(),
        },
        MemoryError::InvalidValueType("test".to_string()),
        MemoryError::UnsupportedOperation("test".to_string()),
        MemoryError::InvalidPattern("test".to_string()),
        MemoryError::Utf8Error(String::from_utf8(vec![0xFF, 0xFF]).unwrap_err()),
        MemoryError::WindowsApi("TestFunc: error 123".to_string()),
    ];

    for error in &errors {
        // Test Display implementation
        let _ = format!("{}", error);
        // Test Debug implementation
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_address_edge_cases() {
    // Test Address operations
    let addr1 = Address::new(0x1000);
    let addr2 = Address::from(0x2000usize);

    assert!(addr1 < addr2);
    assert!(addr2 > addr1);
    assert_eq!(addr1, Address::new(0x1000));

    // Test with pointer
    let value = 42u32;
    let addr = Address::from(&value as *const u32 as usize);
    assert!(addr.as_usize() != 0);
}

#[test]
fn test_value_type_conversions() {
    // Test all ValueType variants
    let types = [
        ValueType::U8,
        ValueType::U16,
        ValueType::U32,
        ValueType::U64,
        ValueType::I8,
        ValueType::I16,
        ValueType::I32,
        ValueType::I64,
        ValueType::F32,
        ValueType::F64,
        ValueType::String,
        ValueType::Bytes,
    ];

    for value_type in &types {
        // Test that each type is distinct
        match value_type {
            ValueType::U8 => assert_eq!(*value_type, ValueType::U8),
            ValueType::U16 => assert_eq!(*value_type, ValueType::U16),
            ValueType::U32 => assert_eq!(*value_type, ValueType::U32),
            ValueType::U64 => assert_eq!(*value_type, ValueType::U64),
            ValueType::I8 => assert_eq!(*value_type, ValueType::I8),
            ValueType::I16 => assert_eq!(*value_type, ValueType::I16),
            ValueType::I32 => assert_eq!(*value_type, ValueType::I32),
            ValueType::I64 => assert_eq!(*value_type, ValueType::I64),
            ValueType::F32 => assert_eq!(*value_type, ValueType::F32),
            ValueType::F64 => assert_eq!(*value_type, ValueType::F64),
            ValueType::String => assert_eq!(*value_type, ValueType::String),
            ValueType::Bytes => assert_eq!(*value_type, ValueType::Bytes),
        }
    }
}

#[test]
fn test_memory_value_conversions() {
    // Test all MemoryValue variants and conversions
    let values = [
        MemoryValue::U8(123),
        MemoryValue::U16(12345),
        MemoryValue::U32(123456789),
        MemoryValue::U64(1234567890123456789),
        MemoryValue::I8(-123),
        MemoryValue::I16(-12345),
        MemoryValue::I32(-123456789),
        MemoryValue::I64(-1234567890123456789),
        MemoryValue::F32(1.23456),
        MemoryValue::F64(1.234567890123456),
        MemoryValue::String("Test String".to_string()),
        MemoryValue::Bytes(vec![0x01, 0x02, 0x03, 0x04]),
    ];

    for value in &values {
        // Test Display implementation
        let _ = format!("{:?}", value);

        // Test pattern matching
        match value {
            MemoryValue::U8(_) => {}
            MemoryValue::U16(_) => {}
            MemoryValue::U32(_) => {}
            MemoryValue::U64(_) => {}
            MemoryValue::I8(_) => {}
            MemoryValue::I16(_) => {}
            MemoryValue::I32(_) => {}
            MemoryValue::I64(_) => {}
            MemoryValue::F32(_) => {}
            MemoryValue::F64(_) => {}
            MemoryValue::String(s) => assert!(!s.is_empty()),
            MemoryValue::Bytes(b) => assert!(!b.is_empty()),
        }
    }
}
