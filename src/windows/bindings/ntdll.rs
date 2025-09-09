//! NTDLL.dll bindings for low-level system operations

use crate::core::types::{MemoryError, MemoryResult};
use std::ffi::c_void;
use std::mem;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, ULONG};
use winapi::shared::ntdef::{NTSTATUS, PVOID};
use winapi::um::winnt::{HANDLE, MEMORY_BASIC_INFORMATION};

// NT Status codes
pub const STATUS_SUCCESS: NTSTATUS = 0x00000000;
pub const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004_u32 as i32;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC0000022_u32 as i32;

/// Process information class for NtQueryInformationProcess
#[repr(C)]
pub enum ProcessInfoClass {
    ProcessBasicInformation = 0,
    ProcessDebugPort = 7,
    ProcessWow64Information = 26,
    ProcessImageFileName = 27,
    ProcessDebugObjectHandle = 30,
}

/// Basic process information structure
#[repr(C)]
pub struct ProcessBasicInfo {
    pub exit_status: NTSTATUS,
    pub peb_base_address: PVOID,
    pub affinity_mask: usize,
    pub base_priority: i32,
    pub unique_process_id: usize,
    pub inherited_from_unique_process_id: usize,
}

/// System information class
#[repr(C)]
pub enum SystemInfoClass {
    SystemBasicInformation = 0,
    SystemProcessInformation = 5,
    SystemHandleInformation = 16,
    SystemExtendedHandleInformation = 64,
}

// External function declarations (would normally link to ntdll.dll)
#[link(name = "ntdll")]
extern "system" {
    fn NtQueryInformationProcess(
        process_handle: HANDLE,
        process_info_class: ULONG,
        process_info: PVOID,
        process_info_length: ULONG,
        return_length: *mut ULONG,
    ) -> NTSTATUS;

    fn NtQuerySystemInformation(
        system_info_class: ULONG,
        system_info: PVOID,
        system_info_length: ULONG,
        return_length: *mut ULONG,
    ) -> NTSTATUS;

    fn NtQueryVirtualMemory(
        process_handle: HANDLE,
        base_address: PVOID,
        memory_info_class: ULONG,
        memory_info: PVOID,
        memory_info_length: SIZE_T,
        return_length: *mut SIZE_T,
    ) -> NTSTATUS;
}

/// Check if NTSTATUS indicates success
pub fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

/// Safe wrapper for NtQueryInformationProcess
///
/// # Safety
/// The handle must be a valid process handle
pub unsafe fn query_process_information(
    handle: HANDLE,
    info_class: ProcessInfoClass,
) -> MemoryResult<ProcessBasicInfo> {
    let mut info = ProcessBasicInfo {
        exit_status: 0,
        peb_base_address: std::ptr::null_mut(),
        affinity_mask: 0,
        base_priority: 0,
        unique_process_id: 0,
        inherited_from_unique_process_id: 0,
    };

    let mut return_length = 0u32;

    let status = NtQueryInformationProcess(
        handle,
        info_class as ULONG,
        &mut info as *mut _ as PVOID,
        mem::size_of::<ProcessBasicInfo>() as ULONG,
        &mut return_length,
    );

    if nt_success(status) {
        Ok(info)
    } else {
        Err(MemoryError::WindowsApi(format!(
            "NtQueryInformationProcess failed with status: 0x{:X}",
            status
        )))
    }
}

/// Query if process is WoW64 (32-bit on 64-bit Windows)
///
/// # Safety
/// The handle must be a valid process handle
pub unsafe fn is_wow64_process(handle: HANDLE) -> MemoryResult<bool> {
    let mut wow64_peb: usize = 0;
    let mut return_length = 0u32;

    let status = NtQueryInformationProcess(
        handle,
        ProcessInfoClass::ProcessWow64Information as ULONG,
        &mut wow64_peb as *mut _ as PVOID,
        mem::size_of::<usize>() as ULONG,
        &mut return_length,
    );

    if nt_success(status) {
        Ok(wow64_peb != 0)
    } else {
        Err(MemoryError::WindowsApi(format!(
            "Failed to query WoW64 status: 0x{:X}",
            status
        )))
    }
}

