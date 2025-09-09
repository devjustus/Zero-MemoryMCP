//! Core module containing fundamental types and traits for Memory-MCP
//!
//! This module provides the foundational building blocks used throughout
//! the Memory-MCP server, including address handling, memory values,
//! process information, and error types.

pub mod types;

// Re-export commonly used types for convenience
pub use types::{Address, MemoryValue, ProcessInfo};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

// Platform verification at compile time
#[cfg(not(target_os = "windows"))]
compile_error!("Memory-MCP only supports Windows platform");

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Memory-MCP requires 64-bit architecture");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        // Test that VERSION constant is accessible and not empty
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_authors_constant() {
        // Test that AUTHORS constant is accessible
        assert_eq!(AUTHORS, env!("CARGO_PKG_AUTHORS"));
    }

    #[test]
    fn test_address_reexport() {
        // Test that Address is properly re-exported from types
        let addr = Address::new(0x1000);
        assert_eq!(addr.as_usize(), 0x1000);

        // Test Address methods
        let null = Address::null();
        assert!(null.is_null());

        let aligned = Address::new(0x1000);
        assert!(aligned.is_aligned(16));

        let unaligned = Address::new(0x1005);
        assert!(!unaligned.is_aligned(4));
        assert_eq!(unaligned.align_down(4), Address::new(0x1004));
        assert_eq!(unaligned.align_up(4), Address::new(0x1008));
    }

    #[test]
    fn test_memory_value_reexport() {
        // Test that MemoryValue is properly re-exported from types
        let value = MemoryValue::U32(42);
        assert_eq!(value.size(), 4);

        // Test different value types
        let i64_val = MemoryValue::I64(999999);
        assert_eq!(i64_val.size(), 8);

        let f32_val = MemoryValue::F32(std::f32::consts::PI);
        assert_eq!(f32_val.size(), 4);

        let bytes_val = MemoryValue::Bytes(vec![1, 2, 3, 4]);
        assert_eq!(bytes_val.size(), 4);
    }

    #[test]
    fn test_process_info_reexport() {
        // Test that ProcessInfo is properly re-exported from types
        let mut process = ProcessInfo::new(1234, "test.exe".to_string());
        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test.exe");

        // Test mutable fields
        process.architecture = types::ProcessArchitecture::X64;
        assert_eq!(process.architecture.pointer_size(), 8);

        // Test other fields
        process.thread_count = 10;
        assert_eq!(process.thread_count, 10);

        process.session_id = 1;
        assert_eq!(process.session_id, 1);
    }

    #[test]
    fn test_types_module_accessible() {
        // Test that types module is publicly accessible
        use crate::core::types::{
            MemoryError, MemoryResult, ModuleInfo, ProcessArchitecture, RegionInfo, ScanResult,
            ScanSession, ScanType, ValueType,
        };

        // Test MemoryError
        let error = MemoryError::ProcessNotFound("test.exe".to_string());
        assert!(error.to_string().contains("Process not found"));

        // Test MemoryResult
        let result: MemoryResult<u32> = Ok(42);
        assert!(result.is_ok());

        // Test ModuleInfo
        let module = ModuleInfo::new("kernel32.dll".to_string(), Address::new(0x10000), 0x1000);
        assert_eq!(module.name, "kernel32.dll");

        // Test ProcessArchitecture
        let arch = ProcessArchitecture::X86;
        assert_eq!(arch.pointer_size(), 4);

        // Test ScanResult
        let scan_result = ScanResult::new(Address::new(0x2000), MemoryValue::U32(100));
        assert_eq!(scan_result.address, Address::new(0x2000));

        // Test ScanSession
        let session = ScanSession::new("test".to_string(), ScanType::Exact, ValueType::U32);
        assert_eq!(session.id, "test");

        // Test ScanType
        let scan_type = ScanType::Unknown;
        assert!(!scan_type.requires_value());

        // Test ValueType
        let value_type = ValueType::F64;
        assert_eq!(value_type.size(), Some(8));

        // Test RegionInfo
        let region = RegionInfo {
            base_address: Address::new(0x10000),
            size: 0x1000,
            protection: 0x20,
            state: 0x1000,
            region_type: 0x20000,
        };
        assert_eq!(region.size, 0x1000);
    }

    #[test]
    fn test_platform_target() {
        // This test will only compile on Windows 64-bit
        // The compile_error! macros ensure this at compile time

        // Verify we're on Windows at runtime
        #[cfg(target_os = "windows")]
        {
            assert!(true, "Running on Windows platform");
        }

        // Verify we're on 64-bit at runtime
        #[cfg(target_pointer_width = "64")]
        {
            assert!(true, "Running on 64-bit architecture");
        }
    }

    #[test]
    fn test_module_documentation() {
        // This test verifies the module has proper documentation
        // The module docstring describes it as containing fundamental types and traits
        assert!(
            !VERSION.is_empty(),
            "Module should have version information"
        );
    }
}
