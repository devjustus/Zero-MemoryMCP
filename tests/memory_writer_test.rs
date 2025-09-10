//! Comprehensive tests for memory writer module

use memory_mcp::core::types::{Address, MemoryValue};
use memory_mcp::memory::writer::{
    BasicMemoryWriter, BatchWrite, ExtendedWrite, MemoryCopy, MemoryWrite, SafeMemoryWriter,
};
use memory_mcp::process::ProcessHandle;

fn create_test_handle() -> ProcessHandle {
    ProcessHandle::open_for_read(std::process::id())
        .unwrap_or_else(|_| ProcessHandle::open_for_read(4).unwrap())
}

#[cfg(test)]
mod basic_writer_tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_basic_writer_creation() {
        let handle = create_test_handle();
        let _writer = BasicMemoryWriter::new(&handle);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_bytes() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        let data = vec![1, 2, 3, 4, 5];
        let result = writer.write_bytes(Address::new(0x1000), &data);
        assert!(result.is_err()); // Should fail with invalid address
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_typed_values() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer.write(Address::new(0x1000), 42u8).is_err());
        assert!(writer.write(Address::new(0x1000), 1234u16).is_err());
        assert!(writer.write(Address::new(0x1000), 0xDEADBEEFu32).is_err());
        assert!(writer
            .write(Address::new(0x1000), 0x123456789ABCDEFu64)
            .is_err());
        assert!(writer.write(Address::new(0x1000), -42i32).is_err());
        assert!(writer
            .write(Address::new(0x1000), std::f32::consts::PI)
            .is_err());
        assert!(writer
            .write(Address::new(0x1000), std::f64::consts::E)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_memory_values() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        let values = vec![
            MemoryValue::U8(255),
            MemoryValue::U16(65535),
            MemoryValue::U32(0xFFFFFFFF),
            MemoryValue::U64(0xFFFFFFFFFFFFFFFF),
            MemoryValue::I8(-128),
            MemoryValue::I16(-32768),
            MemoryValue::I32(-2147483648),
            MemoryValue::I64(i64::MIN),
            MemoryValue::F32(std::f32::consts::PI),
            MemoryValue::F64(std::f64::consts::E),
            MemoryValue::String("Hello, World!".to_string()),
            MemoryValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ];

        for value in values {
            assert!(writer.write_value(Address::new(0x1000), &value).is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_string() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer.write_string(Address::new(0x1000), "Hello").is_err());
        assert!(writer.write_string(Address::new(0x1000), "").is_err());
        assert!(writer
            .write_string(Address::new(0x1000), "Test\nNewline")
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_wide_string() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer
            .write_wide_string(Address::new(0x1000), "Hello")
            .is_err());
        assert!(writer
            .write_wide_string(Address::new(0x1000), "世界")
            .is_err());
        assert!(writer.write_wide_string(Address::new(0x1000), "").is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_fill_memory() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer.fill(Address::new(0x1000), 0xCC, 100).is_err());
        assert!(writer.fill(Address::new(0x1000), 0x00, 0).is_ok()); // Zero size should succeed
        assert!(writer.fill(Address::new(0x1000), 0xFF, 8192).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_batch_write() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        let writes = vec![
            (Address::new(0x1000), 42u32),
            (Address::new(0x2000), 84u32),
            (Address::new(0x3000), 168u32),
        ];

        let results = writer.write_batch(&writes);
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_copy_memory() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 100)
            .is_err());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 16384)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_swap_memory() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 100)
            .is_err());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
    }
}

#[cfg(test)]
mod safe_writer_tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_writer_creation() {
        let handle = create_test_handle();
        let _writer = SafeMemoryWriter::new(&handle);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_writer_settings() {
        let handle = create_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        writer.set_verify_writes(false);
        writer.set_check_permissions(false);

        writer.set_verify_writes(true);
        writer.set_check_permissions(true);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_verified() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        assert!(writer.write_verified(Address::new(0x1000), 42u32).is_err());
        assert!(writer
            .write_verified(Address::new(0x1000), std::f32::consts::PI)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_write_with_backup() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        let result = writer.write_with_backup(Address::new(0x1000), 42u32);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_restore_from_backup() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        let backup = vec![1, 2, 3, 4];
        assert!(writer
            .restore_from_backup(Address::new(0x1000), &backup)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_write_null_address() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Should fail with null address check
        assert!(writer.write(Address::new(0), 42u32).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_write_with_overflow_check() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test address overflow detection
        let max_addr = Address::new(usize::MAX - 10);
        assert!(writer.write_bytes(max_addr, &[0; 20]).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_batch_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        let writes = vec![
            (Address::new(0), 1u32), // Should fail with null check
            (Address::new(0x1000), 2u32),
            (Address::new(0x2000), 3u32),
        ];

        let results = writer.write_batch(&writes);
        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_safe_memory_operations() {
        let handle = create_test_handle();
        let writer = SafeMemoryWriter::new(&handle);

        // Test copy with permission checks
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0), 100)
            .is_err());

        // Test swap with permission checks
        assert!(writer
            .swap_memory(Address::new(0), Address::new(0x1000), 100)
            .is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_disabled_verification() {
        let handle = create_test_handle();
        let mut writer = SafeMemoryWriter::new(&handle);

        writer.set_verify_writes(false);
        writer.set_check_permissions(false);

        // Even with checks disabled, should still fail on invalid memory
        assert!(writer.write(Address::new(0x1000), 42u32).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_writer_comparison() {
        let handle = create_test_handle();
        let basic_writer = BasicMemoryWriter::new(&handle);
        let safe_writer = SafeMemoryWriter::new(&handle);

        let addr = Address::new(0x1000);
        let value = 42u32;

        // Both should fail with same invalid address
        assert!(basic_writer.write(addr, value).is_err());
        assert!(safe_writer.write(addr, value).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_all_value_types() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test all primitive types
        assert!(writer.write(Address::new(0x1000), 0u8).is_err());
        assert!(writer.write(Address::new(0x1000), 0u16).is_err());
        assert!(writer.write(Address::new(0x1000), 0u32).is_err());
        assert!(writer.write(Address::new(0x1000), 0u64).is_err());
        assert!(writer.write(Address::new(0x1000), 0i8).is_err());
        assert!(writer.write(Address::new(0x1000), 0i16).is_err());
        assert!(writer.write(Address::new(0x1000), 0i32).is_err());
        assert!(writer.write(Address::new(0x1000), 0i64).is_err());
        assert!(writer.write(Address::new(0x1000), 0.0f32).is_err());
        assert!(writer.write(Address::new(0x1000), 0.0f64).is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_empty_operations() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Empty operations should succeed
        assert!(writer.write_bytes(Address::new(0x1000), &[]).is_ok());
        assert!(writer.fill(Address::new(0x1000), 0xCC, 0).is_ok());
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
        assert!(writer
            .swap_memory(Address::new(0x1000), Address::new(0x2000), 0)
            .is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_large_operations() {
        let handle = create_test_handle();
        let writer = BasicMemoryWriter::new(&handle);

        // Test large fill operation (should handle chunking)
        assert!(writer.fill(Address::new(0x1000), 0xCC, 100_000).is_err());

        // Test large copy operation (should handle chunking)
        assert!(writer
            .copy_memory(Address::new(0x1000), Address::new(0x10000), 100_000)
            .is_err());
    }
}
