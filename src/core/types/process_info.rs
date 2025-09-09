//! Process information types

use super::{Address, ProcessId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Information about a running process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub name: String,
    pub path: Option<PathBuf>,
    pub architecture: ProcessArchitecture,
    pub parent_pid: Option<ProcessId>,
    pub thread_count: u32,
    pub handle_count: u32,
    pub working_set_size: u64,
    pub page_file_usage: u64,
    pub session_id: u32,
    pub is_wow64: bool,
}

impl ProcessInfo {
    /// Creates a new ProcessInfo with minimal information
    pub fn new(pid: ProcessId, name: String) -> Self {
        ProcessInfo {
            pid,
            name,
            path: None,
            architecture: ProcessArchitecture::Unknown,
            parent_pid: None,
            thread_count: 0,
            handle_count: 0,
            working_set_size: 0,
            page_file_usage: 0,
            session_id: 0,
            is_wow64: false,
        }
    }

    /// Checks if this is a system process
    pub fn is_system_process(&self) -> bool {
        self.pid == 0 || self.pid == 4
    }

    /// Checks if this is a 32-bit process running on 64-bit Windows
    pub fn is_32bit_on_64bit(&self) -> bool {
        self.is_wow64
    }
}

/// Process architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessArchitecture {
    X86,
    X64,
    Arm,
    Arm64,
    Unknown,
}

impl ProcessArchitecture {
    /// Returns the pointer size for this architecture
    pub fn pointer_size(&self) -> usize {
        match self {
            ProcessArchitecture::X86 | ProcessArchitecture::Arm => 4,
            ProcessArchitecture::X64 | ProcessArchitecture::Arm64 => 8,
            ProcessArchitecture::Unknown => std::mem::size_of::<usize>(),
        }
    }

    /// Checks if this is a 64-bit architecture
    pub fn is_64bit(&self) -> bool {
        matches!(self, ProcessArchitecture::X64 | ProcessArchitecture::Arm64)
    }
}

/// Information about a loaded module in a process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: PathBuf,
    pub base_address: Address,
    pub size: usize,
    pub entry_point: Option<Address>,
    pub is_system: bool,
}

impl ModuleInfo {
    /// Creates a new ModuleInfo
    pub fn new(name: String, base_address: Address, size: usize) -> Self {
        ModuleInfo {
            name,
            path: PathBuf::new(),
            base_address,
            size,
            entry_point: None,
            is_system: false,
        }
    }

    /// Gets the end address of the module
    pub fn end_address(&self) -> Address {
        Address::new(self.base_address.as_usize() + self.size)
    }

