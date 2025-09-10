//! Memory protection management

use crate::core::types::{Address, MemoryError, MemoryResult};
use crate::process::ProcessHandle;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::memoryapi::VirtualProtectEx;

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionFlags {
    value: u32,
}

impl ProtectionFlags {
    // Protection constants
    pub const PAGE_NOACCESS: u32 = 0x01;
    pub const PAGE_READONLY: u32 = 0x02;
    pub const PAGE_READWRITE: u32 = 0x04;
    pub const PAGE_WRITECOPY: u32 = 0x08;
    pub const PAGE_EXECUTE: u32 = 0x10;
    pub const PAGE_EXECUTE_READ: u32 = 0x20;
    pub const PAGE_EXECUTE_READWRITE: u32 = 0x40;
    pub const PAGE_EXECUTE_WRITECOPY: u32 = 0x80;
    pub const PAGE_GUARD: u32 = 0x100;
    pub const PAGE_NOCACHE: u32 = 0x200;
    pub const PAGE_WRITECOMBINE: u32 = 0x400;

    /// Create new protection flags
    pub fn new(value: u32) -> Self {
        ProtectionFlags { value }
    }

    /// No access protection
    pub fn no_access() -> Self {
        ProtectionFlags::new(Self::PAGE_NOACCESS)
    }

    /// Read-only protection
    pub fn read_only() -> Self {
        ProtectionFlags::new(Self::PAGE_READONLY)
    }

    /// Read-write protection
    pub fn read_write() -> Self {
        ProtectionFlags::new(Self::PAGE_READWRITE)
    }

    /// Execute-only protection
    pub fn execute() -> Self {
        ProtectionFlags::new(Self::PAGE_EXECUTE)
    }

    /// Execute-read protection
    pub fn execute_read() -> Self {
        ProtectionFlags::new(Self::PAGE_EXECUTE_READ)
    }

    /// Execute-read-write protection
    pub fn execute_read_write() -> Self {
        ProtectionFlags::new(Self::PAGE_EXECUTE_READWRITE)
    }

    /// Check if protection allows reading
    pub fn is_readable(&self) -> bool {
        self.value != Self::PAGE_NOACCESS && self.value != Self::PAGE_EXECUTE
    }

    /// Check if protection allows writing
    pub fn is_writable(&self) -> bool {
        (self.value
            & (Self::PAGE_READWRITE
                | Self::PAGE_WRITECOPY
                | Self::PAGE_EXECUTE_READWRITE
                | Self::PAGE_EXECUTE_WRITECOPY))
            != 0
    }

    /// Check if protection allows execution
    pub fn is_executable(&self) -> bool {
        (self.value
            & (Self::PAGE_EXECUTE
                | Self::PAGE_EXECUTE_READ
                | Self::PAGE_EXECUTE_READWRITE
                | Self::PAGE_EXECUTE_WRITECOPY))
            != 0
    }

    /// Check if guard page flag is set
    pub fn is_guard(&self) -> bool {
        (self.value & Self::PAGE_GUARD) != 0
    }

    /// Add guard page flag
    pub fn with_guard(mut self) -> Self {
        self.value |= Self::PAGE_GUARD;
        self
    }

    /// Remove guard page flag
    pub fn without_guard(mut self) -> Self {
        self.value &= !Self::PAGE_GUARD;
        self
    }

    /// Check if no-cache flag is set
    pub fn is_no_cache(&self) -> bool {
        (self.value & Self::PAGE_NOCACHE) != 0
    }

    /// Add no-cache flag
    pub fn with_no_cache(mut self) -> Self {
        self.value |= Self::PAGE_NOCACHE;
        self
    }

    /// Get the raw protection value
    pub fn raw(&self) -> u32 {
        self.value
    }

    /// Convert to human-readable string
    fn format_string(&self) -> String {
        let base = match self.value & 0xFF {
            Self::PAGE_NOACCESS => "NOACCESS",
            Self::PAGE_READONLY => "R",
            Self::PAGE_READWRITE => "RW",
            Self::PAGE_WRITECOPY => "WC",
            Self::PAGE_EXECUTE => "X",
            Self::PAGE_EXECUTE_READ => "RX",
            Self::PAGE_EXECUTE_READWRITE => "RWX",
            Self::PAGE_EXECUTE_WRITECOPY => "WCX",
            _ => "UNKNOWN",
        };

        let mut flags = String::from(base);

        if self.is_guard() {
            flags.push_str("+G");
        }
        if self.is_no_cache() {
            flags.push_str("+NC");
        }

        flags
    }
}

impl std::fmt::Display for ProtectionFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_string())
    }
}

