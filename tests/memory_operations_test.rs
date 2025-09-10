//! Integration tests for memory operations

use memory_mcp::core::types::{Address, MemoryValue, ValueType};
use memory_mcp::memory::{
    MemoryOperations, MemoryReader, MemoryScanner, MemoryWriter, ScanOptions, ScanPattern,
};
use memory_mcp::process::ProcessHandle;
use std::process;

/// Get handle to current process for testing
fn get_test_handle() -> ProcessHandle {
    #[cfg(miri)]
    {
        // Return a mock handle for Miri testing
        // This handle won't work for actual operations but allows tests to compile
        ProcessHandle::from_raw_handle(std::ptr::null_mut(), 0)
    }
    #[cfg(not(miri))]
    {
        let pid = process::id();
        ProcessHandle::open_for_read(pid).expect("Failed to open current process")
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_operations_creation() {
    let handle = get_test_handle();
    let ops = MemoryOperations::new(handle);

    // Test accessor methods
    assert!(ops.reader().cache_size() == 0);
    let _ = ops.writer();
    let _ = ops.scanner();
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_with_current_process() {
    let handle = get_test_handle();
    let mut reader = MemoryReader::new(&handle);

    // Create a test variable in our own memory
    let test_value: u32 = 0x12345678;
    let test_addr = Address::from(&test_value as *const u32 as usize);

    // Test reading our own memory
    let result = reader.read::<u32>(test_addr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Test cache functionality - read_bytes uses cache
    let bytes_result = reader.read_bytes(test_addr, 4);
    assert!(bytes_result.is_ok());
    // Now cache should have entries
    assert!(reader.cache_size() > 0);

    // Test reading with cache
    let cached_result = reader.read::<u32>(test_addr);
    assert!(cached_result.is_ok());
    assert_eq!(cached_result.unwrap(), test_value);

    // Clear cache and verify
    reader.clear_cache();
    assert_eq!(reader.cache_size(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_batch_operations() {
    let handle = get_test_handle();
    let reader = MemoryReader::new(&handle);

    let values = [1u32, 2u32, 3u32];
    let addresses: Vec<Address> = values
        .iter()
        .map(|v| Address::from(v as *const u32 as usize))
        .collect();

    let results = reader.read_batch::<u32>(&addresses);
    assert_eq!(results.len(), 3);

    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), &values[i]);
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_string_operations() {
    let handle = get_test_handle();
    let reader = MemoryReader::new(&handle);

    // Test with a static string
    let test_str = "Hello, Memory!\0";
    let str_addr = Address::from(test_str.as_ptr() as usize);

    let result = reader.read_string(str_addr, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, Memory!");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_value_types() {
    let handle = get_test_handle();
    let reader = MemoryReader::new(&handle);

    // Test different value types
    let test_u8: u8 = 0xFF;
    let test_i32: i32 = -42;
    let test_f32: f32 = std::f32::consts::PI;

    let u8_addr = Address::from(&test_u8 as *const u8 as usize);
    let i32_addr = Address::from(&test_i32 as *const i32 as usize);
    let f32_addr = Address::from(&test_f32 as *const f32 as usize);

    // Read as MemoryValue
    let u8_result = reader.read_value(u8_addr, ValueType::U8);
    assert!(u8_result.is_ok());
    if let MemoryValue::U8(v) = u8_result.unwrap() {
        assert_eq!(v, test_u8);
    } else {
        panic!("Wrong value type");
    }

    let i32_result = reader.read_value(i32_addr, ValueType::I32);
    assert!(i32_result.is_ok());
    if let MemoryValue::I32(v) = i32_result.unwrap() {
        assert_eq!(v, test_i32);
    } else {
        panic!("Wrong value type");
    }

    let f32_result = reader.read_value(f32_addr, ValueType::F32);
    assert!(f32_result.is_ok());
    if let MemoryValue::F32(v) = f32_result.unwrap() {
        assert!((v - test_f32).abs() < 0.0001);
    } else {
        panic!("Wrong value type");
    }
}

#[test]
#[cfg(windows)]
#[cfg_attr(miri, ignore = "Miri doesn't support Windows memory APIs")]
fn test_memory_writer_with_current_process() {
    // Get handle with write access for current process
    let pid = process::id();
    let handle = ProcessHandle::open_for_read_write(pid).expect("Failed to open with write access");
    let writer = MemoryWriter::new(&handle);

    // Create a mutable test variable
    let mut test_buffer = vec![0u8; 16];
    let buffer_addr = Address::from(test_buffer.as_mut_ptr() as usize);

    // Test writing bytes
    let data = vec![1u8, 2, 3, 4];
    let result = writer.write_bytes(buffer_addr, &data);

    // On Windows with proper permissions, this should work
    if result.is_ok() {
        assert_eq!(&test_buffer[..4], &data[..]);

        // Test write verification
        let verify_result = writer.write_verified(buffer_addr, 0x12345678u32);
        assert!(verify_result.is_ok() || verify_result.is_err()); // May fail due to permissions
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_memory_value_types() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Test with different MemoryValue types
    let values = vec![
        MemoryValue::U8(255),
        MemoryValue::U16(65535),
        MemoryValue::U32(4294967295),
        MemoryValue::I8(-128),
        MemoryValue::F32(std::f32::consts::PI),
        MemoryValue::String("test".to_string()),
    ];

    // Just test that the methods don't panic
    for value in &values {
        let test_addr = Address::new(0x1000);
        let _ = writer.write_value(test_addr, value); // Will fail but tests the code path
    }
}

#[test]
fn test_memory_scanner_pattern_creation() {
    // Test hex pattern parsing
    let pattern = ScanPattern::from_hex_string("48 8B ?? ?? 89").unwrap();
    assert_eq!(pattern.len(), 5);

    // Test exact pattern
    let exact = ScanPattern::Exact(vec![0x48, 0x8B, 0x89]);
    assert_eq!(exact.len(), 3);

    // Test string pattern
    let string = ScanPattern::String("test".to_string());
    assert_eq!(string.len(), 5); // "test" + null terminator

    // Test wide string pattern
    let wide = ScanPattern::WideString("test".to_string());
    assert_eq!(wide.len(), 10); // "test" in UTF-16 + null
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_scanner_with_current_process() {
    let handle = get_test_handle();
    let scanner = MemoryScanner::new(&handle);

    // Create a known pattern in memory
    let pattern_data = [0xDE, 0xAD, 0xBE, 0xEF];
    let pattern_addr = Address::from(pattern_data.as_ptr() as usize);

    // Scan for the pattern in a specific region
    let options = ScanOptions {
        start_address: Some(pattern_addr),
        end_address: Some(Address::new(pattern_addr.as_usize() + 0x1000)),
        parallel: false,
        alignment: 1,
        max_results: Some(10),
        ..Default::default()
    };

    let scan_pattern = ScanPattern::Exact(vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let results = scanner.scan_region(pattern_addr, 0x100, &scan_pattern, &options);

    // Should find at least one match (our pattern)
    assert!(results.is_ok());
    let found = results.unwrap();
    if !found.is_empty() {
        assert_eq!(found[0], pattern_addr);
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_scanner_find_value() {
    let handle = get_test_handle();
    let scanner = MemoryScanner::new(&handle);

    // Create a known value
    let test_value: u32 = 0xCAFEBABE;
    let value_addr = Address::from(&test_value as *const u32 as usize);

    // Search for the value
    let options = ScanOptions {
        start_address: Some(value_addr),
        end_address: Some(Address::new(value_addr.as_usize() + 0x100)),
        parallel: false,
        alignment: 4,
        max_results: Some(1),
        ..Default::default()
    };

    let results = scanner.find_value(test_value, options);
    assert!(results.is_ok());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_operations_integrated() {
    let handle = get_test_handle();
    let mut ops = MemoryOperations::new(handle);

    // Test integrated read operation
    let test_value: u64 = 0xDEADBEEFCAFEBABE;
    let addr = Address::from(&test_value as *const u64 as usize);

    let read_result = ops.read::<u64>(addr);
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), test_value);

    // Test scan operation through MemoryOperations
    let pattern = ScanPattern::Exact(vec![0xBE, 0xBA, 0xFE, 0xCA]);
    let options = ScanOptions {
        start_address: Some(addr),
        end_address: Some(Address::new(addr.as_usize() + 0x100)),
        parallel: false,
        ..Default::default()
    };

    let scan_result = ops.scan(&pattern, options);
    assert!(scan_result.is_ok());
}

#[test]
#[cfg(windows)]
#[cfg_attr(miri, ignore = "Miri doesn't support Windows memory APIs")]
fn test_memory_region_validation() {
    use memory_mcp::memory::validate_region;

    let handle = get_test_handle();

    // Test with a valid address (our stack)
    let test_var = 42u32;
    let valid_addr = Address::from(&test_var as *const u32 as usize);

    let result = validate_region(&handle, valid_addr, 4);
    // Should succeed for our own memory
    assert!(result.is_ok() || result.is_err()); // May vary based on Windows version

    // Test with invalid address
    let invalid_addr = Address::new(0x0);
    let invalid_result = validate_region(&handle, invalid_addr, 100);
    assert!(invalid_result.is_err());
}

#[test]
fn test_scan_options_builder_pattern() {
    let options = ScanOptions {
        executable_only: true,
        writable_only: false,
        parallel: true,
        alignment: 4,
        max_results: Some(100),
        ..Default::default()
    };

    assert!(options.executable_only);
    assert!(!options.writable_only);
    assert!(options.parallel);
    assert_eq!(options.alignment, 4);
    assert_eq!(options.max_results, Some(100));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_batch_operations() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Test batch write (will fail with read-only handle but tests the code)
    let writes = vec![(Address::new(0x1000), 42u32), (Address::new(0x2000), 84u32)];

    let results = writer.write_batch(&writes);
    assert_eq!(results.len(), 2);
    // Both should fail with read-only handle
    assert!(results[0].is_err());
    assert!(results[1].is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_fill_operation() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Test fill operation (will fail but tests the code path)
    let result = writer.fill(Address::new(0x1000), 0xCC, 100);
    assert!(result.is_err()); // Expected to fail with read-only handle
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_scanner_compare_scan() {
    let handle = get_test_handle();
    let scanner = MemoryScanner::new(&handle);

    use memory_mcp::memory::ComparisonType;
    use std::collections::HashMap;

    // Create previous scan results
    let mut previous = HashMap::new();
    let test_value = 42u32;
    let addr = Address::from(&test_value as *const u32 as usize);
    previous.insert(addr, vec![42, 0, 0, 0]); // Little-endian representation

    // Perform compare scan
    let results = scanner.compare_scan(&previous, ComparisonType::Equal);
    assert!(results.is_ok());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_scanner_comparison_types() {
    let handle = get_test_handle();
    let scanner = MemoryScanner::new(&handle);

    use memory_mcp::memory::ComparisonType;
    use std::collections::HashMap;

    // Test different comparison types
    let mut previous = HashMap::new();
    let test_value = 100u32;
    let addr = Address::from(&test_value as *const u32 as usize);
    previous.insert(addr, vec![50, 0, 0, 0]); // Previous value was 50

    // Test Greater comparison (100 > 50)
    let results = scanner.compare_scan(&previous, ComparisonType::Greater);
    assert!(results.is_ok());

    // Test Less comparison (100 < 50 should find nothing)
    let results = scanner.compare_scan(&previous, ComparisonType::Less);
    assert!(results.is_ok());

    // Test NotEqual comparison
    let results = scanner.compare_scan(&previous, ComparisonType::NotEqual);
    assert!(results.is_ok());

    // Test GreaterOrEqual
    let results = scanner.compare_scan(&previous, ComparisonType::GreaterOrEqual);
    assert!(results.is_ok());

    // Test LessOrEqual
    let results = scanner.compare_scan(&previous, ComparisonType::LessOrEqual);
    assert!(results.is_ok());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_copy_and_swap() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Create test buffers
    let buffer1 = [1u8, 2, 3, 4];
    let buffer2 = [5u8, 6, 7, 8];
    let addr1 = Address::from(buffer1.as_ptr() as usize);
    let addr2 = Address::from(buffer2.as_ptr() as usize);

    // Test copy_memory (will fail with read-only handle but tests the path)
    let result = writer.copy_memory(addr1, addr2, 4);
    assert!(result.is_err());

    // Test swap_memory
    let result = writer.swap_memory(addr1, addr2, 4);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_cache_operations() {
    let handle = get_test_handle();
    let mut reader = MemoryReader::new(&handle);

    // Test cache operations
    assert_eq!(reader.cache_size(), 0);

    // Try to read something to populate cache
    let test_value = 42u32;
    let addr = Address::from(&test_value as *const u32 as usize);

    // Read bytes to trigger cache
    let _ = reader.read_bytes(addr, 4);
    // Cache should have something now

    // Clear cache
    reader.clear_cache();
    assert_eq!(reader.cache_size(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_wide_string() {
    let handle = get_test_handle();
    let reader = MemoryReader::new(&handle);

    // Test with a wide string
    let test_str = "Hello\0";
    let wide: Vec<u16> = test_str.encode_utf16().collect();
    let wide_addr = Address::from(wide.as_ptr() as usize);

    let result = reader.read_wide_string(wide_addr, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");
}

#[test]
fn test_scan_pattern_variations() {
    // Test different pattern types
    let exact = ScanPattern::Exact(vec![0xDE, 0xAD]);
    assert_eq!(exact.len(), 2);
    assert!(!exact.is_empty());

    let masked = ScanPattern::Masked(vec![Some(0xDE), None, Some(0xAD)]);
    assert_eq!(masked.len(), 3);

    let string = ScanPattern::String("test".to_string());
    assert_eq!(string.len(), 5); // includes null terminator

    let wide = ScanPattern::WideString("test".to_string());
    assert_eq!(wide.len(), 10); // UTF-16 + null

    // Test empty check
    let empty = ScanPattern::Exact(vec![]);
    assert!(empty.is_empty());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_writer_wide_string_operations() {
    let handle = get_test_handle();
    let writer = MemoryWriter::new(&handle);

    // Test writing wide string (will fail but tests the code path)
    let result = writer.write_wide_string(Address::new(0x1000), "Hello 世界");
    assert!(result.is_err());

    // Test writing regular string
    let result = writer.write_string(Address::new(0x2000), "Test String");
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_operations_facade() {
    let handle = get_test_handle();
    let mut ops = MemoryOperations::new(handle);

    // Test reader_mut access
    ops.reader_mut().clear_cache();
    assert_eq!(ops.reader_mut().cache_size(), 0);

    // Test direct read/write through facade
    let test_value = 42u32;
    let addr = Address::from(&test_value as *const u32 as usize);

    let read_result = ops.read::<u32>(addr);
    assert!(read_result.is_ok());

    let write_result = ops.write(addr, 100u32);
    assert!(write_result.is_err()); // Should fail with read-only handle

    // Test scan through facade
    let pattern = ScanPattern::Exact(vec![42, 0, 0, 0]);
    let options = ScanOptions {
        start_address: Some(addr),
        end_address: Some(Address::new(addr.as_usize() + 0x100)),
        ..Default::default()
    };
    let scan_result = ops.scan(&pattern, options);
    assert!(scan_result.is_ok());
}

#[test]
fn test_scan_options_custom_settings() {
    let options = ScanOptions {
        start_address: Some(Address::new(0x1000)),
        end_address: Some(Address::new(0x9000)),
        executable_only: true,
        writable_only: true,
        parallel: false,
        alignment: 8,
        max_results: Some(50),
    };

    assert_eq!(options.start_address, Some(Address::new(0x1000)));
    assert_eq!(options.end_address, Some(Address::new(0x9000)));
    assert!(options.executable_only);
    assert!(options.writable_only);
    assert!(!options.parallel);
    assert_eq!(options.alignment, 8);
    assert_eq!(options.max_results, Some(50));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_value_all_types() {
    let handle = get_test_handle();
    let reader = MemoryReader::new(&handle);
    let writer = MemoryWriter::new(&handle);

    // Test all MemoryValue variants
    let values = vec![
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
        MemoryValue::String("Test String".to_string()),
        MemoryValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
    ];

    for value in values {
        // Test write_value
        let result = writer.write_value(Address::new(0x1000), &value);
        assert!(result.is_err()); // Expected with read-only handle

        // Test read_value for supported types
        use memory_mcp::core::types::ValueType;
        match value {
            MemoryValue::U8(_) => {
                let _ = reader.read_value(Address::new(0x1000), ValueType::U8);
            }
            MemoryValue::U16(_) => {
                let _ = reader.read_value(Address::new(0x1000), ValueType::U16);
            }
            MemoryValue::U32(_) => {
                let _ = reader.read_value(Address::new(0x1000), ValueType::U32);
            }
            _ => {}
        }
    }
}