    /// Checks if an address is within this module
    pub fn contains_address(&self, address: Address) -> bool {
        address >= self.base_address && address < self.end_address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_creation() {
        let proc = ProcessInfo::new(1234, "notepad.exe".to_string());
        assert_eq!(proc.pid, 1234);
        assert_eq!(proc.name, "notepad.exe");
        assert_eq!(proc.path, None);
        assert_eq!(proc.architecture, ProcessArchitecture::Unknown);
        assert_eq!(proc.parent_pid, None);
        assert_eq!(proc.thread_count, 0);
        assert_eq!(proc.handle_count, 0);
        assert_eq!(proc.working_set_size, 0);
        assert_eq!(proc.page_file_usage, 0);
        assert_eq!(proc.session_id, 0);
        assert!(!proc.is_wow64);
    }

    #[test]
    fn test_system_process_detection() {
        let idle = ProcessInfo::new(0, "System Idle Process".to_string());
        assert!(idle.is_system_process());

        let system = ProcessInfo::new(4, "System".to_string());
        assert!(system.is_system_process());

        let normal = ProcessInfo::new(1000, "explorer.exe".to_string());
        assert!(!normal.is_system_process());
    }

    #[test]
    fn test_wow64_detection() {
        let mut proc = ProcessInfo::new(1234, "app.exe".to_string());
        assert!(!proc.is_32bit_on_64bit());

        proc.is_wow64 = true;
        assert!(proc.is_32bit_on_64bit());
    }

    #[test]
    fn test_process_info_full() {
        let mut proc = ProcessInfo::new(5678, "chrome.exe".to_string());
        proc.path = Some(PathBuf::from("C:\\Program Files\\Chrome\\chrome.exe"));
        proc.architecture = ProcessArchitecture::X64;
        proc.parent_pid = Some(1000);
        proc.thread_count = 25;
        proc.handle_count = 500;
        proc.working_set_size = 1024 * 1024 * 100;
        proc.page_file_usage = 1024 * 1024 * 150;
        proc.session_id = 1;
        proc.is_wow64 = false;

        assert_eq!(proc.pid, 5678);
        assert_eq!(proc.name, "chrome.exe");
        assert_eq!(
            proc.path.unwrap().to_str().unwrap(),
            "C:\\Program Files\\Chrome\\chrome.exe"
        );
        assert_eq!(proc.architecture, ProcessArchitecture::X64);
        assert_eq!(proc.parent_pid, Some(1000));
        assert_eq!(proc.thread_count, 25);
        assert_eq!(proc.handle_count, 500);
        assert_eq!(proc.working_set_size, 104857600);
        assert_eq!(proc.page_file_usage, 157286400);
        assert_eq!(proc.session_id, 1);
    }

    #[test]
    fn test_process_architecture() {
        assert_eq!(ProcessArchitecture::X86.pointer_size(), 4);
        assert_eq!(ProcessArchitecture::X64.pointer_size(), 8);
        assert_eq!(ProcessArchitecture::Arm.pointer_size(), 4);
        assert_eq!(ProcessArchitecture::Arm64.pointer_size(), 8);
        assert_eq!(
            ProcessArchitecture::Unknown.pointer_size(),
            std::mem::size_of::<usize>()
        );

        assert!(!ProcessArchitecture::X86.is_64bit());
        assert!(ProcessArchitecture::X64.is_64bit());
        assert!(!ProcessArchitecture::Arm.is_64bit());
        assert!(ProcessArchitecture::Arm64.is_64bit());
        assert!(!ProcessArchitecture::Unknown.is_64bit());
    }

    #[test]
    fn test_module_info_creation() {
        let module = ModuleInfo::new(
            "kernel32.dll".to_string(),
            Address::new(0x7FF60000),
            0x100000,
        );

        assert_eq!(module.name, "kernel32.dll");
        assert_eq!(module.base_address, Address::new(0x7FF60000));
        assert_eq!(module.size, 0x100000);
        assert_eq!(module.path, PathBuf::new());
        assert_eq!(module.entry_point, None);
        assert!(!module.is_system);
    }

    #[test]
    fn test_module_address_range() {
        let module = ModuleInfo::new("test.dll".to_string(), Address::new(0x10000000), 0x1000);

        assert_eq!(module.end_address(), Address::new(0x10001000));

        assert!(module.contains_address(Address::new(0x10000000)));
        assert!(module.contains_address(Address::new(0x10000500)));
        assert!(module.contains_address(Address::new(0x10000FFF)));
        assert!(!module.contains_address(Address::new(0x10001000)));
        assert!(!module.contains_address(Address::new(0x0FFFFFFF)));
    }

    #[test]
    fn test_module_info_full() {
        let mut module =
            ModuleInfo::new("ntdll.dll".to_string(), Address::new(0x7FFE0000), 0x200000);
        module.path = PathBuf::from("C:\\Windows\\System32\\ntdll.dll");
        module.entry_point = Some(Address::new(0x7FFE1000));
        module.is_system = true;

        assert_eq!(module.name, "ntdll.dll");
        assert_eq!(
            module.path.to_str().unwrap(),
            "C:\\Windows\\System32\\ntdll.dll"
        );
        assert_eq!(module.base_address, Address::new(0x7FFE0000));
        assert_eq!(module.size, 0x200000);
        assert_eq!(module.entry_point, Some(Address::new(0x7FFE1000)));
        assert!(module.is_system);
        assert_eq!(module.end_address(), Address::new(0x801E0000));
    }

    #[test]
    fn test_serialization() {
        let proc = ProcessInfo::new(999, "test.exe".to_string());
        let json = serde_json::to_string(&proc).unwrap();
        let deserialized: ProcessInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(proc.pid, deserialized.pid);
        assert_eq!(proc.name, deserialized.name);

        let arch = ProcessArchitecture::X64;
        let json = serde_json::to_string(&arch).unwrap();
        assert_eq!(json, "\"x64\"");
        let deserialized: ProcessArchitecture = serde_json::from_str(&json).unwrap();
        assert_eq!(arch, deserialized);

        let module = ModuleInfo::new("test.dll".to_string(), Address::new(0x1000), 0x2000);
        let json = serde_json::to_string(&module).unwrap();
        let deserialized: ModuleInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(module.name, deserialized.name);
        assert_eq!(module.base_address, deserialized.base_address);
        assert_eq!(module.size, deserialized.size);
    }

    #[test]
    fn test_clone_and_debug() {
        let proc = ProcessInfo::new(111, "app.exe".to_string());
        let cloned = proc.clone();
        assert_eq!(proc.pid, cloned.pid);
        assert_eq!(proc.name, cloned.name);

        let debug_str = format!("{:?}", proc);
        assert!(debug_str.contains("ProcessInfo"));
        assert!(debug_str.contains("111"));
        assert!(debug_str.contains("app.exe"));

        let arch = ProcessArchitecture::Arm64;
        let cloned = arch;
        assert_eq!(arch, cloned);

        let module = ModuleInfo::new("mod.dll".to_string(), Address::new(0x5000), 0x3000);
        let cloned = module.clone();
        assert_eq!(module.name, cloned.name);
        assert_eq!(module.base_address, cloned.base_address);
    }
}
