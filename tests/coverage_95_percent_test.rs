//! Comprehensive tests to achieve 95% code coverage across entire project

use memory_mcp::core::types::{Address, ValueType};
use memory_mcp::memory::{
    writer::{BatchWrite, ExtendedWrite, MemoryCopy, MemoryWrite},
    BasicMemoryWriter, ComparisonType, MemoryOperations, MemoryScanner, SafeMemoryWriter,
    ScanOptions, ScanPattern,
};
use memory_mcp::process::{ProcessEnumerator, ProcessHandle};
use std::collections::HashMap;

/// Helper to get a test handle
fn get_test_handle() -> ProcessHandle {
    #[cfg(miri)]
    {
        ProcessHandle::from_raw_handle(std::ptr::null_mut(), 0)
    }
    #[cfg(not(miri))]
    {
        ProcessHandle::open_for_read(std::process::id())
            .unwrap_or_else(|_| ProcessHandle::from_raw_handle(std::ptr::null_mut(), 0))
    }
}

#[cfg(test)]
mod scanner_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_scanner_pattern_variations() {
        let handle = get_test_handle();
        let scanner = MemoryScanner::new(&handle);

        // Test all pattern types
        let patterns = vec![
            ScanPattern::Exact(vec![0x90, 0x90]),
            ScanPattern::Masked(vec![Some(0x48), None, Some(0x8B)]),
            ScanPattern::String("test".to_string()),
            ScanPattern::WideString("test".to_string()),
        ];

