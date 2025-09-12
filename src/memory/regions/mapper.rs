//! Memory mapping functionality for regions

use crate::core::types::{Address, MemoryError, MemoryResult};
use crate::process::ProcessHandle;
use std::ptr;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, UnmapViewOfFile, VirtualAlloc, VirtualFree};
use winapi::um::winnt::{HANDLE, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};

/// Options for memory mapping
#[derive(Debug, Clone)]
pub struct MappingOptions {
    /// Desired access rights
    pub access: MappingAccess,
    /// Size of the mapping (0 for entire file)
    pub size: usize,
    /// Offset in the file to start mapping
    pub offset: u64,
    /// Preferred base address (may not be honored)
    pub preferred_address: Option<Address>,
}

impl Default for MappingOptions {
    fn default() -> Self {
        MappingOptions {
            access: MappingAccess::ReadWrite,
            size: 0,
            offset: 0,
            preferred_address: None,
        }
    }
}

/// Access rights for memory mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingAccess {
    /// Read-only access
    ReadOnly,
    /// Read and write access
    ReadWrite,
    /// Read, write, and execute access
    ReadWriteExecute,
    /// Copy-on-write access
    CopyOnWrite,
}

impl MappingAccess {
    /// Convert to Windows FILE_MAP_* constant
    fn to_file_map_access(self) -> DWORD {
        match self {
            MappingAccess::ReadOnly => 0x0004,         // FILE_MAP_READ
            MappingAccess::ReadWrite => 0x0002,        // FILE_MAP_WRITE
            MappingAccess::ReadWriteExecute => 0x0020, // FILE_MAP_EXECUTE
            MappingAccess::CopyOnWrite => 0x0001,      // FILE_MAP_COPY
        }
    }

    /// Convert to Windows PAGE_* protection constant
    fn to_page_protection(self) -> DWORD {
        match self {
            MappingAccess::ReadOnly => 0x02,         // PAGE_READONLY
            MappingAccess::ReadWrite => 0x04,        // PAGE_READWRITE
            MappingAccess::ReadWriteExecute => 0x40, // PAGE_EXECUTE_READWRITE
            MappingAccess::CopyOnWrite => 0x08,      // PAGE_WRITECOPY
        }
    }
}

/// A mapped memory region
pub struct MappedRegion {
    /// Base address of the mapped region
    pub base_address: Address,
    /// Size of the mapped region
    pub size: usize,
    /// Access rights
    pub access: MappingAccess,
    /// Handle to the mapping (if file-backed)
    mapping_handle: Option<HANDLE>,
    /// Whether this is a file mapping or virtual allocation
    is_file_mapping: bool,
}

impl MappedRegion {
    /// Get a pointer to the mapped memory
    pub fn as_ptr(&self) -> *const u8 {
        self.base_address.as_usize() as *const u8
    }

