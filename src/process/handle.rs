//! Safe process handle wrapper with RAII semantics

use crate::core::types::{MemoryError, MemoryResult};
use crate::windows::bindings::kernel32;
use crate::windows::types::Handle;
use std::fmt;
use winapi::um::winnt::HANDLE;

/// Access rights for process handles
#[derive(Debug, Clone, Copy)]
pub struct ProcessAccess {
    value: u32,
}

impl ProcessAccess {
    /// All possible access rights
    pub const ALL_ACCESS: Self = Self { value: 0x1FFFFF };
    /// Query information access
    pub const QUERY_INFORMATION: Self = Self { value: 0x0400 };
    /// Read memory access
    pub const VM_READ: Self = Self { value: 0x0010 };
    /// Write memory access
    pub const VM_WRITE: Self = Self { value: 0x0020 };
    /// Execute operations
    pub const VM_OPERATION: Self = Self { value: 0x0008 };

    /// Combine access rights
    pub fn combine(rights: &[Self]) -> Self {
        let mut value = 0;
        for right in rights {
            value |= right.value;
        }
        Self { value }
    }

    /// Get raw value
    pub fn value(&self) -> u32 {
        self.value
    }
}

/// Safe wrapper around a Windows process handle
pub struct ProcessHandle {
    handle: Handle,
    pid: u32,
    access: ProcessAccess,
}

impl ProcessHandle {
    /// Create a new ProcessHandle from raw handle
    ///
    /// # Safety
    /// This function is intended for testing purposes only.
    /// The handle must be valid or null.
    #[doc(hidden)]
    pub fn from_raw_handle(handle: *mut winapi::ctypes::c_void, pid: u32) -> Self {
        ProcessHandle {
            handle: Handle::new(handle),
            pid,
            access: ProcessAccess::QUERY_INFORMATION,
        }
    }

    /// Create a new ProcessHandle (for internal testing only)
    #[cfg(test)]
    pub fn new(handle: *mut winapi::ctypes::c_void, pid: u32) -> Self {
        Self::from_raw_handle(handle, pid)
    }

    /// Open a process with specified access rights
    pub fn open(pid: u32, access: ProcessAccess) -> MemoryResult<Self> {
        let raw_handle = kernel32::open_process(pid, access.value())?;
        Ok(ProcessHandle {
            handle: Handle::new(raw_handle),
            pid,
            access,
        })
    }

    /// Open a process with all access rights
    pub fn open_all_access(pid: u32) -> MemoryResult<Self> {
        Self::open(pid, ProcessAccess::ALL_ACCESS)
    }

    /// Open a process for reading memory
    pub fn open_for_read(pid: u32) -> MemoryResult<Self> {
        Self::open(
            pid,
            ProcessAccess::combine(&[ProcessAccess::QUERY_INFORMATION, ProcessAccess::VM_READ]),
        )
    }

    /// Open a process for reading and writing memory
    pub fn open_for_read_write(pid: u32) -> MemoryResult<Self> {
        Self::open(
            pid,
            ProcessAccess::combine(&[
                ProcessAccess::QUERY_INFORMATION,
                ProcessAccess::VM_READ,
                ProcessAccess::VM_WRITE,
                ProcessAccess::VM_OPERATION,
            ]),
        )
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get the raw handle
    ///
    /// # Safety
    /// The returned handle is only valid as long as this ProcessHandle exists
    pub unsafe fn raw(&self) -> HANDLE {
        self.handle.raw()
    }

    /// Get the access rights
    pub fn access(&self) -> ProcessAccess {
        self.access
    }

    /// Check if handle is valid
    pub fn is_valid(&self) -> bool {
        !self.handle.is_null()
    }

    /// Read memory from the process
    pub fn read_memory(&self, address: usize, buffer: &mut [u8]) -> MemoryResult<usize> {
        if !self.is_valid() {
            return Err(MemoryError::InvalidHandle(
                "Process handle is null".to_string(),
            ));
        }
        unsafe { kernel32::read_process_memory(self.handle.raw(), address, buffer) }
    }

    /// Write memory to the process
    pub fn write_memory(&self, address: usize, data: &[u8]) -> MemoryResult<usize> {
        if !self.is_valid() {
            return Err(MemoryError::InvalidHandle(
                "Process handle is null".to_string(),
            ));
        }
        unsafe { kernel32::write_process_memory(self.handle.raw(), address, data) }
    }
}

impl fmt::Debug for ProcessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcessHandle")
            .field("pid", &self.pid)
            .field("valid", &self.is_valid())
            .field("access", &format!("0x{:X}", self.access.value()))
            .finish()
    }
}

