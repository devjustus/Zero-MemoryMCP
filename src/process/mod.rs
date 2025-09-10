//! Process management functionality for Windows
//!
//! This module provides safe abstractions for process enumeration,
//! process information retrieval, and process handle management.

pub mod enumerator;
pub mod handle;
pub mod info;
pub mod manager;

pub use enumerator::{enumerate_processes, ProcessEnumerator};
pub use handle::ProcessHandle;
pub use info::{ProcessArchitecture, ProcessInfo};
pub use manager::{
    AttachOptions, AttachmentGuard, DetachOptions, ProcessAttacher, ProcessDetacher,
};

use crate::core::types::{MemoryError, MemoryResult};

/// Check if we have debug privileges
pub fn has_debug_privileges() -> bool {
    // This would require checking token privileges
    // For now, assume we need to request them
    false
}

/// Request debug privileges for the current process
pub fn enable_debug_privileges() -> MemoryResult<()> {
    // This will be implemented when we add privilege management
    // For now, return an error indicating it's not implemented
    Err(MemoryError::PermissionDenied(
        "Debug privilege management not yet implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_debug_privileges() {
        // Should return false by default
        assert!(!has_debug_privileges());
    }

    #[test]
    fn test_enable_debug_privileges() {
        // Should return not implemented error for now
        let result = enable_debug_privileges();
        assert!(result.is_err());
        match result.unwrap_err() {
            MemoryError::PermissionDenied(msg) => {
                assert!(msg.contains("not yet implemented"));
            }
            _ => panic!("Expected PermissionDenied error"),
        }
    }
}