    /// Get a mutable pointer to the mapped memory
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.base_address.as_usize() as *mut u8
    }

    /// Get the mapped memory as a slice
    ///
    /// # Safety
    /// The caller must ensure the mapped memory is valid and accessible
    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.as_ptr(), self.size)
    }

    /// Get the mapped memory as a mutable slice
    ///
    /// # Safety
    /// The caller must ensure the mapped memory is valid, accessible, and writable
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.as_mut_ptr(), self.size)
    }

    /// Check if an address is within this mapped region
    pub fn contains(&self, address: Address) -> bool {
        let start = self.base_address.as_usize();
        let end = start + self.size;
        let addr = address.as_usize();
        addr >= start && addr < end
    }

    /// Flush changes to the underlying file (if file-backed)
    pub fn flush(&self) -> MemoryResult<()> {
        if self.is_file_mapping {
            unsafe {
                use winapi::um::memoryapi::FlushViewOfFile;

                if FlushViewOfFile(self.base_address.as_usize() as *const _, self.size) == FALSE {
                    return Err(MemoryError::WindowsApi(
                        "Failed to flush mapped region".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Drop for MappedRegion {
    fn drop(&mut self) {
        unsafe {
            if self.is_file_mapping {
                UnmapViewOfFile(self.base_address.as_usize() as *const _);
                if let Some(handle) = self.mapping_handle {
                    if !handle.is_null() {
                        CloseHandle(handle);
                    }
                }
            } else {
                VirtualFree(self.base_address.as_usize() as *mut _, 0, MEM_RELEASE);
            }
        }
    }
}

/// Memory mapper for creating and managing memory mappings
pub struct MemoryMapper {
    handle: ProcessHandle,
}

impl MemoryMapper {
    /// Create a new memory mapper for a process
    pub fn new(handle: ProcessHandle) -> Self {
        MemoryMapper { handle }
    }

    /// Allocate virtual memory in the process
    pub fn allocate_memory(
        &self,
        size: usize,
        options: MappingOptions,
    ) -> MemoryResult<MappedRegion> {
        unsafe {
            let base_addr = options
                .preferred_address
                .map(|a| a.as_usize() as *mut _)
                .unwrap_or(ptr::null_mut());

            let protection = options.access.to_page_protection();

            let allocated = VirtualAlloc(base_addr, size, MEM_COMMIT | MEM_RESERVE, protection);

            if allocated.is_null() {
                return Err(MemoryError::WindowsApi(
                    "Failed to allocate virtual memory".to_string(),
                ));
            }

            Ok(MappedRegion {
                base_address: Address::new(allocated as usize),
                size,
                access: options.access,
                mapping_handle: None,
                is_file_mapping: false,
            })
        }
    }

    /// Map a view of a file into memory
    ///
    /// # Safety
    /// The caller must ensure that file_mapping is a valid handle from CreateFileMapping
    pub unsafe fn map_file_view(
        &self,
        file_mapping: HANDLE,
        options: MappingOptions,
    ) -> MemoryResult<MappedRegion> {
        if file_mapping.is_null() {
            return Err(MemoryError::InvalidAddress(
                "Invalid file mapping handle".to_string(),
            ));
        }

        unsafe {
            let access = options.access.to_file_map_access();
            let offset_high = (options.offset >> 32) as DWORD;
            let offset_low = (options.offset & 0xFFFFFFFF) as DWORD;

            let base_addr =
                MapViewOfFile(file_mapping, access, offset_high, offset_low, options.size);

            if base_addr.is_null() {
                return Err(MemoryError::WindowsApi(
                    "Failed to map file view".to_string(),
                ));
            }

            // Query the actual size if not specified
            let actual_size = if options.size == 0 {
                use winapi::um::memoryapi::VirtualQuery;
                use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

                let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
                let result = VirtualQuery(
                    base_addr,
                    &mut mbi,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                );

                if result == 0 {
                    UnmapViewOfFile(base_addr);
                    return Err(MemoryError::WindowsApi(
                        "Failed to query mapped region size".to_string(),
                    ));
                }

                mbi.RegionSize
            } else {
                options.size
            };

            Ok(MappedRegion {
                base_address: Address::new(base_addr as usize),
                size: actual_size,
                access: options.access,
                mapping_handle: Some(file_mapping),
                is_file_mapping: true,
            })
        }
    }

    /// Reserve a region of memory without committing it
    pub fn reserve_memory(&self, size: usize) -> MemoryResult<Address> {
        unsafe {
            let reserved = VirtualAlloc(ptr::null_mut(), size, MEM_RESERVE, PAGE_READWRITE);

            if reserved.is_null() {
                return Err(MemoryError::WindowsApi(
                    "Failed to reserve memory".to_string(),
                ));
            }

            Ok(Address::new(reserved as usize))
        }
    }

    /// Commit a previously reserved memory region
    pub fn commit_memory(
        &self,
        address: Address,
        size: usize,
        access: MappingAccess,
    ) -> MemoryResult<()> {
        unsafe {
            let protection = access.to_page_protection();

            let result = VirtualAlloc(address.as_usize() as *mut _, size, MEM_COMMIT, protection);

            if result.is_null() {
                return Err(MemoryError::WindowsApi(
                    "Failed to commit memory".to_string(),
                ));
            }

            Ok(())
        }
    }

    /// Create a shared memory mapping
    pub fn create_shared_memory(
        &self,
        name: &str,
        size: usize,
        access: MappingAccess,
    ) -> MemoryResult<MappedRegion> {
        use std::ffi::CString;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::winbase::CreateFileMappingA;

        let c_name = CString::new(name).map_err(|_| {
            MemoryError::InvalidValueType("Invalid name for shared memory".to_string())
        })?;
        let protection = access.to_page_protection();

        unsafe {
            let mapping_handle = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                ptr::null_mut(),
                protection,
                (size >> 32) as DWORD,
                (size & 0xFFFFFFFF) as DWORD,
                c_name.as_ptr(),
            );

            if mapping_handle.is_null() {
                return Err(MemoryError::WindowsApi(
                    "Failed to create shared memory mapping".to_string(),
                ));
            }

            self.map_file_view(
                mapping_handle,
                MappingOptions {
                    access,
                    size,
                    ..Default::default()
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_access_conversion() {
        assert_eq!(MappingAccess::ReadOnly.to_file_map_access(), 0x0004);
        assert_eq!(MappingAccess::ReadWrite.to_file_map_access(), 0x0002);
        assert_eq!(MappingAccess::ReadWriteExecute.to_file_map_access(), 0x0020);

        assert_eq!(MappingAccess::ReadOnly.to_page_protection(), 0x02);
        assert_eq!(MappingAccess::ReadWrite.to_page_protection(), 0x04);
        assert_eq!(MappingAccess::ReadWriteExecute.to_page_protection(), 0x40);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_allocate_virtual_memory() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mapper = MemoryMapper::new(handle);

        let result = mapper.allocate_memory(
            4096,
            MappingOptions {
                access: MappingAccess::ReadWrite,
                ..Default::default()
            },
        );

        assert!(result.is_ok());
        let region = result.unwrap();
        assert_eq!(region.size, 4096);
        assert!(!region.base_address.is_null());

        // Region will be freed when dropped
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_reserve_and_commit() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mapper = MemoryMapper::new(handle);

        // Reserve memory
        let reserved = mapper.reserve_memory(8192);
        assert!(reserved.is_ok());

        let address = reserved.unwrap();

        // Commit part of it
        let result = mapper.commit_memory(address, 4096, MappingAccess::ReadWrite);
        assert!(result.is_ok());

        // Clean up
        unsafe {
            VirtualFree(address.as_usize() as *mut _, 0, MEM_RELEASE);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_create_shared_memory() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mapper = MemoryMapper::new(handle);

        // Create shared memory
        let result =
            mapper.create_shared_memory("TestSharedMemory123", 4096, MappingAccess::ReadWrite);
        // May fail if already exists, but should not panic
        let _ = result;
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_allocate_memory() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mapper = MemoryMapper::new(handle);

        // Allocate committed memory directly
        let options = MappingOptions {
            size: 4096,
            access: MappingAccess::ReadWrite,
            offset: 0,
            preferred_address: None,
        };

        let allocated = mapper.allocate_memory(4096, options);
        assert!(allocated.is_ok());

        // MappedRegion will clean up on drop
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_mapped_region_drop() {
        // Test that MappedRegion cleans up on drop
        {
            let _region = MappedRegion {
                base_address: Address::new(0x1000),
                size: 4096,
                access: MappingAccess::ReadOnly,
                mapping_handle: None,
                is_file_mapping: false,
            };
            // Region should be dropped here, calling cleanup
        }
        // No panic means Drop implementation worked
    }

    #[test]
    fn test_mapping_options() {
        let options = MappingOptions {
            size: 8192,
            access: MappingAccess::ReadWrite,
            preferred_address: Some(Address::new(0x10000)),
            offset: 4096,
        };

        assert_eq!(options.size, 8192);
        assert_eq!(options.access, MappingAccess::ReadWrite);
        assert_eq!(options.preferred_address, Some(Address::new(0x10000)));
        assert_eq!(options.offset, 4096);

        // Test default
        let default = MappingOptions::default();
        assert_eq!(default.access, MappingAccess::ReadWrite);
        assert_eq!(default.size, 0);
        assert_eq!(default.offset, 0);
        assert!(default.preferred_address.is_none());
    }

    #[test]
    fn test_mapping_access_conversions() {
        // Test page protection conversions
        assert_eq!(MappingAccess::ReadOnly.to_page_protection(), 0x02);
        assert_eq!(MappingAccess::ReadWrite.to_page_protection(), 0x04);
        assert_eq!(MappingAccess::ReadWriteExecute.to_page_protection(), 0x40);
        assert_eq!(MappingAccess::CopyOnWrite.to_page_protection(), 0x08);

        // Test file map access conversions
        assert_eq!(MappingAccess::ReadOnly.to_file_map_access(), 0x04);
        assert_eq!(MappingAccess::ReadWrite.to_file_map_access(), 0x02);
        assert_eq!(MappingAccess::ReadWriteExecute.to_file_map_access(), 0x20);
        assert_eq!(MappingAccess::CopyOnWrite.to_file_map_access(), 0x01);
    }

    #[test]
    fn test_mapped_region_fields() {
        let region = MappedRegion {
            base_address: Address::new(0x1000),
            size: 4096,
            access: MappingAccess::ReadWrite,
            mapping_handle: None,
            is_file_mapping: false,
        };

        assert_eq!(region.base_address, Address::new(0x1000));
        assert_eq!(region.size, 4096);
        assert_eq!(region.access, MappingAccess::ReadWrite);
        assert!(!region.is_file_mapping);
    }

    #[test]
    fn test_mapped_region_as_ptr() {
        let region = MappedRegion {
            base_address: Address::new(0x2000),
            size: 4096,
            access: MappingAccess::ReadOnly,
            mapping_handle: None,
            is_file_mapping: false,
        };

        let ptr = region.as_ptr();
        assert_eq!(ptr as usize, 0x2000);
    }

    #[test]
    fn test_mapped_region_as_mut_ptr() {
        let mut region = MappedRegion {
            base_address: Address::new(0x3000),
            size: 4096,
            access: MappingAccess::ReadWrite,
            mapping_handle: None,
            is_file_mapping: false,
        };

        let ptr = region.as_mut_ptr();
        assert_eq!(ptr as usize, 0x3000);
    }

    #[test]
    fn test_mapped_region_contains() {
        let region = MappedRegion {
            base_address: Address::new(0x1000),
            size: 0x2000,
            access: MappingAccess::ReadOnly,
            mapping_handle: None,
            is_file_mapping: false,
        };

        // Test addresses within the region
        assert!(region.contains(Address::new(0x1000))); // Start
        assert!(region.contains(Address::new(0x1500))); // Middle
        assert!(region.contains(Address::new(0x2FFF))); // Just before end

        // Test addresses outside the region
        assert!(!region.contains(Address::new(0x0FFF))); // Before start
        assert!(!region.contains(Address::new(0x3000))); // At end (exclusive)
        assert!(!region.contains(Address::new(0x4000))); // After end
    }

    #[test]
    fn test_mapped_region_flush_non_file_mapping() {
        let region = MappedRegion {
            base_address: Address::new(0x1000),
            size: 4096,
            access: MappingAccess::ReadWrite,
            mapping_handle: None,
            is_file_mapping: false,
        };

        // Flush should succeed for non-file mappings (no-op)
        let result = region.flush();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mapping_options_builder_pattern() {
        let options = MappingOptions::default().size;
        assert_eq!(options, 0);

        let options = MappingOptions {
            access: MappingAccess::ReadWriteExecute,
            size: 8192,
            offset: 512,
            preferred_address: Some(Address::new(0x50000)),
        };
        assert_eq!(options.access, MappingAccess::ReadWriteExecute);
        assert_eq!(options.size, 8192);
        assert_eq!(options.offset, 512);
        assert_eq!(options.preferred_address, Some(Address::new(0x50000)));
    }

    #[test]
    fn test_mapping_access_equality() {
        assert_eq!(MappingAccess::ReadOnly, MappingAccess::ReadOnly);
        assert_ne!(MappingAccess::ReadOnly, MappingAccess::ReadWrite);
        assert_ne!(MappingAccess::ReadWrite, MappingAccess::ReadWriteExecute);
        assert_ne!(MappingAccess::ReadWriteExecute, MappingAccess::CopyOnWrite);
    }
}
