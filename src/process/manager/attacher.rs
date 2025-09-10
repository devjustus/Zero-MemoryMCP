//! Safe process attachment with automatic cleanup

use crate::core::types::{MemoryError, MemoryResult, ProcessId};
use crate::process::ProcessHandle;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Options for process attachment
#[derive(Debug, Clone)]
pub struct AttachOptions {
    /// Request all access rights
    pub all_access: bool,
    /// Request only read access
    pub read_only: bool,
    /// Enable debug privileges before attachment
    pub enable_debug_privilege: bool,
    /// Timeout for attachment in milliseconds
    pub timeout_ms: Option<u32>,
}

impl Default for AttachOptions {
    fn default() -> Self {
        AttachOptions {
            all_access: false,
            read_only: true,
            enable_debug_privilege: true,
            timeout_ms: Some(5000),
        }
    }
}

/// RAII guard for automatic process detachment
pub struct AttachmentGuard {
    handle: Option<ProcessHandle>,
    pid: ProcessId,
    auto_detach: bool,
}

impl AttachmentGuard {
    /// Create a new attachment guard
    fn new(handle: ProcessHandle, pid: ProcessId, auto_detach: bool) -> Self {
        AttachmentGuard {
            handle: Some(handle),
            pid,
            auto_detach,
        }
    }

    /// Get the process handle
    pub fn handle(&self) -> Option<&ProcessHandle> {
        self.handle.as_ref()
    }

    /// Get the process ID
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    /// Manually detach the process
    pub fn detach(mut self) -> MemoryResult<()> {
        if let Some(handle) = self.handle.take() {
            drop(handle);
        }
        Ok(())
    }

    /// Take ownership of the handle, preventing automatic detachment
    pub fn into_handle(mut self) -> Option<ProcessHandle> {
        self.auto_detach = false;
        self.handle.take()
    }
}

impl Drop for AttachmentGuard {
    fn drop(&mut self) {
        if self.auto_detach {
            if let Some(handle) = self.handle.take() {
                drop(handle);
            }
        }
    }
}

/// Manages process attachments with safety guarantees
pub struct ProcessAttacher {
    attached_pids: Arc<Mutex<HashSet<ProcessId>>>,
    default_options: AttachOptions,
}

impl ProcessAttacher {
    /// Create a new process attacher
    pub fn new() -> Self {
        ProcessAttacher {
            attached_pids: Arc::new(Mutex::new(HashSet::new())),
            default_options: AttachOptions::default(),
        }
    }

    /// Create with custom default options
    pub fn with_options(options: AttachOptions) -> Self {
        ProcessAttacher {
            attached_pids: Arc::new(Mutex::new(HashSet::new())),
            default_options: options,
        }
    }

    /// Attach to a process by ID
    pub fn attach(&self, pid: ProcessId) -> MemoryResult<AttachmentGuard> {
        self.attach_with_options(pid, &self.default_options)
    }

    /// Attach to a process with custom options
    pub fn attach_with_options(
        &self,
        pid: ProcessId,
        options: &AttachOptions,
    ) -> MemoryResult<AttachmentGuard> {
        // Check if already attached
        {
            let attached = self.attached_pids.lock().unwrap();
            if attached.contains(&pid) {
                return Err(MemoryError::ProcessAlreadyAttached(pid));
            }
        }

        // Open the process with appropriate access
        let handle = if options.all_access {
            ProcessHandle::open_all_access(pid)?
        } else if options.read_only {
            ProcessHandle::open_for_read(pid)?
        } else {
            ProcessHandle::open_for_read_write(pid)?
        };

        // Verify the handle is valid
        if !handle.is_valid() {
            return Err(MemoryError::InvalidHandle(format!(
                "Failed to attach to process {}",
                pid
            )));
        }

        // Store the PID as attached
        {
            let mut attached = self.attached_pids.lock().unwrap();
            attached.insert(pid);
        }

        // Create the attachment guard with the handle
        Ok(AttachmentGuard::new(handle, pid, true))
    }

    /// Get the number of attached processes
    pub fn attached_count(&self) -> usize {
        self.attached_pids.lock().unwrap().len()
    }

    /// Check if a process is attached
    pub fn is_attached(&self, pid: ProcessId) -> bool {
        self.attached_pids.lock().unwrap().contains(&pid)
    }

    /// Detach all processes
    pub fn detach_all(&self) -> MemoryResult<()> {
        let mut attached = self.attached_pids.lock().unwrap();
        attached.clear();
        Ok(())
    }

    /// Remove a PID from the attached set (called by AttachmentGuard on drop)
    fn remove_attached(&self, pid: ProcessId) {
        let mut attached = self.attached_pids.lock().unwrap();
        attached.remove(&pid);
    }
}

impl Default for ProcessAttacher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_options_default() {
        let options = AttachOptions::default();
        assert!(!options.all_access);
        assert!(options.read_only);
        assert!(options.enable_debug_privilege);
        assert_eq!(options.timeout_ms, Some(5000));
    }

    #[test]
    fn test_process_attacher_creation() {
        let attacher = ProcessAttacher::new();
        assert_eq!(attacher.attached_count(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_attach_invalid_process() {
        let attacher = ProcessAttacher::new();
        let result = attacher.attach(0);
        assert!(result.is_err());
    }
}
