//! Process information subsystem

use std::fmt;
use std::path::PathBuf;

pub mod modules;

pub use modules::{
    enumerate_modules, find_module_by_name, get_process_main_module, ModuleEnumerator,
};

/// Architecture of a process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessArchitecture {
    /// 32-bit x86 process
    X86,
    /// 64-bit x64 process
    X64,
    /// Unknown architecture
    Unknown,
}

impl fmt::Display for ProcessArchitecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessArchitecture::X86 => write!(f, "x86"),
            ProcessArchitecture::X64 => write!(f, "x64"),
            ProcessArchitecture::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a running process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Full path to the executable
    pub path: Option<PathBuf>,
    /// Parent process ID
    pub parent_pid: Option<u32>,
    /// Process architecture
    pub architecture: ProcessArchitecture,
    /// Number of threads
    pub thread_count: u32,
    /// Whether this is a WoW64 process (32-bit on 64-bit Windows)
    pub is_wow64: bool,
}

impl ProcessInfo {
    /// Create a new ProcessInfo with minimal information
    pub fn new(pid: u32, name: String) -> Self {
        ProcessInfo {
            pid,
            name,
            path: None,
            parent_pid: None,
            architecture: ProcessArchitecture::Unknown,
            thread_count: 0,
            is_wow64: false,
        }
    }

    /// Create with full details
    pub fn with_details(
        pid: u32,
        name: String,
        path: Option<PathBuf>,
        parent_pid: Option<u32>,
        architecture: ProcessArchitecture,
        thread_count: u32,
        is_wow64: bool,
    ) -> Self {
        ProcessInfo {
            pid,
            name,
            path,
            parent_pid,
            architecture,
            thread_count,
            is_wow64,
        }
    }

    /// Check if this is a system process
    pub fn is_system_process(&self) -> bool {
        self.pid == 0 || self.pid == 4
    }

    /// Get the base name (without extension) of the process
    pub fn base_name(&self) -> &str {
        self.name.split('.').next().unwrap_or(&self.name)
    }

    /// Check if the process name matches (case-insensitive)
    pub fn name_matches(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Process [{}] {} ({})",
            self.pid, self.name, self.architecture
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_architecture_display() {
        assert_eq!(ProcessArchitecture::X86.to_string(), "x86");
        assert_eq!(ProcessArchitecture::X64.to_string(), "x64");
        assert_eq!(ProcessArchitecture::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_process_architecture_equality() {
        assert_eq!(ProcessArchitecture::X86, ProcessArchitecture::X86);
        assert_ne!(ProcessArchitecture::X86, ProcessArchitecture::X64);
    }

    #[test]
    fn test_process_architecture_copy() {
        let arch = ProcessArchitecture::X64;
        let copy = arch;
        assert_eq!(arch, copy);
    }

    #[test]
    fn test_process_info_new() {
        let info = ProcessInfo::new(1234, "test.exe".to_string());
        assert_eq!(info.pid, 1234);
        assert_eq!(info.name, "test.exe");
        assert!(info.path.is_none());
        assert!(info.parent_pid.is_none());
        assert_eq!(info.architecture, ProcessArchitecture::Unknown);
        assert_eq!(info.thread_count, 0);
        assert!(!info.is_wow64);
    }

    #[test]
    fn test_process_info_with_details() {
        let info = ProcessInfo::with_details(
            5678,
            "app.exe".to_string(),
            Some(PathBuf::from("C:\\Program Files\\app.exe")),
            Some(1234),
            ProcessArchitecture::X64,
            8,
            false,
        );
        assert_eq!(info.pid, 5678);
        assert_eq!(info.name, "app.exe");
        assert!(info.path.is_some());
        assert_eq!(info.parent_pid, Some(1234));
        assert_eq!(info.architecture, ProcessArchitecture::X64);
        assert_eq!(info.thread_count, 8);
        assert!(!info.is_wow64);
    }

    #[test]
    fn test_is_system_process() {
        let system_idle = ProcessInfo::new(0, "System Idle Process".to_string());
        assert!(system_idle.is_system_process());

        let system = ProcessInfo::new(4, "System".to_string());
        assert!(system.is_system_process());

        let normal = ProcessInfo::new(1234, "notepad.exe".to_string());
        assert!(!normal.is_system_process());
    }

    #[test]
    fn test_base_name() {
        let info = ProcessInfo::new(1234, "notepad.exe".to_string());
        assert_eq!(info.base_name(), "notepad");

        let info_no_ext = ProcessInfo::new(5678, "svchost".to_string());
        assert_eq!(info_no_ext.base_name(), "svchost");
    }

    #[test]
    fn test_name_matches() {
        let info = ProcessInfo::new(1234, "Notepad.exe".to_string());
        assert!(info.name_matches("notepad.exe"));
        assert!(info.name_matches("NOTEPAD.EXE"));
        assert!(info.name_matches("Notepad.exe"));
        assert!(!info.name_matches("calc.exe"));
    }

    #[test]
    fn test_process_info_display() {
        let info = ProcessInfo::new(1234, "test.exe".to_string());
        let display = format!("{}", info);
        assert!(display.contains("1234"));
        assert!(display.contains("test.exe"));
        assert!(display.contains("unknown"));
    }

    #[test]
    fn test_process_info_clone() {
        let info = ProcessInfo::new(1234, "test.exe".to_string());
        let cloned = info.clone();
        assert_eq!(cloned.pid, info.pid);
        assert_eq!(cloned.name, info.name);
    }
}
