//! Privilege elevation and management

use crate::core::types::{MemoryError, MemoryResult};
use std::collections::HashMap;
use std::sync::Mutex;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::winnt::{
    HANDLE, LUID, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};

lazy_static::lazy_static! {
    static ref ELEVATED_PRIVILEGES: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());
}

/// Options for privilege elevation
#[derive(Debug, Clone)]
pub struct ElevationOptions {
    /// Attempt to enable the privilege if not already enabled
    pub auto_enable: bool,
    /// Fail if the privilege cannot be enabled
    pub require_success: bool,
    /// Cache the elevation status
    pub cache_result: bool,
}

impl Default for ElevationOptions {
    fn default() -> Self {
        ElevationOptions {
            auto_enable: true,
            require_success: false,
            cache_result: true,
        }
    }
}

/// Manages privilege elevation for the current process
pub struct PrivilegeElevator {
    options: ElevationOptions,
}

impl PrivilegeElevator {
    /// Create a new privilege elevator with default options
    pub fn new() -> Self {
        PrivilegeElevator {
            options: ElevationOptions::default(),
        }
    }

    /// Create with custom options
    pub fn with_options(options: ElevationOptions) -> Self {
        PrivilegeElevator { options }
    }

    /// Elevate a specific privilege by name
    pub fn elevate(&self, privilege_name: &str) -> MemoryResult<bool> {
        // Check cache first
        if self.options.cache_result {
            let cache = ELEVATED_PRIVILEGES.lock().unwrap();
            if let Some(&elevated) = cache.get(privilege_name) {
                return Ok(elevated);
            }
        }

        // Convert privilege name to wide string
        let wide_name: Vec<u16> = privilege_name.encode_utf16().chain(Some(0)).collect();

        let result = unsafe { self.elevate_privilege_internal(&wide_name) };

        // Cache the result
        if self.options.cache_result {
            let mut cache = ELEVATED_PRIVILEGES.lock().unwrap();
            cache.insert(privilege_name.to_string(), result.is_ok());
        }

        match result {
            Ok(()) => Ok(true),
            Err(_e) if !self.options.require_success => {
                // Log but don't fail if not required
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }

    /// Internal elevation implementation
    unsafe fn elevate_privilege_internal(&self, privilege_name: &[u16]) -> MemoryResult<()> {
        let mut token: HANDLE = std::ptr::null_mut();

        // Open the current process token
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(MemoryError::PermissionDenied(
                "Failed to open process token for elevation".to_string(),
            ));
        }

        // Ensure we close the token
        let _guard = TokenGuard(token);

        // Look up the privilege LUID
        let mut luid = LUID {
            LowPart: 0,
            HighPart: 0,
        };

        if LookupPrivilegeValueW(std::ptr::null(), privilege_name.as_ptr(), &mut luid) == FALSE {
            return Err(MemoryError::InsufficientPrivileges(
                "Failed to lookup privilege value".to_string(),
            ));
        }

        // Prepare privilege structure
        let mut privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        // Adjust token privileges
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
                "Failed to adjust token privileges".to_string(),
            ));
        }

        Ok(())
    }

    /// Clear the privilege cache
    pub fn clear_cache() {
        let mut cache = ELEVATED_PRIVILEGES.lock().unwrap();
        cache.clear();
    }
}

impl Default for PrivilegeElevator {
    fn default() -> Self {
        Self::new()
    }
}

/// Token handle guard for RAII cleanup
struct TokenGuard(HANDLE);

impl Drop for TokenGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Require a specific privilege to be elevated
pub fn require_privilege(privilege_name: &str) -> MemoryResult<()> {
    let elevator = PrivilegeElevator::with_options(ElevationOptions {
        auto_enable: true,
        require_success: true,
        cache_result: true,
    });

    elevator.elevate(privilege_name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevation_options_default() {
        let options = ElevationOptions::default();
        assert!(options.auto_enable);
        assert!(!options.require_success);
        assert!(options.cache_result);
    }

    #[test]
    fn test_elevation_options_custom() {
        let options = ElevationOptions {
            auto_enable: false,
            require_success: true,
            cache_result: false,
        };
        assert!(!options.auto_enable);
        assert!(options.require_success);
        assert!(!options.cache_result);
    }

    #[test]
    fn test_elevation_options_clone() {
        let options = ElevationOptions::default();
        let cloned = options.clone();
        assert_eq!(options.auto_enable, cloned.auto_enable);
        assert_eq!(options.require_success, cloned.require_success);
        assert_eq!(options.cache_result, cloned.cache_result);
    }

    #[test]
    fn test_elevation_options_debug() {
        let options = ElevationOptions::default();
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("ElevationOptions"));
    }

    #[test]
    fn test_privilege_elevator_creation() {
        let _elevator = PrivilegeElevator::new();
        // Clear cache for clean test
        PrivilegeElevator::clear_cache();
    }

    #[test]
    fn test_privilege_elevator_default() {
        let elevator1 = PrivilegeElevator::new();
        let elevator2 = PrivilegeElevator::default();
        // Both should have default options
        assert_eq!(elevator1.options.auto_enable, elevator2.options.auto_enable);
    }

    #[test]
    fn test_privilege_elevator_with_options() {
        let options = ElevationOptions {
            auto_enable: false,
            require_success: true,
            cache_result: false,
        };
        let elevator = PrivilegeElevator::with_options(options.clone());
        assert_eq!(elevator.options.auto_enable, options.auto_enable);
        assert_eq!(elevator.options.require_success, options.require_success);
        assert_eq!(elevator.options.cache_result, options.cache_result);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_elevate_nonexistent_privilege() {
        let elevator = PrivilegeElevator::new();
        let result = elevator.elevate("SeNonexistentPrivilege");
        // Should fail gracefully
        assert!(!result.unwrap_or(false));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_elevate_with_require_success() {
        let options = ElevationOptions {
            auto_enable: true,
            require_success: true,
            cache_result: false,
        };
        let elevator = PrivilegeElevator::with_options(options);
        let result = elevator.elevate("SeNonexistentPrivilege");
        // Should return error when require_success is true
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_elevate_with_caching() {
        PrivilegeElevator::clear_cache();

        let elevator = PrivilegeElevator::new();

        // First call
        let result1 = elevator.elevate("SeDebugPrivilege");

        // Second call should use cache
        let result2 = elevator.elevate("SeDebugPrivilege");

        // Results should be consistent
        if result1.is_ok() && result2.is_ok() {
            assert_eq!(result1.unwrap(), result2.unwrap());
        }

        PrivilegeElevator::clear_cache();
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_require_privilege_function() {
        // Test the standalone function
        let result = require_privilege("SeNonexistentPrivilege");
        // Should fail since privilege doesn't exist and require_success is true
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_cache() {
        // Ensure clear_cache doesn't panic
        PrivilegeElevator::clear_cache();
        PrivilegeElevator::clear_cache(); // Call twice to ensure idempotent
    }

    #[test]
    fn test_token_guard_drop() {
        // Test that TokenGuard properly handles null
        let guard = TokenGuard(std::ptr::null_mut());
        drop(guard); // Should not crash
    }
}
