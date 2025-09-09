//! Process information types

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use super::{Address, ProcessId, ThreadId};

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
    ARM,
    ARM64,
    Unknown,
}

impl ProcessArchitecture {
    /// Returns the pointer size for this architecture
    pub fn pointer_size(&self) -> usize {
        match self {
            ProcessArchitecture::X86 | ProcessArchitecture::ARM => 4,
            ProcessArchitecture::X64 | ProcessArchitecture::ARM64 => 8,
            ProcessArchitecture::Unknown => std::mem::size_of::<usize>(),
        }
    }

    /// Checks if this is a 64-bit architecture
    pub fn is_64bit(&self) -> bool {
        matches!(self, ProcessArchitecture::X64 | ProcessArchitecture::ARM64)
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
        self.base_address.offset(self.size as isize)
    }

    /// Checks if an address is within this module
    pub fn contains_address(&self, address: Address) -> bool {
        address >= self.base_address && address < self.end_address()
    }
}