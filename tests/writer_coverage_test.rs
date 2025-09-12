//! Additional tests to improve code coverage for memory writer modules

use memory_mcp::core::types::{Address, MemoryValue};
use memory_mcp::memory::{
    create_safe_writer, create_writer,
    writer::{BatchWrite, ExtendedWrite, MemoryCopy, MemoryWrite},
    BasicMemoryWriter, SafeMemoryWriter,
};
use memory_mcp::process::ProcessHandle;

fn create_test_handle() -> ProcessHandle {
    ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap())
}

#[cfg(test)]
mod writer_factory_tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_create_writer_factory() {
        let handle = create_test_handle();
        let writer = create_writer(&handle);

        // Test basic write operation
        let result = writer.write(Address::new(0x1000), 42u32);
        assert!(result.is_err()); // Expected to fail with invalid address
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_create_safe_writer_factory() {
        let handle = create_test_handle();
        let writer = create_safe_writer(&handle);

        // Test safe write operation
        let result = writer.write(Address::new(0x1000), 42u32);
        assert!(result.is_err()); // Expected to fail with invalid address
    }
}

#[cfg(test)]
mod basic_writer_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_basic_writer_handle_access() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test handle() method
        let retrieved_handle = writer.handle();
        assert_eq!(retrieved_handle.pid(), handle.pid());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_all_memory_value_variants() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);
        let addr = Address::new(0x1000);

        // Test all MemoryValue variants
        assert!(writer.write_value(addr, &MemoryValue::U8(1)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::U16(2)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::U32(3)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::U64(4)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::I8(-1)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::I16(-2)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::I32(-3)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::I64(-4)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::F32(1.0)).is_err());
        assert!(writer.write_value(addr, &MemoryValue::F64(2.0)).is_err());
        assert!(writer
            .write_value(addr, &MemoryValue::String("test".to_string()))
            .is_err());
        assert!(writer
            .write_value(addr, &MemoryValue::Bytes(vec![1, 2, 3]))
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_fill_with_large_chunks() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test fill with size larger than chunk size (4096)
        let result = writer.fill(Address::new(0x1000), 0xAA, 10000);
        assert!(result.is_err());

        // Test fill with exact chunk size
        let result = writer.fill(Address::new(0x1000), 0xBB, 4096);
        assert!(result.is_err());

        // Test fill with multiple chunks
        let result = writer.fill(Address::new(0x1000), 0xCC, 8192);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_copy_memory_chunking() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test copy with size larger than chunk size (8192)
        let result = writer.copy_memory(Address::new(0x1000), Address::new(0x5000), 20000);
        assert!(result.is_err());

        // Test copy with exact chunk size
        let result = writer.copy_memory(Address::new(0x1000), Address::new(0x5000), 8192);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_swap_memory_various_sizes() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test swap with different sizes
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 1)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 100)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 1000)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_batch_write_empty() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        let empty: Vec<(Address, u32)> = vec![];
        let results = writer.write_batch(&empty);
        assert_eq!(results.len(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_string_various_lengths() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test strings of various lengths
        assert!(writer.write_string(Address::new(0x1000), "").is_err());
        assert!(writer.write_string(Address::new(0x1000), "a").is_err());
        assert!(writer.write_string(Address::new(0x1000), "ab").is_err());
        assert!(writer
            .write_string(Address::new(0x1000), &"x".repeat(1000))
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_wide_string_unicode() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test various Unicode strings
        assert!(writer.write_wide_string(Address::new(0x1000), "").is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "Hello")
            .is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "世界")
            .is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "🎨🎭🎪")
            .is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "Mixed 混合 Text")
            .is_err());
    }
}

#[cfg(test)]
mod safe_writer_coverage {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_writer_configuration() {
        let handle = create_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        // Test all configuration combinations
        writer.set_verify_writes(true);
        writer.set_check_permissions(true);
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());

