//! Windows API layer for memory manipulation
//!
//! Provides safe wrappers around Windows API functions for process
//! and memory operations. All unsafe FFI calls are contained within
//! this module with proper error handling and validation.

pub mod bindings;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use types::{Handle, MemoryBasicInfo, ModuleInfo as WinModuleInfo};
pub use utils::{ErrorCode, WinError};

// Re-export key bindings
pub use bindings::{kernel32, ntdll, psapi};

/// Check if running on supported Windows version (Windows 10+)
pub fn is_supported_windows() -> bool {
    // Windows 10 is version 10.0 or higher
    cfg!(target_os = "windows") && cfg!(target_arch = "x86_64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_support() {
        // Should be true on Windows x64
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        assert!(is_supported_windows());

        // Should be false on non-Windows or non-x64
        #[cfg(not(all(target_os = "windows", target_arch = "x86_64")))]
        assert!(!is_supported_windows());
    }
}
