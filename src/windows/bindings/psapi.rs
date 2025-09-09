//! PSAPI.dll bindings for process and module enumeration

use crate::core::types::{MemoryError, MemoryResult};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, MAX_PATH};
use winapi::um::psapi::{
    EnumProcessModules, EnumProcesses, GetModuleBaseNameW, GetModuleInformation,
    GetProcessImageFileNameW, MODULEINFO,
};
use winapi::um::winnt::HANDLE;

/// Safe wrapper for EnumProcesses
pub fn enum_processes() -> MemoryResult<Vec<u32>> {
    let mut pids = vec![0u32; 1024];
    let mut bytes_needed = 0u32;

    unsafe {
        let result = EnumProcesses(
            pids.as_mut_ptr(),
            (pids.len() * std::mem::size_of::<DWORD>()) as u32,
            &mut bytes_needed,
        );

        if result == FALSE {
            return Err(MemoryError::WindowsApi(
                "Failed to enumerate processes".to_string(),
            ));
        }
    }

    let count = bytes_needed as usize / std::mem::size_of::<DWORD>();
    pids.truncate(count);
    pids.retain(|&pid| pid != 0);

    Ok(pids)
}

/// Safe wrapper for EnumProcessModules
///
/// # Safety
/// The handle must be a valid process handle
pub unsafe fn enum_process_modules(handle: HANDLE) -> MemoryResult<Vec<HMODULE>> {
    let mut modules = vec![std::ptr::null_mut(); 1024];
    let mut bytes_needed = 0u32;

    let result = EnumProcessModules(
        handle,
        modules.as_mut_ptr(),
        (modules.len() * std::mem::size_of::<HMODULE>()) as u32,
        &mut bytes_needed,
    );

    if result == FALSE {
        return Err(MemoryError::WindowsApi(
            "Failed to enumerate process modules".to_string(),
        ));
    }

    let count = bytes_needed as usize / std::mem::size_of::<HMODULE>();
    modules.truncate(count);

    Ok(modules)
}

/// Safe wrapper for GetModuleInformation
///
/// # Safety
/// The handle must be a valid process handle and module must be valid
pub unsafe fn get_module_information(handle: HANDLE, module: HMODULE) -> MemoryResult<MODULEINFO> {
    let mut info = MODULEINFO {
        lpBaseOfDll: std::ptr::null_mut(),
        SizeOfImage: 0,
        EntryPoint: std::ptr::null_mut(),
    };

    let result = GetModuleInformation(
        handle,
        module,
        &mut info,
        std::mem::size_of::<MODULEINFO>() as u32,
    );

    if result == FALSE {
        return Err(MemoryError::WindowsApi(
            "Failed to get module information".to_string(),
        ));
    }

    Ok(info)
}

/// Safe wrapper for GetModuleBaseNameW
///
/// # Safety
/// The handle must be a valid process handle and module must be valid
pub unsafe fn get_module_base_name(handle: HANDLE, module: HMODULE) -> MemoryResult<String> {
    let mut buffer = vec![0u16; MAX_PATH];

    let length = GetModuleBaseNameW(handle, module, buffer.as_mut_ptr(), MAX_PATH as u32);

    if length == 0 {
        return Err(MemoryError::WindowsApi(
            "Failed to get module base name".to_string(),
        ));
    }

    buffer.truncate(length as usize);

    let os_string = OsString::from_wide(&buffer);
    os_string
        .into_string()
        .map_err(|_| MemoryError::WindowsApi("Invalid module name encoding".to_string()))
}

/// Safe wrapper for GetProcessImageFileNameW
///
/// # Safety
/// The handle must be a valid process handle
pub unsafe fn get_process_image_filename(handle: HANDLE) -> MemoryResult<String> {
    let mut buffer = vec![0u16; MAX_PATH];

    let length = GetProcessImageFileNameW(handle, buffer.as_mut_ptr(), MAX_PATH as u32);

    if length == 0 {
        return Err(MemoryError::WindowsApi(
            "Failed to get process image filename".to_string(),
        ));
    }

    buffer.truncate(length as usize);

    let os_string = OsString::from_wide(&buffer);
    os_string
        .into_string()
        .map_err(|_| MemoryError::WindowsApi("Invalid filename encoding".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enum_processes() {
        // Should be able to enumerate processes (at least System process)
        let result = enum_processes();
        assert!(result.is_ok());

        if let Ok(pids) = result {
            assert!(!pids.is_empty());
            // System process (PID 4) should usually exist on Windows
            // But we won't assert this as it could vary
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_null_handle_operations() {
        unsafe {
            // Operations with null handle should fail
            let result = enum_process_modules(ptr::null_mut());
            assert!(result.is_err());

            let result = get_module_information(ptr::null_mut(), ptr::null_mut());
            assert!(result.is_err());

            let result = get_module_base_name(ptr::null_mut(), ptr::null_mut());
            assert!(result.is_err());

            let result = get_process_image_filename(ptr::null_mut());
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_invalid_module_operations() {
        unsafe {
            // Test with invalid module handle
            let invalid_module = 0xDEADBEEF as HMODULE;
            let current_process = winapi::um::processthreadsapi::GetCurrentProcess();

            let result = get_module_information(current_process, invalid_module);
            assert!(result.is_err());

            let result = get_module_base_name(current_process, invalid_module);
            assert!(result.is_err());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_enumeration_max_count() {
        // Test that we handle the maximum process count properly
        let result = enum_processes();
        assert!(result.is_ok());

        if let Ok(pids) = result {
            // Should not exceed MAX_PROCESSES
            assert!(pids.len() <= 1024);
            // Should have at least some processes
            assert!(!pids.is_empty());
        }
    }
}
