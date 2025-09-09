//! Kernel32.dll bindings for process and memory operations

use crate::core::types::{MemoryError, MemoryResult};
use std::ffi::c_void;
use std::{mem, ptr};
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx, WriteProcessMemory};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{HANDLE, MEMORY_BASIC_INFORMATION, PROCESS_ALL_ACCESS};

/// Safe wrapper for OpenProcess
pub fn open_process(pid: u32, desired_access: u32) -> MemoryResult<HANDLE> {
    unsafe {
        let handle = OpenProcess(desired_access, FALSE, pid);
        if handle.is_null() {
            Err(MemoryError::ProcessNotFound(format!("PID: {}", pid)))
        } else {
            Ok(handle)
        }
    }
}

/// Safe wrapper for opening process with all access
pub fn open_process_all_access(pid: u32) -> MemoryResult<HANDLE> {
    open_process(pid, PROCESS_ALL_ACCESS)
}

/// Safe wrapper for CloseHandle
///
/// # Safety
/// The handle must be a valid Windows handle
pub unsafe fn close_handle(handle: HANDLE) -> MemoryResult<()> {
    if handle.is_null() {
        return Ok(());
    }

    if CloseHandle(handle) == FALSE {
        Err(MemoryError::WindowsApi(
            "Failed to close handle".to_string(),
        ))
    } else {
        Ok(())
    }
}

/// Safe wrapper for ReadProcessMemory
///
/// # Safety
/// The handle must be a valid process handle with appropriate access rights
pub unsafe fn read_process_memory(
    handle: HANDLE,
    address: usize,
    buffer: &mut [u8],
) -> MemoryResult<usize> {
    let mut bytes_read = 0;

    let result = ReadProcessMemory(
        handle,
        address as LPVOID,
        buffer.as_mut_ptr() as LPVOID,
        buffer.len(),
        &mut bytes_read,
    );

    if result == FALSE {
        Err(MemoryError::read_failed(
            format!("0x{:X}", address),
            "ReadProcessMemory failed",
        ))
    } else {
        Ok(bytes_read)
    }
}

/// Safe wrapper for WriteProcessMemory
///
/// # Safety
/// The handle must be a valid process handle with appropriate access rights
pub unsafe fn write_process_memory(
    handle: HANDLE,
    address: usize,
    data: &[u8],
) -> MemoryResult<usize> {
    let mut bytes_written = 0;

    let result = WriteProcessMemory(
        handle,
        address as LPVOID,
        data.as_ptr() as LPVOID,
        data.len(),
        &mut bytes_written,
    );

    if result == FALSE {
        Err(MemoryError::write_failed(
            format!("0x{:X}", address),
            "WriteProcessMemory failed",
        ))
    } else {
        Ok(bytes_written)
    }
}

/// Safe wrapper for VirtualQueryEx
///
/// # Safety
/// The handle must be a valid process handle with appropriate access rights
pub unsafe fn virtual_query_ex(
    handle: HANDLE,
    address: usize,
) -> MemoryResult<MEMORY_BASIC_INFORMATION> {
    let mut mbi: MEMORY_BASIC_INFORMATION = mem::zeroed();

    let result = VirtualQueryEx(
        handle,
        address as LPVOID,
        &mut mbi,
        mem::size_of::<MEMORY_BASIC_INFORMATION>(),
    );

    if result == 0 {
        Err(MemoryError::WindowsApi(format!(
            "VirtualQueryEx failed for address: 0x{:X}",
            address
        )))
    } else {
        Ok(mbi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle_operations() {
        unsafe {
            // Closing null handle should succeed
            assert!(close_handle(ptr::null_mut()).is_ok());

            // Reading from null handle should fail
            let mut buffer = vec![0u8; 4];
            assert!(read_process_memory(ptr::null_mut(), 0x1000, &mut buffer).is_err());

            // Writing to null handle should fail
            let data = vec![0u8; 4];
            assert!(write_process_memory(ptr::null_mut(), 0x1000, &data).is_err());
        }
    }

    #[test]
    fn test_open_invalid_process() {
        // Opening process with invalid PID should fail
        let result = open_process(0, PROCESS_ALL_ACCESS);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_process_all_access() {
        // Should fail for invalid PID
        let result = open_process_all_access(0);
        assert!(result.is_err());
    }
}
