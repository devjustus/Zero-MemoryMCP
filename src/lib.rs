//! Memory-MCP library for Windows memory manipulation

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod config;
pub mod core;
pub mod memory;
pub mod process;
pub mod windows;

// Re-export main types from core module
pub use core::types::{
    Address, MemoryError, MemoryResult, MemoryValue, ModuleInfo, ProcessArchitecture, ProcessId,
    ProcessInfo, ThreadId, ValueType,
};

// Re-export core directly for full access
pub use core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_module_accessible() {
        // Test that core module is accessible
        let _version = core::VERSION;
        let _authors = core::AUTHORS;
        assert_eq!(core::VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_address_reexport() {
        // Test that Address is properly re-exported
        let addr = Address::new(0x1000);
        assert_eq!(addr.as_usize(), 0x1000);

        // Test null address
        let null = Address::null();
        assert!(null.is_null());
    }

    #[test]
    fn test_memory_value_reexport() {
        // Test that MemoryValue is properly re-exported
        let value = MemoryValue::U32(42);
        assert_eq!(value.value_type(), ValueType::U32);
        assert_eq!(value.size(), 4);

        // Test different value types
        let i32_val = MemoryValue::I32(-100);
        assert_eq!(i32_val.value_type(), ValueType::I32);

        let f64_val = MemoryValue::F64(std::f64::consts::PI);
        assert_eq!(f64_val.value_type(), ValueType::F64);

        let string_val = MemoryValue::String("test".to_string());
        assert_eq!(string_val.value_type(), ValueType::String);
    }

    #[test]
    fn test_process_info_reexport() {
        // Test that ProcessInfo is properly re-exported
        let process = ProcessInfo::new(1234, "test.exe".to_string());
        assert_eq!(process.pid, 1234);
        assert_eq!(process.name, "test.exe");
        assert_eq!(process.architecture, ProcessArchitecture::Unknown);
    }

    #[test]
    fn test_module_info_reexport() {
        // Test that ModuleInfo is properly re-exported
        let module = ModuleInfo::new("kernel32.dll".to_string(), Address::new(0x10000), 0x1000);
        assert_eq!(module.name, "kernel32.dll");
        assert_eq!(module.base_address, Address::new(0x10000));
        assert_eq!(module.size, 0x1000);
        assert!(module.contains_address(Address::new(0x10500)));
    }

    #[test]
    fn test_process_architecture_reexport() {
        // Test that ProcessArchitecture is properly re-exported
        let arch_x86 = ProcessArchitecture::X86;
        assert_eq!(arch_x86.pointer_size(), 4);

        let arch_x64 = ProcessArchitecture::X64;
        assert_eq!(arch_x64.pointer_size(), 8);

        let arch_unknown = ProcessArchitecture::Unknown;
        assert_eq!(arch_unknown.pointer_size(), 8);
    }

    #[test]
    fn test_memory_error_reexport() {
        // Test that MemoryError is properly re-exported
        let error = MemoryError::ProcessNotFound("notepad.exe".to_string());
        assert!(error.to_string().contains("Process not found"));

        let error2 = MemoryError::InvalidAddress("0xBAD".to_string());
        assert!(error2.to_string().contains("Invalid memory address"));
    }

    #[test]
    fn test_memory_result_reexport() {
        // Test that MemoryResult is properly re-exported
        let result: MemoryResult<u32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert_eq!(value, 42);
        }

        let error_result: MemoryResult<u32> = Err(MemoryError::Unknown("test".to_string()));
        assert!(error_result.is_err());
    }

    #[test]
    fn test_value_type_reexport() {
        // Test that ValueType is properly re-exported
        let vt = ValueType::U32;
        assert_eq!(vt.size(), Some(4));

        let string_type = ValueType::String;
        assert_eq!(string_type.size(), None);

        let bytes_type = ValueType::Bytes;
        assert_eq!(bytes_type.size(), None);
    }

    #[test]
    fn test_process_and_thread_id_reexport() {
        // Test that ProcessId and ThreadId are properly re-exported
        let pid: ProcessId = 1234;
        let tid: ThreadId = 5678;

        assert_eq!(pid, 1234);
        assert_eq!(tid, 5678);
    }

    #[test]
    #[cfg_attr(miri, ignore = "SystemTime operations not supported under Miri")]
    fn test_all_types_module_exports() {
        // Import from types module through core
        use crate::core::types::{RegionInfo, ScanResult, ScanSession, ScanType};

        // Test ScanResult
        let result = ScanResult::new(Address::new(0x2000), MemoryValue::U64(999));
        assert_eq!(result.address, Address::new(0x2000));

        // Test ScanSession
        let session = ScanSession::new("test-session".to_string(), ScanType::Exact, ValueType::U32);
        assert_eq!(session.id, "test-session");

        // Test ScanType
        let scan_type = ScanType::Between;
        assert!(scan_type.requires_value());

        // Test RegionInfo
        let region = RegionInfo {
            base_address: Address::new(0x10000),
            size: 0x1000,
            protection: 0x20,
            state: 0x1000,
            region_type: 0x20000,
        };
        assert_eq!(region.base_address, Address::new(0x10000));
    }

    #[test]
    fn test_core_constants() {
        // Test that core constants are accessible
        // VERSION should match the package version
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(AUTHORS, env!("CARGO_PKG_AUTHORS"));
    }
}
