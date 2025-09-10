//! Test error paths and edge cases for memory reader to maximize coverage

use memory_mcp::core::types::Address;
use memory_mcp::memory::reader::{BasicMemoryReader, SafeMemoryReader};
use memory_mcp::process::ProcessHandle;

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_invalid_handle_operations() {
    // Try to create reader with invalid handle
    match ProcessHandle::open_for_read(0xFFFFFFFF) {
        Ok(handle) => {
            let reader = BasicMemoryReader::new(&handle);

            // All operations should fail with invalid handle
            let result = reader.read::<u32>(Address::new(0x1000));
            assert!(result.is_err());

            let result = reader.read_raw(Address::new(0x1000), 10);
            assert!(result.is_err());

            let result = reader.read_string(Address::new(0x1000), 100);
            assert!(result.is_err());

            let result = reader.read_wide_string(Address::new(0x1000), 100);
            assert!(result.is_err());
        }
        Err(_) => {
            // Expected - invalid PID
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_boundary_conditions() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let basic_reader = BasicMemoryReader::new(&handle);
    let safe_reader = SafeMemoryReader::new(&handle);

    // Test with zero-sized reads
    let data: u32 = 12345;
    let addr = Address::new(&data as *const _ as usize);

    let result = basic_reader.read_raw(addr, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<u8>::new());

    let result = basic_reader.read_array::<u32>(addr, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<u32>::new());

    // Test with large (but not overflow-causing) reads
    let _ = basic_reader.read_string(addr, 10000);
    let _ = basic_reader.read_wide_string(addr, 5000);

    // Test batch with empty slice
    let empty_addrs: Vec<Address> = vec![];
    let results = basic_reader.read_batch::<u32>(&empty_addrs);
    assert_eq!(results.len(), 0);

    let results = safe_reader.read_batch::<u32>(&empty_addrs);
    assert_eq!(results.len(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_validation_error_messages() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test different validation failure scenarios
    let result = reader.validate_region(Address::new(0), 100);
    assert!(result.is_err());
    if let Err(e) = result {
        // Check that error message contains useful information
        let msg = format!("{}", e);
        assert!(msg.contains("0x") || msg.contains("not committed") || msg.contains("invalid"));
    }

    // Test very large region that likely exceeds available memory
    let some_data: u32 = 42;
    let addr = Address::new(&some_data as *const _ as usize);
    let result = reader.validate_region(addr, usize::MAX);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_string_reading_edge_cases() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // String with UTF-8 characters
    let utf8_string = "Hello 世界 🌍\0";
    let addr = Address::new(utf8_string.as_ptr() as usize);
    let result = reader.read_string(addr, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello 世界 🌍");

    // String that's exactly max_len (no null found)
    let exact_len = b"ABCD";
    let addr = Address::new(exact_len.as_ptr() as usize);
    let result = reader.read_string(addr, 4);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ABCD");

    // Wide string with no null terminator in range
    let wide_data: Vec<u16> = vec![0x41, 0x42, 0x43, 0x44]; // "ABCD" without null
    let addr = Address::new(wide_data.as_ptr() as usize);
    let result = reader.read_wide_string(addr, 4);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ABCD");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_array_reading_edge_cases() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Read array of different sizes
    let data: [u64; 5] = [1, 2, 3, 4, 5];
    let addr = Address::new(&data as *const _ as usize);

    // Read single element
    let result = reader.read_array::<u64>(addr, 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1]);

    // Read partial array
    let result = reader.read_array::<u64>(addr, 3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3]);

    // Read full array
    let result = reader.read_array::<u64>(addr, 5);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![1, 2, 3, 4, 5]);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_safe_reader_string_validation() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test string operations with validation
    let test_string = b"Valid String\0";
    let addr = Address::new(test_string.as_ptr() as usize);

    // Should validate at least first byte
    let result = reader.read_string(addr, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Valid String");

    // Test wide string validation
    let wide_string: Vec<u16> = "Wide Test\0".encode_utf16().collect();
    let addr = Address::new(wide_string.as_ptr() as usize);

    // Should validate at least first 2 bytes
    let result = reader.read_wide_string(addr, 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Wide Test");

    // Test with invalid address
    let result = reader.read_string(Address::new(0), 100);
    assert!(result.is_err());

    let result = reader.read_wide_string(Address::new(0), 100);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_batch_operations_mixed() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let basic_reader = BasicMemoryReader::new(&handle);
    let safe_reader = SafeMemoryReader::new(&handle);

    // Create test data
    let val1: f32 = 1.234;
    let val2: f64 = 5.678;
    let val3: i32 = -999;

    // Test batch with different types
    let addrs = vec![
        Address::new(&val1 as *const _ as usize),
        Address::new(&val2 as *const _ as usize),
        Address::new(&val3 as *const _ as usize),
    ];

    // Basic reader batch
    let f32_results = basic_reader.read_batch::<f32>(&addrs);
    assert_eq!(f32_results.len(), 3);
    assert!(f32_results[0].is_ok());

    // Safe reader batch
    let i32_results = safe_reader.read_batch::<i32>(&addrs);
    assert_eq!(i32_results.len(), 3);
    assert!(i32_results[2].is_ok());
}

#[test]
fn test_utf_conversion_failures() {
    use memory_mcp::core::types::MemoryError;

    // Test invalid UTF-8 bytes
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0xFC];
    let result = String::from_utf8(invalid_utf8.clone());
    assert!(result.is_err());

    // Convert the error to MemoryError
    let err = result.unwrap_err();
    let mem_err = MemoryError::Utf8Error(err);
    let err_str = format!("{}", mem_err);
    assert!(err_str.contains("UTF") || err_str.contains("utf"));

    // Test invalid UTF-16
    let invalid_utf16 = vec![0xD800]; // Unpaired high surrogate
    let result = String::from_utf16(&invalid_utf16);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_memory_reader_with_all_types() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = memory_mcp::memory::MemoryReader::new(&handle);

    // Test all primitive type reading through cached reader
    let bool_val: bool = true;
    let char_val: char = 'A';
    let usize_val: usize = 999999;
    let isize_val: isize = -888888;

    let result = reader.read::<bool>(Address::new(&bool_val as *const _ as usize));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let result = reader.read::<char>(Address::new(&char_val as *const _ as usize));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 'A');

    let result = reader.read::<usize>(Address::new(&usize_val as *const _ as usize));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 999999);

    let result = reader.read::<isize>(Address::new(&isize_val as *const _ as usize));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), -888888);
}