/// Protection change result
#[derive(Debug)]
pub struct ProtectionChange {
    /// Address where protection was changed
    pub address: Address,
    /// Size of the region changed
    pub size: usize,
    /// Old protection flags
    pub old_protection: ProtectionFlags,
    /// New protection flags
    pub new_protection: ProtectionFlags,
}

/// Manages memory protection for a process
pub struct ProtectionManager {
    handle: ProcessHandle,
}

impl ProtectionManager {
    /// Create a new protection manager
    pub fn new(handle: ProcessHandle) -> Self {
        ProtectionManager { handle }
    }

    /// Change memory protection for a region
    pub fn change_protection(
        &self,
        address: Address,
        size: usize,
        new_protection: ProtectionFlags,
    ) -> MemoryResult<ProtectionChange> {
        if size == 0 {
            return Err(MemoryError::InvalidValueType(
                "Size cannot be zero".to_string(),
            ));
        }

        unsafe {
            let mut old_protection: DWORD = 0;

            let result = VirtualProtectEx(
                self.handle.raw(),
                address.as_usize() as *mut _,
                size,
                new_protection.raw(),
                &mut old_protection,
            );

            if result == FALSE {
                return Err(MemoryError::ProtectionError(format!(
                    "Failed to change protection at {:#x}",
                    address.as_usize()
                )));
            }

            Ok(ProtectionChange {
                address,
                size,
                old_protection: ProtectionFlags::new(old_protection),
                new_protection,
            })
        }
    }

    /// Temporarily remove protection for an operation
    pub fn unprotect_for_operation<F, R>(
        &self,
        address: Address,
        size: usize,
        operation: F,
    ) -> MemoryResult<R>
    where
        F: FnOnce() -> MemoryResult<R>,
    {
        // Change to read-write
        let change = self.change_protection(address, size, ProtectionFlags::read_write())?;

        // Perform the operation
        let result = operation();

        // Restore original protection
        let _ = self.change_protection(address, size, change.old_protection);

        result
    }

    /// Add guard page protection to a region
    pub fn add_guard_page(&self, address: Address, size: usize) -> MemoryResult<()> {
        // Get current protection
        let info = crate::memory::regions::query_region_at(address)?;
        let current = ProtectionFlags::new(info.protection);

        // Add guard flag
        let new_protection = current.with_guard();

        self.change_protection(address, size, new_protection)?;
        Ok(())
    }

    /// Remove guard page protection from a region
    pub fn remove_guard_page(&self, address: Address, size: usize) -> MemoryResult<()> {
        // Get current protection
        let info = crate::memory::regions::query_region_at(address)?;
        let current = ProtectionFlags::new(info.protection);

        // Remove guard flag
        let new_protection = current.without_guard();

        self.change_protection(address, size, new_protection)?;
        Ok(())
    }

    /// Make a region executable
    pub fn make_executable(&self, address: Address, size: usize) -> MemoryResult<ProtectionChange> {
        // Get current protection to determine if it's readable/writable
        let info = crate::memory::regions::query_region_at(address)?;
        let current = ProtectionFlags::new(info.protection);

        let new_protection = if current.is_writable() {
            ProtectionFlags::execute_read_write()
        } else if current.is_readable() {
            ProtectionFlags::execute_read()
        } else {
            ProtectionFlags::execute()
        };

        self.change_protection(address, size, new_protection)
    }

    /// Make a region non-executable
    pub fn make_non_executable(
        &self,
        address: Address,
        size: usize,
    ) -> MemoryResult<ProtectionChange> {
        let info = crate::memory::regions::query_region_at(address)?;
        let current = ProtectionFlags::new(info.protection);

        let new_protection = if current.is_writable() {
            ProtectionFlags::read_write()
        } else if current.is_readable() {
            ProtectionFlags::read_only()
        } else {
            ProtectionFlags::no_access()
        };

        self.change_protection(address, size, new_protection)
    }
}