        writer.set_verify_writes(false);
        writer.set_check_permissions(true);
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());

        writer.set_verify_writes(true);
        writer.set_check_permissions(false);
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());

        writer.set_verify_writes(false);
        writer.set_check_permissions(false);
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_verified_different_types() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test write_verified with different types
        assert!(writer.write_verified(Address::new(0x1000), 42u8).is_err());
        assert!(writer.write_verified(Address::new(0x1000), 42u16).is_err());
        assert!(writer.write_verified(Address::new(0x1000), 42u32).is_err());
        assert!(writer.write_verified(Address::new(0x1000), 42u64).is_err());
        assert!(writer.write_verified(Address::new(0x1000), 42i32).is_err());
        assert!(writer
            .write_verified(Address::new(0x1000), std::f32::consts::PI)
            .is_err());
        assert!(writer
            .write_verified(Address::new(0x1000), std::f64::consts::E)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_with_backup_types() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test backup with different types
        assert!(writer
            .write_with_backup(Address::new(0x1000), 42u8)
            .is_err());
        assert!(writer
            .write_with_backup(Address::new(0x1000), 42u16)
            .is_err());
        assert!(writer
            .write_with_backup(Address::new(0x1000), 42u32)
            .is_err());
        assert!(writer
            .write_with_backup(Address::new(0x1000), 42u64)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_restore_backup_various_sizes() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test restore with different backup sizes
        assert!(writer
            .restore_from_backup(Address::new(0x1000), &[1])
            .is_err());
        assert!(writer
            .restore_from_backup(Address::new(0x1000), &[1, 2])
            .is_err());
        assert!(writer
            .restore_from_backup(Address::new(0x1000), &[1, 2, 3, 4])
            .is_err());
        assert!(writer
            .restore_from_backup(Address::new(0x1000), &[0; 100])
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_write_bytes_with_verification() {
        let handle = create_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        // Test write_bytes with verification enabled
        writer.set_verify_writes(true);
        let data = vec![1, 2, 3, 4, 5];
        assert!(writer.write_bytes(Address::new(0x1000), &data).is_err());

        // Test with empty data and verification
        assert!(writer.write_bytes(Address::new(0x1000), &[]).is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_write_value_all_types() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test write_value with all MemoryValue types
        let values = vec![
            MemoryValue::U8(255),
            MemoryValue::U16(65535),
            MemoryValue::U32(0xFFFFFFFF),
            MemoryValue::U64(0xFFFFFFFFFFFFFFFF),
            MemoryValue::I8(-128),
            MemoryValue::I16(-32768),
            MemoryValue::I32(i32::MIN),
            MemoryValue::I64(i64::MIN),
            MemoryValue::F32(std::f32::consts::PI),
            MemoryValue::F64(std::f64::consts::E),
            MemoryValue::String("Test String".to_string()),
            MemoryValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ];

        for value in values {
            assert!(writer.write_value(Address::new(0x1000), &value).is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_string_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test string operations with various lengths
        assert!(writer.write_string(Address::new(0x1000), "").is_err());
        assert!(writer.write_string(Address::new(0x1000), "short").is_err());
        assert!(writer
            .write_string(Address::new(0x1000), &"x".repeat(1000))
            .is_err());

        // Test wide string operations
        assert!(writer.write_wide_string(Address::new(0x1000), "").is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "ASCII")
            .is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "Unicode 世界")
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_fill_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test fill with various sizes
        assert!(writer.fill(Address::new(0x1000), 0x00, 0).is_ok());
        assert!(writer.fill(Address::new(0x1000), 0xFF, 1).is_err());
        assert!(writer.fill(Address::new(0x1000), 0xAA, 100).is_err());
        assert!(writer.fill(Address::new(0x1000), 0x55, 10000).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_batch_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test batch with various sizes
        let batch_u8 = vec![(Address::new(0x1000), 1u8), (Address::new(0x2000), 2u8)];
        let results = writer.write_batch(&batch_u8);
        assert!(results.iter().all(|r| r.is_err()));

        let batch_u32 = vec![
            (Address::new(0x1000), 100u32),
            (Address::new(0x2000), 200u32),
            (Address::new(0x3000), 300u32),
        ];
        let results = writer.write_batch(&batch_u32);
        assert!(results.iter().all(|r| r.is_err()));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_copy_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test copy with various configurations
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 100)
            .is_err());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0), 100)
            .is_err());
        assert!(writer
            .copy_memory(Address::new(0), Address::new(0x1000), 100)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_swap_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test swap with various configurations
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 100)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0), Address::new(0x1000), 100)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0), 100)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0), Address::new(0), 100)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_address_overflow_detection() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test various overflow scenarios
        let max_addr = Address::new(usize::MAX);
        assert!(writer.write_bytes(max_addr, &[1, 2, 3]).is_err());
        assert!(writer.write(max_addr, 42u32).is_err());
        assert!(writer.fill(max_addr, 0xFF, 10).is_err());

        let near_max = Address::new(usize::MAX - 5);
        assert!(writer.write_bytes(near_max, &[0; 10]).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_permissions_disabled() {
        let handle = create_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        // Disable permission checks and test
        writer.set_check_permissions(false);

        // Even with checks disabled, operations should fail on invalid memory
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());
        assert!(writer.write_string(Address::new(0x1000), "test").is_err());
        assert!(writer.fill(Address::new(0x1000), 0xCC, 100).is_err());
    }
}

#[cfg(test)]
mod memory_operations_coverage {
    use super::*;
    use memory_mcp::memory::MemoryOperations;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_operations_safe_writer() {
        let handle = create_test_handle();
        let ops = MemoryOperations::new(handle);

        // Test safe_writer() method
        let writer = ops.safe_writer();
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_operations_writer() {
        let handle = create_test_handle();
        let ops = MemoryOperations::new(handle);

        // Test writer() method
        let writer = ops.writer();
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());
    }
}
