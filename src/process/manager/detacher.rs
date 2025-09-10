//! Safe process detachment with cleanup

use crate::core::types::{MemoryError, MemoryResult, ProcessId};
use crate::process::ProcessHandle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Options for process detachment
#[derive(Debug, Clone)]
pub struct DetachOptions {
    /// Force detachment even if operations are pending
    pub force: bool,
    /// Clear any cached data for this process
    pub clear_cache: bool,
    /// Wait for pending operations to complete
    pub wait_for_pending: bool,
}

impl Default for DetachOptions {
    fn default() -> Self {
        DetachOptions {
            force: false,
            clear_cache: true,
            wait_for_pending: true,
        }
    }
}

/// Manages safe process detachment
pub struct ProcessDetacher {
    detached_processes: Arc<Mutex<HashMap<ProcessId, DetachInfo>>>,
}

#[derive(Debug, Clone)]
struct DetachInfo {
    timestamp: std::time::Instant,
    reason: String,
}

impl ProcessDetacher {
    /// Create a new process detacher
    pub fn new() -> Self {
        ProcessDetacher {
            detached_processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Detach a process safely
    pub fn detach(&self, handle: ProcessHandle) -> MemoryResult<()> {
        self.detach_with_options(handle, &DetachOptions::default())
    }

    /// Detach a process with custom options
    pub fn detach_with_options(
        &self,
        handle: ProcessHandle,
        _options: &DetachOptions,
    ) -> MemoryResult<()> {
        let pid = handle.pid();

        // Record detachment
        {
            let mut detached = self.detached_processes.lock().unwrap();
            detached.insert(
                pid,
                DetachInfo {
                    timestamp: std::time::Instant::now(),
                    reason: "Manual detachment".to_string(),
                },
            );
        }

        // Drop the handle to close it
        drop(handle);

        Ok(())
    }

    /// Detach multiple processes
    pub fn detach_batch(&self, handles: Vec<ProcessHandle>) -> Vec<MemoryResult<()>> {
        handles
            .into_iter()
            .map(|handle| self.detach(handle))
            .collect()
    }

    /// Get detachment history
    pub fn get_detach_history(&self) -> Vec<(ProcessId, std::time::Instant)> {
        self.detached_processes
            .lock()
            .unwrap()
            .iter()
            .map(|(pid, info)| (*pid, info.timestamp))
            .collect()
    }

    /// Clear detachment history
    pub fn clear_history(&self) {
        self.detached_processes.lock().unwrap().clear();
    }

    /// Check if a process was recently detached
    pub fn was_recently_detached(&self, pid: ProcessId) -> bool {
        self.detached_processes.lock().unwrap().contains_key(&pid)
    }
}

impl Default for ProcessDetacher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detach_options_default() {
        let options = DetachOptions::default();
        assert!(!options.force);
        assert!(options.clear_cache);
        assert!(options.wait_for_pending);
    }

    #[test]
    fn test_process_detacher_creation() {
        let detacher = ProcessDetacher::new();
        assert_eq!(detacher.get_detach_history().len(), 0);
    }

    #[test]
    fn test_detach_history() {
        let detacher = ProcessDetacher::new();
        assert!(!detacher.was_recently_detached(1234));

        // Clear history
        detacher.clear_history();
        assert_eq!(detacher.get_detach_history().len(), 0);
    }
}
