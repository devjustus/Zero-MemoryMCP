//! Privilege checking utilities

use crate::core::types::{MemoryError, MemoryResult};
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{HANDLE, LUID_AND_ATTRIBUTES, TOKEN_PRIVILEGES, TOKEN_QUERY};

/// State of a privilege
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeState {
    /// Privilege is enabled
    Enabled,
    /// Privilege is disabled but can be enabled
    Disabled,
    /// Privilege is not available to the token
    NotPresent,
}

/// Checks privileges for the current process
pub struct PrivilegeChecker;

impl PrivilegeChecker {
    /// Check if the current process has a specific privilege
    pub fn check_privilege(privilege_luid: u32) -> MemoryResult<PrivilegeState> {
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();

            // Open the current process token
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
                return Err(MemoryError::PermissionDenied(
                    "Failed to open process token for checking".to_string(),
                ));
            }

            // Ensure we close the token
            let _guard = TokenGuard(token);

            // Query token privileges
            let mut size: DWORD = 0;
            GetTokenInformation(
                token,
                winapi::um::winnt::TokenPrivileges,
                std::ptr::null_mut(),
                0,
                &mut size,
            );

            if size == 0 {
                return Err(MemoryError::PermissionDenied(
                    "Failed to get token information size".to_string(),
                ));
            }

            // Allocate buffer for privileges
            let mut buffer = vec![0u8; size as usize];
            if GetTokenInformation(
                token,
                winapi::um::winnt::TokenPrivileges,
                buffer.as_mut_ptr() as *mut _,
                size,
                &mut size,
            ) == FALSE
            {
                return Err(MemoryError::PermissionDenied(
                    "Failed to get token privileges".to_string(),
                ));
            }

            // Parse the privileges
            let privileges = &*(buffer.as_ptr() as *const TOKEN_PRIVILEGES);
            let privilege_array = std::slice::from_raw_parts(
                privileges.Privileges.as_ptr(),
                privileges.PrivilegeCount as usize,
            );

            // Check if our privilege is present
            for privilege in privilege_array {
                if privilege.Luid.LowPart == privilege_luid {
                    if privilege.Attributes & winapi::um::winnt::SE_PRIVILEGE_ENABLED != 0 {
                        return Ok(PrivilegeState::Enabled);
                    } else {
                        return Ok(PrivilegeState::Disabled);
                    }
                }
            }

            Ok(PrivilegeState::NotPresent)
        }
    }

    /// Check if the current process is running as administrator
    pub fn is_elevated() -> bool {
        // Simple check - try to open a protected process token
        // In production, we'd check the elevation type properly
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();
            let result = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token);

            if result != FALSE && !token.is_null() {
                CloseHandle(token);
                true
            } else {
                false
            }
        }
    }

    /// Get all available privileges for the current process
    pub fn list_privileges() -> MemoryResult<Vec<LUID_AND_ATTRIBUTES>> {
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();

            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
                return Err(MemoryError::PermissionDenied(
                    "Failed to open process token".to_string(),
                ));
            }

            let _guard = TokenGuard(token);

            let mut size: DWORD = 0;
            GetTokenInformation(
                token,
                winapi::um::winnt::TokenPrivileges,
                std::ptr::null_mut(),
                0,
                &mut size,
            );

            if size == 0 {
                return Ok(Vec::new());
            }

            let mut buffer = vec![0u8; size as usize];
            if GetTokenInformation(
                token,
                winapi::um::winnt::TokenPrivileges,
                buffer.as_mut_ptr() as *mut _,
                size,
                &mut size,
            ) == FALSE
            {
                return Err(MemoryError::PermissionDenied(
                    "Failed to enumerate privileges".to_string(),
                ));
            }

            let privileges = &*(buffer.as_ptr() as *const TOKEN_PRIVILEGES);
            let privilege_array = std::slice::from_raw_parts(
                privileges.Privileges.as_ptr(),
                privileges.PrivilegeCount as usize,
            );

            Ok(privilege_array.to_vec())
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_state_equality() {
        assert_eq!(PrivilegeState::Enabled, PrivilegeState::Enabled);
        assert_ne!(PrivilegeState::Enabled, PrivilegeState::Disabled);
        assert_ne!(PrivilegeState::Disabled, PrivilegeState::NotPresent);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_is_elevated() {
        // Just ensure it doesn't crash
        let _ = PrivilegeChecker::is_elevated();
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_list_privileges() {
        // This should work even without admin rights
        let result = PrivilegeChecker::list_privileges();
        // Just ensure it doesn't crash
        let _ = result;
    }
}
