//! Process enumeration using Windows ToolHelp32 API

use crate::core::types::{MemoryError, MemoryResult};
use crate::process::info::{ProcessArchitecture, ProcessInfo};
use crate::windows::bindings::ntdll;
use crate::windows::utils::string_conv::wide_to_string;
use std::mem;
use std::path::PathBuf;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::HANDLE;

/// Process enumerator using ToolHelp32 API
pub struct ProcessEnumerator {
    snapshot: HANDLE,
    first_called: bool,
}

impl ProcessEnumerator {
    /// Create a new process enumerator
    pub fn new() -> MemoryResult<Self> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot.is_null() || snapshot == winapi::um::handleapi::INVALID_HANDLE_VALUE {
                return Err(MemoryError::WindowsApi(
                    "Failed to create process snapshot".to_string(),
                ));
            }
            Ok(ProcessEnumerator {
                snapshot,
                first_called: false,
            })
        }
    }

    /// Get the next process in the enumeration
    fn next_process(&mut self) -> Option<ProcessInfo> {
        unsafe {
            let mut entry: PROCESSENTRY32 = mem::zeroed();
            entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

            let success = if !self.first_called {
                self.first_called = true;
                Process32First(self.snapshot, &mut entry)
            } else {
                Process32Next(self.snapshot, &mut entry)
            };

            if success == FALSE {
                return None;
            }

            // Convert the process name from fixed-size array
            let name = {
                let name_bytes = &entry.szExeFile;
                let null_pos = name_bytes
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(name_bytes.len());
                // Convert from i8 to u8 for String conversion
                let name_u8: Vec<u8> = name_bytes[..null_pos].iter().map(|&c| c as u8).collect();
                String::from_utf8_lossy(&name_u8).into_owned()
            };

            // Check if process is WoW64 (32-bit on 64-bit Windows)
            let is_wow64 = if let Ok(handle) = crate::windows::bindings::kernel32::open_process(
                entry.th32ProcessID,
                0x0400, // PROCESS_QUERY_INFORMATION
            ) {
                let result = ntdll::is_wow64_process(handle);
                let _ = CloseHandle(handle);
                result.unwrap_or(false)
            } else {
                false
            };

            // Determine architecture
            let architecture = if is_wow64 {
                ProcessArchitecture::X86
            } else {
                // On 64-bit Windows, native processes are x64
                // On 32-bit Windows, all processes are x86
                #[cfg(target_pointer_width = "64")]
                {
                    ProcessArchitecture::X64
                }
                #[cfg(target_pointer_width = "32")]
                {
                    ProcessArchitecture::X86
                }
            };

            Some(ProcessInfo::with_details(
                entry.th32ProcessID,
                name,
                None, // Path would require OpenProcess + GetModuleFileNameEx
                Some(entry.th32ParentProcessID),
                architecture,
                entry.cntThreads,
                is_wow64,
            ))
        }
    }
}

impl Drop for ProcessEnumerator {
    fn drop(&mut self) {
        if !self.snapshot.is_null() && self.snapshot != winapi::um::handleapi::INVALID_HANDLE_VALUE
        {
            unsafe {
                let _ = CloseHandle(self.snapshot);
            }
        }
    }
}

impl Iterator for ProcessEnumerator {
    type Item = ProcessInfo;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_process()
    }
}

/// Enumerate all running processes
pub fn enumerate_processes() -> MemoryResult<Vec<ProcessInfo>> {
    let mut processes = Vec::new();
    let mut enumerator = ProcessEnumerator::new()?;

    while let Some(process) = enumerator.next_process() {
        processes.push(process);
    }

    Ok(processes)
}

/// Find processes by name (case-insensitive)
pub fn find_processes_by_name(name: &str) -> MemoryResult<Vec<ProcessInfo>> {
    let processes = enumerate_processes()?;
    Ok(processes
        .into_iter()
        .filter(|p| p.name_matches(name))
        .collect())
}

/// Find a single process by name
pub fn find_process_by_name(name: &str) -> MemoryResult<Option<ProcessInfo>> {
    Ok(find_processes_by_name(name)?.into_iter().next())
}

/// Get process by PID
pub fn get_process_by_pid(pid: u32) -> MemoryResult<Option<ProcessInfo>> {
    let processes = enumerate_processes()?;
    Ok(processes.into_iter().find(|p| p.pid == pid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_enumerator_new() {
        let enumerator = ProcessEnumerator::new();
        assert!(enumerator.is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enumerate_processes() {
        let result = enumerate_processes();
        assert!(result.is_ok());
        let processes = result.unwrap();

        // Should have at least System and System Idle Process
        assert!(processes.len() >= 2);

        // Check for System process (PID 4)
        let system_process = processes.iter().find(|p| p.pid == 4);
        assert!(system_process.is_some());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_enumerator_iterator() {
        let enumerator = ProcessEnumerator::new();
        assert!(enumerator.is_ok());

        let mut enumerator = enumerator.unwrap();
        let mut count = 0;

        for _ in enumerator.by_ref().take(5) {
            count += 1;
        }

        assert!(count > 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_processes_by_name() {
        // System process should always exist
        let result = find_processes_by_name("System");
        assert!(result.is_ok());
        let processes = result.unwrap();
        assert!(!processes.is_empty());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_process_by_name() {
        let result = find_process_by_name("System");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Non-existent process
        let result = find_process_by_name("NonExistentProcess123456");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_get_process_by_pid() {
        // PID 4 is System process
        let result = get_process_by_pid(4);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Non-existent PID
        let result = get_process_by_pid(999999);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_enumerator_drop() {
        // Test that drop doesn't crash
        {
            let _enumerator = ProcessEnumerator::new().unwrap();
        }
        // Should not crash when dropped
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_current_process_in_enumeration() {
        use std::process;
        let current_pid = process::id();

        let processes = enumerate_processes().unwrap();
        let current_process = processes.iter().find(|p| p.pid == current_pid);
        assert!(current_process.is_some());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_process_info_details() {
        let processes = enumerate_processes().unwrap();

        // Find a known process with details
        if let Some(system_process) = processes.iter().find(|p| p.pid == 4) {
            assert!(system_process.is_system_process());
            assert!(system_process.thread_count > 0);
            assert!(system_process.parent_pid.is_some());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_case_insensitive_search() {
        // These should all find the same process
        let lower = find_processes_by_name("system");
        let upper = find_processes_by_name("SYSTEM");
        let mixed = find_processes_by_name("SyStEm");

        assert!(lower.is_ok());
        assert!(upper.is_ok());
        assert!(mixed.is_ok());

        // Should find at least one system process
        assert!(!lower.unwrap().is_empty());
        assert!(!upper.unwrap().is_empty());
        assert!(!mixed.unwrap().is_empty());
    }
}
