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
}
