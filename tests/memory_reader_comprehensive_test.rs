//! Comprehensive tests for memory reader to increase code coverage

use memory_mcp::core::types::{Address, MemoryValue, ValueType};
use memory_mcp::memory::reader::{BasicMemoryReader, SafeMemoryReader};
use memory_mcp::memory::{MemoryReader, Reader};
use memory_mcp::process::ProcessHandle;
use std::mem;

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_reader_all_methods() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Test read_raw with various sizes
    let test_data: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let addr = Address::new(&test_data as *const _ as usize);

    let result = reader.read_raw(addr, 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![0]);

    let result = reader.read_raw(addr, 8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![0, 1, 2, 3, 4, 5, 6, 7]);

    let result = reader.read_raw(addr, 16);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 16);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_reader_all_types() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Test all primitive types
    let u8_val: u8 = 255;
    let u16_val: u16 = 65535;
    let u32_val: u32 = 0xFFFFFFFF;
    let u64_val: u64 = 0xFFFFFFFFFFFFFFFF;
    let i8_val: i8 = -128;
    let i16_val: i16 = -32768;
    let i32_val: i32 = -2147483648;
    let i64_val: i64 = -9223372036854775808;
    let f32_val: f32 = -123.456;
    let f64_val: f64 = -123456.789012;

    assert_eq!(
        reader
            .read::<u8>(Address::new(&u8_val as *const _ as usize))
            .unwrap(),
        u8_val
    );
    assert_eq!(
        reader
            .read::<u16>(Address::new(&u16_val as *const _ as usize))
            .unwrap(),
        u16_val
    );
    assert_eq!(
        reader
            .read::<u32>(Address::new(&u32_val as *const _ as usize))
            .unwrap(),
        u32_val
    );
    assert_eq!(
        reader
            .read::<u64>(Address::new(&u64_val as *const _ as usize))
            .unwrap(),
        u64_val
    );
    assert_eq!(
        reader
            .read::<i8>(Address::new(&i8_val as *const _ as usize))
            .unwrap(),
        i8_val
    );
    assert_eq!(
        reader
            .read::<i16>(Address::new(&i16_val as *const _ as usize))
            .unwrap(),
        i16_val
    );
    assert_eq!(
        reader
            .read::<i32>(Address::new(&i32_val as *const _ as usize))
            .unwrap(),
        i32_val
    );
    assert_eq!(
        reader
            .read::<i64>(Address::new(&i64_val as *const _ as usize))
            .unwrap(),
        i64_val
    );

    let f32_result = reader
        .read::<f32>(Address::new(&f32_val as *const _ as usize))
        .unwrap();
    assert!((f32_result - f32_val).abs() < 0.001);

    let f64_result = reader
        .read::<f64>(Address::new(&f64_val as *const _ as usize))
        .unwrap();
    assert!((f64_result - f64_val).abs() < 0.001);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_reader_arrays() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Test different array sizes
    let array1: [u32; 1] = [42];
    let array10: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let array100: [u8; 100] = [5; 100];

    let result = reader.read_array::<u32>(Address::new(&array1 as *const _ as usize), 1);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![42]);

    let result = reader.read_array::<u32>(Address::new(&array10 as *const _ as usize), 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let result = reader.read_array::<u8>(Address::new(&array100 as *const _ as usize), 100);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![5; 100]);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_reader_strings() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Test various string scenarios
    let empty_string = b"\0";
    let short_string = b"Hi\0";
    let long_string = b"This is a much longer string for testing purposes\0";
    let no_null = b"NoNullTerminator";

    // Empty string
    let result = reader.read_string(Address::new(empty_string.as_ptr() as usize), 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    // Short string
    let result = reader.read_string(Address::new(short_string.as_ptr() as usize), 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hi");

    // Long string
    let result = reader.read_string(Address::new(long_string.as_ptr() as usize), 100);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "This is a much longer string for testing purposes"
    );

    // String with exact max_len (no null terminator found)
    let result = reader.read_string(Address::new(no_null.as_ptr() as usize), 16);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "NoNullTerminator");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_basic_reader_wide_strings() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = BasicMemoryReader::new(&handle);

    // Test UTF-16 strings
    let empty_wide: Vec<u16> = vec![0];
    let hello_wide: Vec<u16> = "Hello\0".encode_utf16().collect();
    let unicode_wide: Vec<u16> = "Hello 世界 🌍\0".encode_utf16().collect();

    // Empty wide string
    let result = reader.read_wide_string(Address::new(empty_wide.as_ptr() as usize), 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    // ASCII in UTF-16
    let result = reader.read_wide_string(Address::new(hello_wide.as_ptr() as usize), 50);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello");

    // Unicode characters
    let result = reader.read_wide_string(Address::new(unicode_wide.as_ptr() as usize), 50);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello 世界 🌍");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_safe_reader_validation() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test validation with valid memory
    let valid_data: u32 = 12345;
    let valid_addr = Address::new(&valid_data as *const _ as usize);

    let result = reader.validate_region(valid_addr, mem::size_of::<u32>());
    assert!(result.is_ok());

    // Test validation with invalid addresses
    let result = reader.validate_region(Address::new(0), 4);
    assert!(result.is_err());

    let result = reader.validate_region(Address::new(0x1000), 4);
    assert!(result.is_err());

    let result = reader.validate_region(Address::new(0xFFFFFFFF), 4);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_safe_reader_all_value_types() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test reading all MemoryValue types
    let u8_val: u8 = 200;
    let u16_val: u16 = 50000;
    let u32_val: u32 = 3000000000;
    let u64_val: u64 = 9000000000000000000;
    let i8_val: i8 = -100;
    let i16_val: i16 = -20000;
    let i32_val: i32 = -1500000000;
    let i64_val: i64 = -8000000000000000000;
    let f32_val: f32 = 999.999;
    let f64_val: f64 = 999999.999999;
    let string_val = b"TestString\0";

    // Test all value types
    assert!(matches!(
        reader
            .read_value(Address::new(&u8_val as *const _ as usize), ValueType::U8)
            .unwrap(),
        MemoryValue::U8(200)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&u16_val as *const _ as usize), ValueType::U16)
            .unwrap(),
        MemoryValue::U16(50000)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&u32_val as *const _ as usize), ValueType::U32)
            .unwrap(),
        MemoryValue::U32(3000000000)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&u64_val as *const _ as usize), ValueType::U64)
            .unwrap(),
        MemoryValue::U64(9000000000000000000)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&i8_val as *const _ as usize), ValueType::I8)
            .unwrap(),
        MemoryValue::I8(-100)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&i16_val as *const _ as usize), ValueType::I16)
            .unwrap(),
        MemoryValue::I16(-20000)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&i32_val as *const _ as usize), ValueType::I32)
            .unwrap(),
        MemoryValue::I32(-1500000000)
    ));

    assert!(matches!(
        reader
            .read_value(Address::new(&i64_val as *const _ as usize), ValueType::I64)
            .unwrap(),
        MemoryValue::I64(-8000000000000000000)
    ));

    if let MemoryValue::F32(val) = reader
        .read_value(Address::new(&f32_val as *const _ as usize), ValueType::F32)
        .unwrap()
    {
        assert!((val - 999.999).abs() < 0.01);
    } else {
        panic!("Expected F32");
    }

    if let MemoryValue::F64(val) = reader
        .read_value(Address::new(&f64_val as *const _ as usize), ValueType::F64)
        .unwrap()
    {
        assert!((val - 999999.999999).abs() < 0.01);
    } else {
        panic!("Expected F64");
    }

    if let MemoryValue::String(val) = reader
        .read_value(
            Address::new(string_val.as_ptr() as usize),
            ValueType::String,
        )
        .unwrap()
    {
        assert_eq!(val, "TestString");
    } else {
        panic!("Expected String");
    }

    // Test Bytes type
    let bytes_data: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    if let MemoryValue::Bytes(val) = reader
        .read_value(
            Address::new(&bytes_data as *const _ as usize),
            ValueType::Bytes,
        )
        .unwrap()
    {
        assert!(val.len() == 256); // Default size for Bytes
        assert_eq!(&val[..10], &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    } else {
        panic!("Expected Bytes");
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_safe_reader_batch() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let reader = SafeMemoryReader::new(&handle);

    // Test batch operations with mixed valid/invalid addresses
    let val1: u32 = 111;
    let val2: u32 = 222;
    let val3: u32 = 333;

    let addresses = vec![
        Address::new(&val1 as *const _ as usize),
        Address::new(0), // Invalid
        Address::new(&val2 as *const _ as usize),
        Address::new(0xDEADBEEF), // Invalid
        Address::new(&val3 as *const _ as usize),
    ];

    let results = reader.read_batch::<u32>(&addresses);
    assert_eq!(results.len(), 5);

    assert!(results[0].is_ok());
    assert_eq!(*results[0].as_ref().unwrap(), 111);

    assert!(results[1].is_err());

    assert!(results[2].is_ok());
    assert_eq!(*results[2].as_ref().unwrap(), 222);

    assert!(results[3].is_err());

    assert!(results[4].is_ok());
    assert_eq!(*results[4].as_ref().unwrap(), 333);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_unified_reader_all_operations() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let mut reader = Reader::new(&handle);

    // Test all unified reader methods
    let test_value: u64 = 0xDEADBEEFCAFEBABE;
    let test_addr = Address::new(&test_value as *const _ as usize);

    // Test safe reading
    let result = reader.read_safe::<u64>(test_addr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Test cached reading (should cache the value)
    let result = reader.read_cached::<u64>(test_addr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), test_value);

    // Test read_bytes
    let result = reader.read_bytes(test_addr, 8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 8);

    // Test read_value
    let result = reader.read_value(test_addr, ValueType::U64);
    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        MemoryValue::U64(0xDEADBEEFCAFEBABE)
    ));

    // Clear cache
    reader.clear_cache();
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_cache_operations() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let mut reader = MemoryReader::new(&handle);

    // Test cache operations
    let data1: u32 = 111;
    let data2: u64 = 222;
    let data3: [u8; 32] = [3; 32];

    let addr1 = Address::new(&data1 as *const _ as usize);
    let addr2 = Address::new(&data2 as *const _ as usize);
    let addr3 = Address::new(&data3 as *const _ as usize);

    // Initial cache should be empty
    assert_eq!(reader.cache_size(), 0);

    // Read and cache data
    let _ = reader.read_bytes(addr1, 4);
    assert!(reader.cache_size() > 0);

    let _ = reader.read_bytes(addr2, 8);
    assert!(reader.cache_size() > 1);

    let _ = reader.read_bytes(addr3, 32);
    assert!(reader.cache_size() > 2);

    // Clear cache
    reader.clear_cache();
    assert_eq!(reader.cache_size(), 0);

    // Test batch reading
    let addresses = vec![addr1, addr2];
    let results = reader.read_batch::<u32>(&addresses);
    assert_eq!(results.len(), 2);
    assert!(results[0].is_ok());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_error_scenarios() {
    let handle =
        ProcessHandle::open_for_read(std::process::id()).expect("Failed to open current process");

    let basic_reader = BasicMemoryReader::new(&handle);
    let safe_reader = SafeMemoryReader::new(&handle);

    // Test reading from invalid addresses
    let invalid_addrs = vec![
        Address::new(0),
        Address::new(0x1000),
        Address::new(0xDEADBEEF),
        Address::new(0xFFFFFFFFFFFFFFFF),
    ];

    for addr in &invalid_addrs {
        // Basic reader might succeed or fail depending on memory protection
        let _ = basic_reader.read::<u32>(*addr);

        // Safe reader should always fail validation
        assert!(safe_reader.read::<u32>(*addr).is_err());
        assert!(!safe_reader.is_readable(*addr, 4));
    }

    // Test string reading with small buffer that might not have null terminator
    let long_string = b"This is a very long string without null terminator in the read range";
    let addr = Address::new(long_string.as_ptr() as usize);

    // Should read up to max_len without error
    let result = basic_reader.read_string(addr, 10);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "This is a ");
}

#[test]
fn test_string_conversion_edge_cases() {
    // Test UTF-8 conversion edge cases
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let result = String::from_utf8(invalid_utf8.clone());
    assert!(result.is_err());

    // Test UTF-16 conversion edge cases
    let invalid_utf16 = vec![0xD800, 0x0000]; // Unpaired surrogate
    let result = String::from_utf16(&invalid_utf16);
    assert!(result.is_err());

    // Test empty conversions
    let empty_utf8: Vec<u8> = vec![];
    let result = String::from_utf8(empty_utf8);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    let empty_utf16: Vec<u16> = vec![];
    let result = String::from_utf16(&empty_utf16);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}
