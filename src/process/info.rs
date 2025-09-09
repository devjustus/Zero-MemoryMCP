//! Process information structures and utilities

use std::fmt;
use std::path::PathBuf;

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

    /// Create ProcessInfo with full details
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

    /// Get the process name without extension
    pub fn base_name(&self) -> &str {
        if let Some(dot_pos) = self.name.rfind('.') {
            &self.name[..dot_pos]
        } else {
            &self.name
        }
    }

    /// Check if process name matches (case-insensitive)
    pub fn name_matches(&self, pattern: &str) -> bool {
        self.name.to_lowercase().contains(&pattern.to_lowercase())
    }
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} ({}, {} threads)",
            self.pid, self.name, self.architecture, self.thread_count
        )?;
        if self.is_wow64 {
            write!(f, " [WoW64]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_architecture_display() {
        assert_eq!(format!("{}", ProcessArchitecture::X86), "x86");
        assert_eq!(format!("{}", ProcessArchitecture::X64), "x64");
        assert_eq!(format!("{}", ProcessArchitecture::Unknown), "unknown");
    }

    #[test]
    fn test_process_architecture_equality() {
        assert_eq!(ProcessArchitecture::X86, ProcessArchitecture::X86);
        assert_ne!(ProcessArchitecture::X86, ProcessArchitecture::X64);
        assert_ne!(ProcessArchitecture::X64, ProcessArchitecture::Unknown);
    }

    #[test]
    fn test_process_info_new() {
        let info = ProcessInfo::new(1234, "test.exe".to_string());
        assert_eq!(info.pid, 1234);
        assert_eq!(info.name, "test.exe");
        assert_eq!(info.path, None);
        assert_eq!(info.parent_pid, None);
        assert_eq!(info.architecture, ProcessArchitecture::Unknown);
        assert_eq!(info.thread_count, 0);
        assert!(!info.is_wow64);
    }

    #[test]
    fn test_process_info_with_details() {
        let info = ProcessInfo::with_details(
            5678,
            "app.exe".to_string(),
            Some(PathBuf::from("C:\\Program Files\\App\\app.exe")),
            Some(1234),
            ProcessArchitecture::X64,
            8,
            false,
        );
        assert_eq!(info.pid, 5678);
        assert_eq!(info.name, "app.exe");
        assert_eq!(
            info.path,
            Some(PathBuf::from("C:\\Program Files\\App\\app.exe"))
        );
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

        let no_ext = ProcessInfo::new(1234, "app".to_string());
        assert_eq!(no_ext.base_name(), "app");

        let multi_dot = ProcessInfo::new(1234, "my.app.exe".to_string());
        assert_eq!(multi_dot.base_name(), "my.app");
    }

    #[test]
    fn test_name_matches() {
        let info = ProcessInfo::new(1234, "Notepad.exe".to_string());
        assert!(info.name_matches("notepad"));
        assert!(info.name_matches("NOTEPAD"));
        assert!(info.name_matches("pad"));
        assert!(!info.name_matches("wordpad"));
    }

    #[test]
    fn test_process_info_display() {
        let info = ProcessInfo::with_details(
            1234,
            "test.exe".to_string(),
            None,
            None,
            ProcessArchitecture::X64,
            4,
            false,
        );
        let display = format!("{}", info);
        assert!(display.contains("[1234]"));
        assert!(display.contains("test.exe"));
        assert!(display.contains("x64"));
        assert!(display.contains("4 threads"));
        assert!(!display.contains("WoW64"));

        let wow64_info = ProcessInfo::with_details(
            5678,
            "app.exe".to_string(),
            None,
            None,
            ProcessArchitecture::X86,
            2,
            true,
        );
        let wow64_display = format!("{}", wow64_info);
        assert!(wow64_display.contains("[WoW64]"));
    }

    #[test]
    fn test_process_info_clone() {
        let info = ProcessInfo::new(1234, "test.exe".to_string());
        let cloned = info.clone();
        assert_eq!(cloned.pid, info.pid);
        assert_eq!(cloned.name, info.name);
    }

    #[test]
    fn test_process_architecture_copy() {
        let arch = ProcessArchitecture::X64;
        let copied = arch;
        assert_eq!(copied, arch);
    }
}
