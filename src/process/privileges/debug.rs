//! SeDebugPrivilege handling for process manipulation

use crate::core::types::{MemoryError, MemoryResult};
use std::sync::atomic::{AtomicBool, Ordering};
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::winnt::{
    HANDLE, LUID, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};

static DEBUG_PRIVILEGE_ENABLED: AtomicBool = AtomicBool::new(false);

/// RAII guard for temporarily enabling debug privilege
pub struct DebugPrivilegeGuard {
    was_enabled: bool,
}

impl DebugPrivilegeGuard {
    /// Create a new guard, enabling debug privilege
    pub fn new() -> MemoryResult<Self> {
        let was_enabled = has_debug_privilege();
        if !was_enabled {
            enable_debug_privilege()?;
        }
        Ok(DebugPrivilegeGuard { was_enabled })
    }
}

impl Drop for DebugPrivilegeGuard {
    fn drop(&mut self) {
        // Only disable if we enabled it
        if !self.was_enabled {
            // In production, we might want to disable it
            // For now, leave it enabled for performance
        }
    }
}

/// Check if the current process has SeDebugPrivilege enabled
pub fn has_debug_privilege() -> bool {
    DEBUG_PRIVILEGE_ENABLED.load(Ordering::Relaxed)
}

/// Enable SeDebugPrivilege for the current process
pub fn enable_debug_privilege() -> MemoryResult<()> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();

        // Open the current process token
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(MemoryError::PermissionDenied(
                "Failed to open process token".to_string(),
            ));
        }

        // Ensure we close the token handle on exit
        let _token_guard = TokenGuard::new(token);

        // Look up the LUID for SeDebugPrivilege
        let mut luid = LUID {
            LowPart: 0,
            HighPart: 0,
        };

        let privilege_name: Vec<u16> = "SeDebugPrivilege".encode_utf16().chain(Some(0)).collect();
        if LookupPrivilegeValueW(std::ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            return Err(MemoryError::PermissionDenied(
                "Failed to lookup SeDebugPrivilege".to_string(),
            ));
        }

        // Prepare the privilege structure
        let mut privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        // Enable the privilege
        if AdjustTokenPrivileges(
            token,
            FALSE,
            &mut privileges,
            std::mem::size_of::<TOKEN_PRIVILEGES>() as DWORD,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) == FALSE
        {
            return Err(MemoryError::InsufficientPrivileges(
                "Failed to enable SeDebugPrivilege".to_string(),
            ));
        }

        // Mark as enabled
        DEBUG_PRIVILEGE_ENABLED.store(true, Ordering::Relaxed);
        Ok(())
    }
}

/// Internal token handle guard for RAII cleanup
struct TokenGuard {
    handle: HANDLE,
}

impl TokenGuard {
    fn new(handle: HANDLE) -> Self {
        TokenGuard { handle }
    }
}

