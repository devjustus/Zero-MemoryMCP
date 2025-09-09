//! Memory region information wrapper

use crate::core::types::Address;
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// Wrapper for MEMORY_BASIC_INFORMATION
#[derive(Debug, Clone)]
pub struct MemoryBasicInfo {
    pub base_address: Address,
    pub allocation_base: Address,
    pub allocation_protect: u32,
    pub region_size: usize,
    pub state: u32,
    pub protect: u32,
    pub type_flags: u32,
}

impl From<MEMORY_BASIC_INFORMATION> for MemoryBasicInfo {
    fn from(mbi: MEMORY_BASIC_INFORMATION) -> Self {
        MemoryBasicInfo {
            base_address: Address::new(mbi.BaseAddress as usize),
            allocation_base: Address::new(mbi.AllocationBase as usize),
            allocation_protect: mbi.AllocationProtect,
            region_size: mbi.RegionSize,
            state: mbi.State,
            protect: mbi.Protect,
            type_flags: mbi.Type,
        }
    }
}

impl MemoryBasicInfo {
    /// Check if memory is committed
    pub fn is_committed(&self) -> bool {
        const MEM_COMMIT: u32 = 0x1000;
        self.state == MEM_COMMIT
    }

    /// Check if memory is readable
    pub fn is_readable(&self) -> bool {
        const PAGE_NOACCESS: u32 = 0x01;
        const PAGE_GUARD: u32 = 0x100;

        self.protect != PAGE_NOACCESS && (self.protect & PAGE_GUARD) == 0
    }

    /// Check if memory is writable
    pub fn is_writable(&self) -> bool {
        const PAGE_READWRITE: u32 = 0x04;
        const PAGE_WRITECOPY: u32 = 0x08;
        const PAGE_EXECUTE_READWRITE: u32 = 0x40;
        const PAGE_EXECUTE_WRITECOPY: u32 = 0x80;

        (self.protect
            & (PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY))
            != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_info_flags() {
        let info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000, // MEM_COMMIT
            protect: 0x04, // PAGE_READWRITE
            type_flags: 0x20000,
        };

        assert!(info.is_committed());
        assert!(info.is_readable());
        assert!(info.is_writable());
    }

    #[test]
    fn test_from_memory_basic_information() {
        use std::mem;

        let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { mem::zeroed() };
        mbi.BaseAddress = 0x2000 as *mut _;
        mbi.AllocationBase = 0x1000 as *mut _;
        mbi.AllocationProtect = 0x20;
        mbi.RegionSize = 8192;
        mbi.State = 0x2000; // MEM_RESERVE
        mbi.Protect = 0x01; // PAGE_NOACCESS
        mbi.Type = 0x40000;

        let info = MemoryBasicInfo::from(mbi);
        assert_eq!(info.base_address, Address::new(0x2000));
        assert_eq!(info.allocation_base, Address::new(0x1000));
        assert_eq!(info.allocation_protect, 0x20);
        assert_eq!(info.region_size, 8192);
        assert_eq!(info.state, 0x2000);
        assert_eq!(info.protect, 0x01);
        assert_eq!(info.type_flags, 0x40000);
    }

    #[test]
    fn test_is_committed() {
        let mut info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000, // MEM_COMMIT
            protect: 0x04,
            type_flags: 0x20000,
        };

        assert!(info.is_committed());

        // Test with MEM_RESERVE
        info.state = 0x2000;
        assert!(!info.is_committed());

        // Test with MEM_FREE
        info.state = 0x10000;
        assert!(!info.is_committed());
    }

    #[test]
    fn test_is_readable() {
        let mut info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000,
            protect: 0x04, // PAGE_READWRITE
            type_flags: 0x20000,
        };

        assert!(info.is_readable());

        // Test with PAGE_NOACCESS
        info.protect = 0x01;
        assert!(!info.is_readable());

        // Test with PAGE_GUARD
        info.protect = 0x104; // PAGE_READWRITE | PAGE_GUARD
        assert!(!info.is_readable());

        // Test with PAGE_READONLY
        info.protect = 0x02;
        assert!(info.is_readable());

        // Test with PAGE_EXECUTE_READ
        info.protect = 0x20;
        assert!(info.is_readable());
    }

    #[test]
    fn test_is_writable() {
        let mut info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000,
            protect: 0x04, // PAGE_READWRITE
            type_flags: 0x20000,
        };

        assert!(info.is_writable());

        // Test with PAGE_READONLY
        info.protect = 0x02;
        assert!(!info.is_writable());

        // Test with PAGE_WRITECOPY
        info.protect = 0x08;
        assert!(info.is_writable());

        // Test with PAGE_EXECUTE_READWRITE
        info.protect = 0x40;
        assert!(info.is_writable());

        // Test with PAGE_EXECUTE_WRITECOPY
        info.protect = 0x80;
        assert!(info.is_writable());

        // Test with PAGE_NOACCESS
        info.protect = 0x01;
        assert!(!info.is_writable());

        // Test with PAGE_EXECUTE
        info.protect = 0x10;
        assert!(!info.is_writable());
    }

    #[test]
    fn test_memory_info_clone() {
        let info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000,
            protect: 0x04,
            type_flags: 0x20000,
        };

        let cloned = info.clone();
        assert_eq!(cloned.base_address, info.base_address);
        assert_eq!(cloned.allocation_base, info.allocation_base);
        assert_eq!(cloned.region_size, info.region_size);
    }

    #[test]
    fn test_memory_info_debug() {
        let info = MemoryBasicInfo {
            base_address: Address::new(0x1000),
            allocation_base: Address::new(0x1000),
            allocation_protect: 0x04,
            region_size: 4096,
            state: 0x1000,
            protect: 0x04,
            type_flags: 0x20000,
        };

        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("MemoryBasicInfo"));
        assert!(debug_str.contains("base_address"));
    }
}