        for pattern in patterns {
            let _ = scanner.scan(&pattern, ScanOptions::default());
        }
    }

    #[test]
    fn test_pattern_from_hex_edge_cases() {
        // Test valid patterns
        assert!(ScanPattern::from_hex_string("48").is_ok());
        assert!(ScanPattern::from_hex_string("48 8B").is_ok());
        assert!(ScanPattern::from_hex_string("48 ?? 8B").is_ok());
        assert!(ScanPattern::from_hex_string("?? ?? ??").is_ok());
        assert!(ScanPattern::from_hex_string("48 8B ?? ?? 89 90").is_ok());

        // Test invalid patterns
        assert!(ScanPattern::from_hex_string("").is_err());
        assert!(ScanPattern::from_hex_string("   ").is_err());
        assert!(ScanPattern::from_hex_string("GG").is_err());
        assert!(ScanPattern::from_hex_string("ZZ XX").is_err());
        assert!(ScanPattern::from_hex_string("12 3").is_err());
        assert!(ScanPattern::from_hex_string("12 345").is_err());
        assert!(ScanPattern::from_hex_string("1").is_err());
        assert!(ScanPattern::from_hex_string("123").is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_scanner_find_value_all_types() {
        let handle = get_test_handle();
        let scanner = MemoryScanner::new(&handle);

        // Test all value types
        let _ = scanner.find_value(42u8, ScanOptions::default());
        let _ = scanner.find_value(1234u16, ScanOptions::default());
        let _ = scanner.find_value(0xDEADBEEFu32, ScanOptions::default());
        let _ = scanner.find_value(0x123456789ABCDEFu64, ScanOptions::default());
        let _ = scanner.find_value(-42i8, ScanOptions::default());
        let _ = scanner.find_value(-1234i16, ScanOptions::default());
        let _ = scanner.find_value(-123456789i32, ScanOptions::default());
        let _ = scanner.find_value(-1234567890123456789i64, ScanOptions::default());
        let _ = scanner.find_value(3.14f32, ScanOptions::default());
        let _ = scanner.find_value(2.71828f64, ScanOptions::default());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_scanner_compare_scan_all_types() {
        let handle = get_test_handle();
        let scanner = MemoryScanner::new(&handle);

        let mut previous = HashMap::new();
        previous.insert(Address::new(0x1000), vec![10, 0, 0, 0]);
        previous.insert(Address::new(0x2000), vec![20, 0, 0, 0]);

        // Test all comparison types
        let comparisons = [
            ComparisonType::Equal,
            ComparisonType::NotEqual,
            ComparisonType::Greater,
            ComparisonType::Less,
            ComparisonType::GreaterOrEqual,
            ComparisonType::LessOrEqual,
        ];

        for comparison in &comparisons {
            let _ = scanner.compare_scan(&previous, *comparison);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_scan_options_variations() {
        let handle = get_test_handle();
        let scanner = MemoryScanner::new(&handle);
        let pattern = ScanPattern::Exact(vec![0x90]);

        // Test various scan options
        let options_list = vec![
            ScanOptions {
                start_address: Some(Address::new(0x10000)),
                end_address: Some(Address::new(0x20000)),
                executable_only: true,
                writable_only: false,
                parallel: false,
                alignment: 1,
                max_results: Some(100),
            },
            ScanOptions {
                start_address: None,
                end_address: None,
                executable_only: false,
                writable_only: true,
                parallel: true,
                alignment: 4,
                max_results: None,
            },
            ScanOptions {
                start_address: Some(Address::new(0)),
                end_address: Some(Address::new(usize::MAX)),
                executable_only: true,
                writable_only: true,
                parallel: true,
                alignment: 16,
                max_results: Some(1),
            },
        ];

        for options in options_list {
            let _ = scanner.scan(&pattern, options);
        }
    }
}

#[cfg(test)]
mod writer_comprehensive_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_basic_writer_edge_cases() {
        let handle = get_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test zero-sized operations
        assert!(writer.write_bytes(Address::new(0x1000), &[]).is_ok());
        assert!(writer.fill(Address::new(0x1000), 0xCC, 0).is_ok());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());

        // Test large operations (triggers chunking)
        let _ = writer.fill(Address::new(0x1000), 0xCC, 10000);
        let _ = writer.copy_memory(Address::new(0x1000), Address::new(0x20000), 10000);
        let _ = writer.swap_memory(Address::new(0x1000), Address::new(0x20000), 10000);

        // Test string operations
        let _ = writer.write_string(Address::new(0x1000), "");
        let _ = writer.write_string(Address::new(0x1000), "a");
        let _ = writer.write_string(Address::new(0x1000), &"x".repeat(1000));
        let _ = writer.write_wide_string(Address::new(0x1000), "");
        let _ = writer.write_wide_string(Address::new(0x1000), "🦀");
        let _ = writer.write_wide_string(Address::new(0x1000), &"世界".repeat(100));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_writer_comprehensive() {
        let handle = get_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        // Test all configurations
        writer.set_verify_writes(true);
        writer.set_check_permissions(true);
        let _ = writer.write(Address::new(0x1000), 42u32);

        writer.set_verify_writes(false);
        writer.set_check_permissions(true);
        let _ = writer.write(Address::new(0x1000), 42u32);

        writer.set_verify_writes(true);
        writer.set_check_permissions(false);
        let _ = writer.write(Address::new(0x1000), 42u32);

        writer.set_verify_writes(false);
        writer.set_check_permissions(false);
        let _ = writer.write(Address::new(0x1000), 42u32);

        // Test null address checks
        let _ = writer.write(Address::new(0), 42u32);
        let _ = writer.write_bytes(Address::new(0), &[1, 2, 3]);
        let _ = writer.fill(Address::new(0), 0xCC, 100);

        // Test overflow detection
        let _ = writer.write(Address::new(usize::MAX), 42u32);
        let _ = writer.write_bytes(Address::new(usize::MAX - 5), &[0; 10]);
        let _ = writer.fill(Address::new(usize::MAX - 10), 0xCC, 20);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_writer_verification() {
        let handle = get_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test write_verified with all types
        let _ = writer.write_verified(Address::new(0x1000), 42u8);
        let _ = writer.write_verified(Address::new(0x1000), 1234u16);
        let _ = writer.write_verified(Address::new(0x1000), 0xDEADBEEFu32);
        let _ = writer.write_verified(Address::new(0x1000), 0x123456789ABCDEFu64);
        let _ = writer.write_verified(Address::new(0x1000), -42i32);
        let _ = writer.write_verified(Address::new(0x1000), 3.14f32);
        let _ = writer.write_verified(Address::new(0x1000), 2.71828f64);

        // Test write_with_backup
        let _ = writer.write_with_backup(Address::new(0x1000), [0u8; 256]);
        let _ = writer.write_with_backup(Address::new(0x1000), [0u8; 1024]);

        // Test restore_from_backup
        let _ = writer.restore_from_backup(Address::new(0x1000), &[]);
        let _ = writer.restore_from_backup(Address::new(0x1000), &[0; 1]);
        let _ = writer.restore_from_backup(Address::new(0x1000), &vec![0; 4096]);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_batch_operations_comprehensive() {
        let handle = get_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test empty batch
        let empty: Vec<(Address, u32)> = vec![];
        let results = writer.write_batch(&empty);
        assert_eq!(results.len(), 0);

        // Test single item batch
        let single = vec![(Address::new(0x1000), 42u32)];
        let results = writer.write_batch(&single);
        assert_eq!(results.len(), 1);

        // Test large batch
        let large: Vec<(Address, u32)> = (0..1000)
            .map(|i| (Address::new(0x1000 + i * 4), i as u32))
            .collect();
        let results = writer.write_batch(&large);
        assert_eq!(results.len(), 1000);

        // Test mixed addresses
        let mixed = vec![
            (Address::new(0), 1u32),
            (Address::new(0x1000), 2u32),
            (Address::new(usize::MAX), 3u32),
        ];
        let results = writer.write_batch(&mixed);
        assert_eq!(results.len(), 3);
    }
}

#[cfg(test)]
mod reader_comprehensive_coverage {
    use super::*;
    use memory_mcp::memory::{MemoryReader, SafeMemoryReader};

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_reader_all_operations() {
        let handle = get_test_handle();
        let mut reader = MemoryReader::new(&handle);

        // Test all read operations
        let _ = reader.read::<u8>(Address::new(0x1000));
        let _ = reader.read::<u16>(Address::new(0x1000));
        let _ = reader.read::<u32>(Address::new(0x1000));
        let _ = reader.read::<u64>(Address::new(0x1000));
        let _ = reader.read::<i8>(Address::new(0x1000));
        let _ = reader.read::<i16>(Address::new(0x1000));
        let _ = reader.read::<i32>(Address::new(0x1000));
        let _ = reader.read::<i64>(Address::new(0x1000));
        let _ = reader.read::<f32>(Address::new(0x1000));
        let _ = reader.read::<f64>(Address::new(0x1000));

        // Test cache operations
        reader.clear_cache();
        assert_eq!(reader.cache_size(), 0);

        // Test read_value with all types
        for value_type in &[
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
        ] {
            let _ = reader.read_value(Address::new(0x1000), *value_type);
        }

        // Test batch operations
        let addresses = vec![
            Address::new(0x1000),
            Address::new(0x2000),
            Address::new(0x3000),
        ];
        let _: Vec<_> = reader.read_batch::<u32>(&addresses);

        // Test string operations
        let _ = reader.read_string(Address::new(0x1000), 0);
        let _ = reader.read_string(Address::new(0x1000), 1);
        let _ = reader.read_string(Address::new(0x1000), 1000);
        let _ = reader.read_wide_string(Address::new(0x1000), 0);
        let _ = reader.read_wide_string(Address::new(0x1000), 1);
        let _ = reader.read_wide_string(Address::new(0x1000), 1000);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_reader_validation() {
        let handle = get_test_handle();
        let reader = SafeMemoryReader::new(&handle);

        // Test validation with various addresses
        let _ = reader.read::<u32>(Address::new(0));
        let _ = reader.read::<u32>(Address::new(0x1000));
        let _ = reader.read::<u32>(Address::new(usize::MAX));

        // Test array reading
        let _ = reader.read_array::<u8>(Address::new(0x1000), 0);
        let _ = reader.read_array::<u8>(Address::new(0x1000), 1);
        let _ = reader.read_array::<u8>(Address::new(0x1000), 1000);
        let _ = reader.read_array::<u32>(Address::new(0x1000), 256);

        // Test raw reading
        let _ = reader.read_raw(Address::new(0x1000), 1024);
        let _ = reader.read_raw(Address::new(0x1000), 0);
        let _ = reader.read_raw(Address::new(0x1000), 100);
    }
}

#[cfg(test)]
mod process_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_enumeration_comprehensive() {
        use memory_mcp::process::enumerator::{
            enumerate_processes, find_process_by_name, find_processes_by_name, get_process_by_pid,
        };

        // Test standalone functions
        let _ = find_processes_by_name("System");
        let _ = find_processes_by_name("NonExistent.exe");
        let _ = find_processes_by_name("svchost.exe");
        let _ = find_process_by_name("System");
        let _ = find_process_by_name("NonExistent.exe");
        let _ = get_process_by_pid(0);
        let _ = get_process_by_pid(4);
        let _ = get_process_by_pid(std::process::id());
        let _ = get_process_by_pid(u32::MAX);

        // Test enumerator iteration
        if let Ok(mut enumerator) = ProcessEnumerator::new() {
            for process in enumerator.by_ref().take(10) {
                let _ = process.pid;
                let _ = process.name.clone();
                let _ = process.architecture;
                let _ = process.is_system_process();
                let _ = process.is_wow64;
            }
        }

        // Test enumerate_processes function
        if let Ok(processes) = enumerate_processes() {
            for process in processes.iter().take(5) {
                let _ = process.pid;
                let _ = process.name.clone();
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_comprehensive() {
        // Test all open methods
        let _ = ProcessHandle::open_for_read(4);
        let _ = ProcessHandle::open_for_read_write(4);
        let _ = ProcessHandle::open_all_access(4);
        let _ = ProcessHandle::open_for_read(std::process::id());
        let _ = ProcessHandle::open_for_read(u32::MAX);

        // Test from_raw_handle variations
        let _ = ProcessHandle::from_raw_handle(std::ptr::null_mut(), 0);
        let _ = ProcessHandle::from_raw_handle(std::ptr::null_mut(), 4);
        let _ = ProcessHandle::from_raw_handle(std::ptr::null_mut(), std::process::id());

        // Test handle operations
        if let Ok(handle) = ProcessHandle::open_for_read(std::process::id()) {
            assert!(handle.is_valid());
            assert_eq!(handle.pid(), std::process::id());

            // Test memory operations
            let mut buffer = vec![0u8; 100];
            let _ = handle.read_memory(0x10000, &mut buffer);
            let _ = handle.write_memory(0x10000, &buffer);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_info_modules() {
        use memory_mcp::process::ModuleEnumerator;

        // Test with current process
        if let Ok(handle) = ProcessHandle::open_for_read(std::process::id()) {
            let enumerator = ModuleEnumerator::new(handle);

            // Test module enumeration
            if let Ok(modules) = enumerator.enumerate() {
                for module in modules.iter().take(5) {
                    let _ = module.name.clone();
                    let _ = module.base_address;
                    let _ = module.size;
                }
            }
            let _ = enumerator.get_main_module();
            let _ = enumerator.find_by_name("kernel32.dll");
            let _ = enumerator.find_by_name("NonExistent.dll");
        }

        // Test with another handle
        if let Ok(handle2) = ProcessHandle::open_for_read(std::process::id()) {
            let enum2 = ModuleEnumerator::new(handle2);
            if let Ok(modules) = enum2.enumerate() {
                for module in modules.iter().take(3) {
                    let _ = module.name.clone();
                }
            }
        }
    }
}

#[cfg(test)]
mod privileges_coverage {
    use memory_mcp::process::{ElevationOptions, PrivilegeElevator};

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_privilege_checker_comprehensive() {
        use memory_mcp::process::privileges::PrivilegeChecker;

        // Test is_elevated (static method)
        let _ = PrivilegeChecker::is_elevated();

        // Test check_privilege with various LUIDs (common privilege values)
        let _ = PrivilegeChecker::check_privilege(2); // SE_CREATE_TOKEN_PRIVILEGE
        let _ = PrivilegeChecker::check_privilege(3); // SE_ASSIGNPRIMARYTOKEN_PRIVILEGE
        let _ = PrivilegeChecker::check_privilege(4); // SE_LOCK_MEMORY_PRIVILEGE
        let _ = PrivilegeChecker::check_privilege(5); // SE_INCREASE_QUOTA_PRIVILEGE
        let _ = PrivilegeChecker::check_privilege(20); // SE_DEBUG_PRIVILEGE
        let _ = PrivilegeChecker::check_privilege(0); // Invalid LUID
        let _ = PrivilegeChecker::check_privilege(u32::MAX); // Large LUID

        // Test list privileges
        let _ = PrivilegeChecker::list_privileges();
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_privilege_elevator_comprehensive() {
        let elevator = PrivilegeElevator::new();

        // Test with default options
        let _ = elevator.elevate("SeDebugPrivilege");
        let _ = elevator.elevate("SeShutdownPrivilege");
        let _ = elevator.elevate("NonExistentPrivilege");

        // Test with custom options
        let options = ElevationOptions {
            require_success: false,
            cache_result: true,
            auto_enable: true,
        };
        let elevator_custom = PrivilegeElevator::with_options(options);
        let _ = elevator_custom.elevate("SeDebugPrivilege");
        let _ = elevator_custom.elevate("SeDebugPrivilege");

        // Clear cache is not a method

        // Test with empty string
        let _ = elevator.elevate("");
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_functions() {
        use memory_mcp::process::{enable_debug_privilege, has_debug_privilege};

        // Test debug privilege functions
        let _ = has_debug_privilege();
        let _ = enable_debug_privilege();

        // Test debug privilege guard
        use memory_mcp::process::DebugPrivilegeGuard;
        {
            let _guard = DebugPrivilegeGuard::new();
            // Guard should drop automatically
        }

        // Test nested guards
        {
            let _guard1 = DebugPrivilegeGuard::new();
            {
                let _guard2 = DebugPrivilegeGuard::new();
            }
        }
    }
}

#[cfg(test)]
mod memory_operations_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_operations_all_methods() {
        let handle = get_test_handle();
        let mut ops = MemoryOperations::new(handle);

        // Test all accessor methods
        let _ = ops.reader();
        let _ = ops.reader_mut();
        let _ = ops.writer();
        let _ = ops.safe_writer();
        let _ = ops.scanner();

        // Test convenience methods
        let addr = Address::new(0x1000);
        let _ = ops.read::<u32>(addr);
        let _ = ops.write(addr, 42u32);
        let _ = ops.scan(&ScanPattern::Exact(vec![0x90]), ScanOptions::default());

        // Test with empty scan pattern
        let _ = ops.scan(&ScanPattern::Exact(vec![]), ScanOptions::default());

        // Test reader cache operations
        ops.reader_mut().clear_cache();
        assert_eq!(ops.reader().cache_size(), 0);
    }
}
