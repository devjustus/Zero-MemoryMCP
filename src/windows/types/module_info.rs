//! Windows module information wrapper

use crate::core::types::Address;
use winapi::um::psapi::MODULEINFO;

/// Windows module information
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub base_address: Address,
    pub size: usize,
    pub entry_point: Address,
}

impl ModuleInfo {
    /// Create new module info
    pub fn new(name: String, info: MODULEINFO) -> Self {
        ModuleInfo {
            name,
            base_address: Address::new(info.lpBaseOfDll as usize),
            size: info.SizeOfImage as usize,
            entry_point: Address::new(info.EntryPoint as usize),
        }
    }

    /// Check if an address is within this module
    pub fn contains_address(&self, addr: Address) -> bool {
        let addr_val = addr.as_usize();
        let base = self.base_address.as_usize();
        addr_val >= base && addr_val < base + self.size
    }

    /// Get the end address of the module
    pub fn end_address(&self) -> Address {
        Address::new(self.base_address.as_usize() + self.size)
    }

    /// Check if this is a system module
    pub fn is_system_module(&self) -> bool {
        let lower_name = self.name.to_lowercase();
        lower_name.contains("kernel32")
            || lower_name.contains("ntdll")
            || lower_name.contains("user32")
            || lower_name.contains("advapi32")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_module_info() {
        let info = MODULEINFO {
            lpBaseOfDll: 0x10000 as *mut _,
            SizeOfImage: 0x1000,
            EntryPoint: 0x10100 as *mut _,
        };

        let module = ModuleInfo::new("test.dll".to_string(), info);

        assert_eq!(module.name, "test.dll");
        assert_eq!(module.base_address, Address::new(0x10000));
        assert_eq!(module.size, 0x1000);
        assert_eq!(module.entry_point, Address::new(0x10100));

        assert!(module.contains_address(Address::new(0x10500)));
        assert!(!module.contains_address(Address::new(0x20000)));

        assert_eq!(module.end_address(), Address::new(0x11000));
    }

    #[test]
    fn test_system_module_detection() {
        let info = MODULEINFO {
            lpBaseOfDll: ptr::null_mut(),
            SizeOfImage: 0,
            EntryPoint: ptr::null_mut(),
        };

        let kernel32 = ModuleInfo::new("KERNEL32.DLL".to_string(), info);
        assert!(kernel32.is_system_module());

        let custom = ModuleInfo::new("custom.dll".to_string(), info);
        assert!(!custom.is_system_module());
    }
}