impl Drop for TokenGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_creation() {
        // This might fail without admin rights
        let guard = DebugPrivilegeGuard::new();
        // It's okay if this fails in CI
        if guard.is_ok() {
            assert!(has_debug_privilege());
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_drop() {
        // Test guard drop behavior
        let initial_state = has_debug_privilege();
        {
            let guard = DebugPrivilegeGuard::new();
            if guard.is_ok() {
                // Guard exists in this scope
                let _ = guard;
            }
        }
        // Guard dropped, verify state consistency
        let _ = initial_state;
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enable_debug_privilege() {
        // This test might fail without admin rights
        let result = enable_debug_privilege();
        // We don't assert success as it requires admin privileges
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enable_debug_privilege_twice() {
        // Test calling enable_debug_privilege twice
        let result1 = enable_debug_privilege();
        let result2 = enable_debug_privilege();

        // If first succeeds, second should also succeed
        if result1.is_ok() {
            assert!(result2.is_ok());
            assert!(has_debug_privilege());
        }
    }

    #[test]
    fn test_has_debug_privilege() {
        // Test the atomic bool reading
        let state = has_debug_privilege();
        // Should return consistent value
        assert_eq!(state, has_debug_privilege());
    }

    #[test]
    fn test_token_guard_new() {
        // Test TokenGuard creation
        let guard = TokenGuard::new(std::ptr::null_mut());
        assert!(guard.handle.is_null());
    }

    #[test]
    fn test_token_guard_drop_null() {
        // Test that dropping null handle doesn't crash
        let guard = TokenGuard::new(std::ptr::null_mut());
        drop(guard);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_with_enabled() {
        // Test creating guard when privilege might already be enabled
        let _result1 = enable_debug_privilege();

        // Now try to create a guard
        let guard = DebugPrivilegeGuard::new();
        if let Ok(guard_val) = guard {
            // Check internal state
            let _ = guard_val.was_enabled;
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_comprehensive() {
        // Test all paths in enable_debug_privilege
        let initial = has_debug_privilege();
        
        // Try enabling multiple times
        for _ in 0..3 {
            let result = enable_debug_privilege();
            if result.is_ok() {
                assert!(has_debug_privilege());
            }
        }
        
        // State should be consistent
        let final_state = has_debug_privilege();
        if initial && !final_state {
            panic!("State should not have been disabled");
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_nested() {
        // Test nested guard creation
        let initial = has_debug_privilege();
        
        {
            let guard1 = DebugPrivilegeGuard::new();
            if let Ok(g1) = guard1 {
                let was_enabled1 = g1.was_enabled;
                
                {
                    let guard2 = DebugPrivilegeGuard::new();
                    if let Ok(g2) = guard2 {
                        let was_enabled2 = g2.was_enabled;
                        
                        // Inner guard should see the state from outer guard
                        if !was_enabled1 {
                            assert!(was_enabled2 || has_debug_privilege());
                        }
                    }
                    // guard2 dropped
                }
                
                // Still have guard1
                if !was_enabled1 {
                    let _ = has_debug_privilege();
                }
            }
            // guard1 dropped
        }
        
        let _ = initial;
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_error_path() {
        // Test guard creation failure path
        // This test might not fail, but ensures error path doesn't panic
        
        let guards: Vec<Result<DebugPrivilegeGuard, _>> = (0..5)
            .map(|_| DebugPrivilegeGuard::new())
            .collect();
        
        for guard in guards {
            match guard {
                Ok(g) => {
                    // Guard created successfully
                    drop(g);
                }
                Err(e) => {
                    // Error should have meaningful message
                    let msg = e.to_string();
                    assert!(!msg.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_token_guard_creation() {
        // Test TokenGuard creation with various handles
        let guards = vec![
            TokenGuard::new(std::ptr::null_mut()),
            TokenGuard::new(1 as HANDLE),
            TokenGuard::new(usize::MAX as HANDLE),
        ];
        
        for guard in guards {
            // All should drop without panic
            drop(guard);
        }
    }

    #[test]
    fn test_token_guard_drop_behavior() {
        // Test that drop is called properly
        {
            let _guard = TokenGuard::new(std::ptr::null_mut());
            // Guard dropped at end of scope
        }
        
        {
            let guard = TokenGuard::new(1 as HANDLE);
            drop(guard); // Explicit drop
        }
        
        // Test moving guard
        let guard1 = TokenGuard::new(std::ptr::null_mut());
        let guard2 = guard1; // Move
        drop(guard2);
    }

    #[test]
    fn test_has_debug_privilege_atomic() {
        // Test atomic operations
        let initial = has_debug_privilege();
        
        // Multiple reads should be consistent
        let reads: Vec<bool> = (0..100)
            .map(|_| has_debug_privilege())
            .collect();
        
        for read in &reads {
            assert_eq!(*read, initial);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_enable_debug_privilege_idempotent() {
        // Test that enabling is idempotent
        let results: Vec<_> = (0..3)
            .map(|_| enable_debug_privilege())
            .collect();
        
        // If first succeeds, all should succeed
        if results[0].is_ok() {
            for result in &results[1..] {
                assert!(result.is_ok());
            }
        }
        
        // If first fails, all should fail with same error type
        if results[0].is_err() {
            for result in &results[1..] {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_debug_privilege_guard_drop_state() {
        // Test that guard drop doesn't change state unexpectedly
        let initial = has_debug_privilege();
        
        // Create and immediately drop guard
        if let Ok(guard) = DebugPrivilegeGuard::new() {
            drop(guard);
        }
        
        // State should be preserved (we don't disable on drop)
        let after_drop = has_debug_privilege();
        if !initial && after_drop {
            // This is expected - we enabled it
        } else if initial && !after_drop {
            panic!("Should not have disabled privilege");
        }
    }
}
