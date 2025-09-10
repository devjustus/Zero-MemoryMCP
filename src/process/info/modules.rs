//! Module enumeration and information retrieval

use crate::core::types::{Address, MemoryError, MemoryResult, ModuleInfo, ProcessId};
use crate::process::ProcessHandle;
use crate::windows::utils::string_conv::wide_to_string;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::{DWORD, FALSE, HMODULE, MAX_PATH};
use winapi::um::psapi::{
    EnumProcessModules, GetModuleBaseNameW, GetModuleFileNameExW, GetModuleInformation, MODULEINFO,
};

/// Enumerates modules loaded in a process
pub struct ModuleEnumerator {
    handle: ProcessHandle,
}

impl ModuleEnumerator {
    /// Create a new module enumerator for a process
    pub fn new(handle: ProcessHandle) -> Self {
        ModuleEnumerator { handle }
    }

    /// Enumerate all modules in the process
    pub fn enumerate(&self) -> MemoryResult<Vec<ModuleInfo>> {
        // First, get the count of modules
        let mut modules: Vec<HMODULE> = Vec::with_capacity(1024);
        let mut cb_needed: DWORD = 0;

        unsafe {
            // Initial call to get the required buffer size
            let result = EnumProcessModules(
                self.handle.raw(),
                modules.as_mut_ptr(),
                (modules.capacity() * mem::size_of::<HMODULE>()) as DWORD,
                &mut cb_needed,
            );

            if result == FALSE {
                return Err(MemoryError::WindowsApi(
                    "Failed to enumerate process modules".to_string(),
                ));
            }

            // Calculate the actual number of modules
            let module_count = cb_needed as usize / mem::size_of::<HMODULE>();
            modules.set_len(module_count);
        }

        // Now get information for each module
        let mut module_infos = Vec::with_capacity(modules.len());
        for &module in &modules {
            if let Ok(info) = self.get_module_info(module) {
                module_infos.push(info);
            }
        }

        Ok(module_infos)
    }

    /// Get information about a specific module
    fn get_module_info(&self, module: HMODULE) -> MemoryResult<ModuleInfo> {
        unsafe {
            // Get module base name
            let mut base_name: [u16; MAX_PATH] = [0; MAX_PATH];
            let name_len = GetModuleBaseNameW(
                self.handle.raw(),
                module,
                base_name.as_mut_ptr(),
                MAX_PATH as DWORD,
            );

            if name_len == 0 {
                return Err(MemoryError::WindowsApi(
                    "Failed to get module base name".to_string(),
                ));
            }

            let name = wide_to_string(&base_name[..name_len as usize]);

            // Get module file path
            let mut file_path: [u16; MAX_PATH] = [0; MAX_PATH];
            let path_len = GetModuleFileNameExW(
                self.handle.raw(),
                module,
                file_path.as_mut_ptr(),
                MAX_PATH as DWORD,
            );

            let path = if path_len > 0 {
                Some(wide_to_string(&file_path[..path_len as usize]))
            } else {
                None
            };

            // Get module information (base address and size)
            let mut mod_info: MODULEINFO = mem::zeroed();
            let result = GetModuleInformation(
                self.handle.raw(),
                module,
                &mut mod_info,
                mem::size_of::<MODULEINFO>() as DWORD,
            );

            if result == FALSE {
                return Err(MemoryError::WindowsApi(
                    "Failed to get module information".to_string(),
                ));
            }

            let mut module_info = ModuleInfo::new(
                name,
                Address::from(mod_info.lpBaseOfDll as usize),
                mod_info.SizeOfImage as usize,
            );

            // Set the path if available
            if let Some(path_str) = path {
                module_info.path = std::path::PathBuf::from(path_str);
            }

            Ok(module_info)
        }
    }

    /// Find a module by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> MemoryResult<Option<ModuleInfo>> {
        let modules = self.enumerate()?;
        let name_lower = name.to_lowercase();

        Ok(modules
            .into_iter()
            .find(|m| m.name.to_lowercase() == name_lower))
    }

    /// Get the main module (executable) of the process
    pub fn get_main_module(&self) -> MemoryResult<ModuleInfo> {
        // The main module is always the first one
        let modules = self.enumerate()?;
        modules
            .into_iter()
            .next()
            .ok_or_else(|| MemoryError::InvalidAddress("No main module found".to_string()))
    }
}

/// Enumerate modules for a specific process
pub fn enumerate_modules(pid: ProcessId) -> MemoryResult<Vec<ModuleInfo>> {
    let handle = ProcessHandle::open_for_read(pid)?;
    let enumerator = ModuleEnumerator::new(handle);
    enumerator.enumerate()
}

/// Find a module by name in a specific process
pub fn find_module_by_name(pid: ProcessId, name: &str) -> MemoryResult<Option<ModuleInfo>> {
    let handle = ProcessHandle::open_for_read(pid)?;
    let enumerator = ModuleEnumerator::new(handle);
    enumerator.find_by_name(name)
}

/// Get the main module of a process
pub fn get_process_main_module(pid: ProcessId) -> MemoryResult<ModuleInfo> {
    let handle = ProcessHandle::open_for_read(pid)?;
    let enumerator = ModuleEnumerator::new(handle);
    enumerator.get_main_module()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_module_enumerator_creation() {
        // Get current process handle
        let handle =
            ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
        let enumerator = ModuleEnumerator::new(handle);
        // Just ensure it creates without panic
        drop(enumerator);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enumerate_current_process_modules() {
        let handle =
            ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
        let enumerator = ModuleEnumerator::new(handle);

        let result = enumerator.enumerate();
        assert!(result.is_ok());

        let modules = result.unwrap();
        // Current process should have at least one module (the executable)
        assert!(!modules.is_empty());

        // The first module should be the main executable
        let main_module = &modules[0];
        assert!(!main_module.name.is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_get_main_module() {
        let handle =
            ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
        let enumerator = ModuleEnumerator::new(handle);

        let result = enumerator.get_main_module();
        assert!(result.is_ok());

        let main_module = result.unwrap();
        assert!(!main_module.name.is_empty());
        assert!(main_module.size > 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_module_by_name() {
        let handle =
            ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
        let enumerator = ModuleEnumerator::new(handle);

        // Try to find kernel32.dll (should be loaded in every Windows process)
        let result = enumerator.find_by_name("kernel32.dll");
        assert!(result.is_ok());

        if let Some(module) = result.unwrap() {
            assert_eq!(module.name.to_lowercase(), "kernel32.dll");
            assert!(module.size > 0);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enumerate_modules_helper() {
        let pid = process::id();
        let result = enumerate_modules(pid);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_module_by_name_helper() {
        let pid = process::id();
        let result = find_module_by_name(pid, "ntdll.dll");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_get_process_main_module_helper() {
        let pid = process::id();
        let result = get_process_main_module(pid);
        assert!(result.is_ok());
        assert!(!result.unwrap().name.is_empty());
    }
}
