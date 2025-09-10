//! Process management functionality for Windows
//!
//! This module provides safe abstractions for process enumeration,
//! process information retrieval, and process handle management.

pub mod enumerator;
pub mod handle;
pub mod info;
pub mod manager;
pub mod privileges;

pub use enumerator::{enumerate_processes, ProcessEnumerator};
pub use handle::ProcessHandle;
pub use info::{ProcessArchitecture, ProcessInfo};
pub use manager::{
    AttachOptions, AttachmentGuard, DetachOptions, ProcessAttacher, ProcessDetacher,
};
pub use privileges::{
    enable_debug_privilege, has_debug_privilege, require_privilege, DebugPrivilegeGuard,
    ElevationOptions, PrivilegeChecker, PrivilegeElevator, PrivilegeState,
};

use crate::core::types::MemoryResult;

/// Check if we have debug privileges
pub fn has_debug_privileges() -> bool {
    has_debug_privilege()
}

/// Request debug privileges for the current process
pub fn enable_debug_privileges() -> MemoryResult<()> {
    enable_debug_privilege()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_debug_privileges() {
        // Check that the function exists and doesn't panic
        let _ = has_debug_privileges();
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enable_debug_privileges() {
        // This might fail without admin rights, but should not panic
        let result = enable_debug_privileges();
        // Just ensure it doesn't panic - the result depends on privileges
        let _ = result;
    }
}