/// Memory information class for NtQueryVirtualMemory
#[repr(C)]
pub enum MemoryInfoClass {
    MemoryBasicInformation = 0,
    MemoryWorkingSetList = 1,
    MemoryMappedFilenameInformation = 2,
}

/// Safe wrapper for NtQueryVirtualMemory
///
/// # Safety
/// The handle must be a valid process handle
pub unsafe fn query_virtual_memory(
    handle: HANDLE,
    address: usize,
) -> MemoryResult<MEMORY_BASIC_INFORMATION> {
    let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();
    let mut return_length = 0;

    let status = NtQueryVirtualMemory(
        handle,
        address as PVOID,
        MemoryInfoClass::MemoryBasicInformation as ULONG,
        &mut mbi as *mut _ as PVOID,
        mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        &mut return_length,
    );

    if nt_success(status) {
        Ok(mbi)
    } else {
        Err(MemoryError::WindowsApi(format!(
            "NtQueryVirtualMemory failed for address 0x{:X}: 0x{:X}",
            address, status
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_nt_success() {
        assert!(nt_success(STATUS_SUCCESS));
        assert!(!nt_success(STATUS_ACCESS_DENIED));
        assert!(!nt_success(STATUS_INFO_LENGTH_MISMATCH));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_null_handle_queries() {
        unsafe {
            // Querying with null handle should fail
            let result = query_process_information(
                ptr::null_mut(),
                ProcessInfoClass::ProcessBasicInformation,
            );
            assert!(result.is_err());

            let result = is_wow64_process(ptr::null_mut());
            assert!(result.is_err());

            let result = query_virtual_memory(ptr::null_mut(), 0x1000);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_process_info_class_values() {
        // Verify enum values match Windows constants
        assert_eq!(ProcessInfoClass::ProcessBasicInformation as u32, 0);
        assert_eq!(ProcessInfoClass::ProcessDebugPort as u32, 7);
        assert_eq!(ProcessInfoClass::ProcessWow64Information as u32, 26);
    }

    #[test]
    fn test_memory_info_class_values() {
        // Verify memory info class enum values
        assert_eq!(MemoryInfoClass::MemoryBasicInformation as u32, 0);
        assert_eq!(MemoryInfoClass::MemoryWorkingSetList as u32, 1);
        assert_eq!(MemoryInfoClass::MemoryMappedFilenameInformation as u32, 2);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_get_current_process() {
        // Test current process handle is valid
        use winapi::um::processthreadsapi::GetCurrentProcess;
        let handle = unsafe { GetCurrentProcess() };
        assert!(!handle.is_null());
        // Current process pseudo-handle is always -1
        assert_eq!(handle as isize, -1);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_query_with_invalid_addresses() {
        unsafe {
            use winapi::um::processthreadsapi::GetCurrentProcess;
            let current_process = GetCurrentProcess();

            // Test with invalid addresses
            let result = query_virtual_memory(current_process, usize::MAX);
            // This might succeed or fail depending on the system, just check it doesn't crash
            let _ = result;

            let result = query_virtual_memory(current_process, 0);
            // Address 0 is typically invalid but might not error on all systems
            let _ = result;
        }
    }

    #[test]
    fn test_status_codes() {
        // Test various status codes
        assert!(nt_success(0)); // STATUS_SUCCESS
        assert!(!nt_success(0xC0000005u32 as i32)); // STATUS_ACCESS_VIOLATION
        assert!(!nt_success(0xC000000Du32 as i32)); // STATUS_INVALID_PARAMETER
        assert!(!nt_success(0x80000000u32 as i32)); // High bit set indicates error
    }
}