/// Change protection for a memory region in the current process
pub fn change_protection(
    address: Address,
    size: usize,
    new_protection: ProtectionFlags,
) -> MemoryResult<ProtectionChange> {
    let handle = ProcessHandle::open_for_read_write(std::process::id())?;
    let manager = ProtectionManager::new(handle);
    manager.change_protection(address, size, new_protection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_flags() {
        let flags = ProtectionFlags::read_write();
        assert!(flags.is_readable());
        assert!(flags.is_writable());
        assert!(!flags.is_executable());
        assert!(!flags.is_guard());

        let exec_flags = ProtectionFlags::execute_read_write();
        assert!(exec_flags.is_readable());
        assert!(exec_flags.is_writable());
        assert!(exec_flags.is_executable());

        let guard_flags = flags.with_guard();
        assert!(guard_flags.is_guard());
        assert!(guard_flags.is_readable());
    }

    #[test]
    fn test_protection_string_conversion() {
        assert_eq!(format!("{}", ProtectionFlags::read_only()), "R");
        assert_eq!(format!("{}", ProtectionFlags::read_write()), "RW");
        assert_eq!(format!("{}", ProtectionFlags::execute_read_write()), "RWX");

        let guard_rw = ProtectionFlags::read_write().with_guard();
        assert_eq!(format!("{}", guard_rw), "RW+G");
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_change_protection() {
        use std::ptr;
        use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
        use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE};

        unsafe {
            // Allocate some memory
            let mem = VirtualAlloc(
                ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                ProtectionFlags::PAGE_READWRITE,
            );

            if !mem.is_null() {
                let address = Address::new(mem as usize);

                // Try to change protection
                let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
                let manager = ProtectionManager::new(handle);

                let result = manager.change_protection(address, 4096, ProtectionFlags::read_only());

                assert!(result.is_ok());

                let change = result.unwrap();
                assert_eq!(change.old_protection.raw(), ProtectionFlags::PAGE_READWRITE);
                assert_eq!(change.new_protection.raw(), ProtectionFlags::PAGE_READONLY);

                // Clean up
                VirtualFree(mem, 0, MEM_RELEASE);
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_unprotect_for_operation() {
        use std::ptr;
        use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
        use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE};

        unsafe {
            let mem = VirtualAlloc(
                ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                ProtectionFlags::PAGE_READONLY,
            );

            if !mem.is_null() {
                let address = Address::new(mem as usize);
                let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
                let manager = ProtectionManager::new(handle);

                // Temporarily make writable for operation
                let result = manager.unprotect_for_operation(address, 4096, || {
                    // Simulate some operation
                    Ok(42)
                });

                assert!(result.is_ok());
                assert_eq!(result.unwrap(), 42);

                VirtualFree(mem, 0, MEM_RELEASE);
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_make_executable() {
        use std::ptr;
        use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
        use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE};

        unsafe {
            let mem = VirtualAlloc(
                ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                ProtectionFlags::PAGE_READWRITE,
            );

            if !mem.is_null() {
                let address = Address::new(mem as usize);
                let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
                let manager = ProtectionManager::new(handle);

                let result = manager.make_executable(address, 4096);
                assert!(result.is_ok());

                let change = result.unwrap();
                assert_eq!(change.new_protection.raw(), ProtectionFlags::PAGE_EXECUTE_READWRITE);

                VirtualFree(mem, 0, MEM_RELEASE);
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_make_non_executable() {
        use std::ptr;
        use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
        use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE};

        unsafe {
            let mem = VirtualAlloc(
                ptr::null_mut(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                ProtectionFlags::PAGE_EXECUTE_READWRITE,
            );

            if !mem.is_null() {
                let address = Address::new(mem as usize);
                let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
                let manager = ProtectionManager::new(handle);

                let result = manager.make_non_executable(address, 4096);
                assert!(result.is_ok());

                let change = result.unwrap();
                assert_eq!(change.new_protection.raw(), ProtectionFlags::PAGE_READWRITE);

                VirtualFree(mem, 0, MEM_RELEASE);
            }
        }
    }

    #[test]
    fn test_protection_flags_with_modifiers() {
        let flags = ProtectionFlags::read_write();
        let with_no_cache = flags.with_no_cache();
        
        assert!(with_no_cache.is_no_cache());
        assert!(with_no_cache.is_readable());
        assert!(with_no_cache.is_writable());
    }

    #[test]
    fn test_protection_flags_combinations() {
        // Test various protection flag combinations
        let no_access = ProtectionFlags::no_access();
        assert!(!no_access.is_readable());
        assert!(!no_access.is_writable());
        assert!(!no_access.is_executable());

        let execute_only = ProtectionFlags::execute();
        assert!(!execute_only.is_readable());
        assert!(!execute_only.is_writable());
        assert!(execute_only.is_executable());

        let execute_read = ProtectionFlags::execute_read();
        assert!(execute_read.is_readable());
        assert!(!execute_read.is_writable());
        assert!(execute_read.is_executable());
    }

    #[test]
    fn test_protection_change_invalid_size() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let manager = ProtectionManager::new(handle);

        // Test with zero size
        let result = manager.change_protection(
            Address::new(0x1000),
            0,
            ProtectionFlags::read_write(),
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MemoryError::InvalidValueType(_)));
    }
}