impl fmt::Display for ProcessHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ProcessHandle(pid={}, valid={})",
            self.pid,
            self.is_valid()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_handle_new() {
        let handle = ProcessHandle::new(std::ptr::null_mut(), 1234);
        assert_eq!(handle.pid(), 1234);
    }

    #[test]
    fn test_process_access_constants() {
        assert_eq!(ProcessAccess::ALL_ACCESS.value(), 0x1FFFFF);
        assert_eq!(ProcessAccess::QUERY_INFORMATION.value(), 0x0400);
        assert_eq!(ProcessAccess::VM_READ.value(), 0x0010);
        assert_eq!(ProcessAccess::VM_WRITE.value(), 0x0020);
        assert_eq!(ProcessAccess::VM_OPERATION.value(), 0x0008);
    }

    #[test]
    fn test_process_access_combine() {
        let combined = ProcessAccess::combine(&[ProcessAccess::VM_READ, ProcessAccess::VM_WRITE]);
        assert_eq!(combined.value(), 0x0030);

        let all_combined = ProcessAccess::combine(&[
            ProcessAccess::QUERY_INFORMATION,
            ProcessAccess::VM_READ,
            ProcessAccess::VM_WRITE,
            ProcessAccess::VM_OPERATION,
        ]);
        assert_eq!(all_combined.value(), 0x0438);
    }

    #[test]
    fn test_process_access_copy() {
        let access = ProcessAccess::VM_READ;
        let copied = access;
        assert_eq!(copied.value(), access.value());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_open_invalid() {
        // Opening process with PID 0 should fail
        let result = ProcessHandle::open(0, ProcessAccess::ALL_ACCESS);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_open_all_access() {
        let result = ProcessHandle::open_all_access(0);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_open_for_read() {
        let result = ProcessHandle::open_for_read(0);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_open_for_read_write() {
        let result = ProcessHandle::open_for_read_write(0);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_handle_current_process() {
        use std::process;
        let current_pid = process::id();

        // Should be able to open current process
        let handle = ProcessHandle::open_for_read(current_pid);
        if let Ok(h) = handle {
            assert_eq!(h.pid(), current_pid);
            assert!(h.is_valid());

            // Test reading memory (will likely fail but shouldn't crash)
            let mut buffer = vec![0u8; 4];
            let _ = h.read_memory(0x10000, &mut buffer);
        }
    }

    #[test]
    fn test_process_handle_display() {
        // Create a mock handle for testing display
        let handle = ProcessHandle {
            handle: Handle::null(),
            pid: 1234,
            access: ProcessAccess::VM_READ,
        };

        let display = format!("{}", handle);
        assert!(display.contains("pid=1234"));
        assert!(display.contains("valid=false"));
    }

    #[test]
    fn test_process_handle_debug() {
        let handle = ProcessHandle {
            handle: Handle::null(),
            pid: 5678,
            access: ProcessAccess::ALL_ACCESS,
        };

        let debug = format!("{:?}", handle);
        assert!(debug.contains("ProcessHandle"));
        assert!(debug.contains("pid: 5678"));
        assert!(debug.contains("valid: false"));
        assert!(debug.contains("0x1FFFFF"));
    }

    #[test]
    fn test_invalid_handle_operations() {
        let handle = ProcessHandle {
            handle: Handle::null(),
            pid: 1234,
            access: ProcessAccess::VM_READ,
        };

        assert!(!handle.is_valid());

        // Reading from invalid handle should fail
        let mut buffer = vec![0u8; 4];
        let read_result = handle.read_memory(0x1000, &mut buffer);
        assert!(read_result.is_err());
        match read_result.unwrap_err() {
            MemoryError::InvalidHandle(msg) => {
                assert!(msg.contains("null"));
            }
            _ => panic!("Expected InvalidHandle error"),
        }

        // Writing to invalid handle should fail
        let data = vec![0u8; 4];
        let write_result = handle.write_memory(0x1000, &data);
        assert!(write_result.is_err());
        match write_result.unwrap_err() {
            MemoryError::InvalidHandle(msg) => {
                assert!(msg.contains("null"));
            }
            _ => panic!("Expected InvalidHandle error"),
        }
    }

    #[test]
    fn test_process_access_debug() {
        let access = ProcessAccess::VM_READ;
        let debug = format!("{:?}", access);
        assert!(debug.contains("ProcessAccess"));
        assert!(debug.contains("0x10") || debug.contains("16"));
    }
}
