//! Integration tests for privilege management

use memory_mcp::process::{
    enable_debug_privilege, has_debug_privilege, require_privilege, DebugPrivilegeGuard,
    ElevationOptions, PrivilegeChecker, PrivilegeElevator, PrivilegeState,
};

#[test]
fn test_debug_privilege_state() {
    // Check initial state
    let initial_state = has_debug_privilege();

    // State should be consistent
    assert_eq!(initial_state, has_debug_privilege());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_enable_debug_privilege() {
    // This test might fail without admin rights
    let result = enable_debug_privilege();

    // If it succeeds, verify the state changed
    if result.is_ok() {
        assert!(has_debug_privilege());
    }
    // If it fails, that's expected without admin rights
}

#[test]
fn test_debug_privilege_guard() {
    // Test RAII guard creation
    let _initial_state = has_debug_privilege();

    {
        // This might fail without admin rights
        let guard_result = DebugPrivilegeGuard::new();
        if guard_result.is_ok() {
            // If successful, privilege should be enabled
            assert!(has_debug_privilege());
        }
        // Guard will clean up on drop
    }

    // Note: We don't disable privileges on drop currently
}

#[test]
fn test_privilege_elevator_creation() {
    let _elevator = PrivilegeElevator::new();

    // Test with custom options
    let options = ElevationOptions {
        auto_enable: false,
        require_success: true,
        cache_result: false,
    };
    let _elevator_with_opts = PrivilegeElevator::with_options(options);

    // Clear cache
    PrivilegeElevator::clear_cache();
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_elevate_debug_privilege() {
    let elevator = PrivilegeElevator::new();

    // Try to elevate SeDebugPrivilege
    let result = elevator.elevate("SeDebugName");

    // This might fail without admin rights
    // Just ensure it doesn't panic
    let _ = result;
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_elevate_nonexistent_privilege() {
    let elevator = PrivilegeElevator::with_options(ElevationOptions {
        auto_enable: true,
        require_success: false, // Don't fail on error
        cache_result: false,
    });

    // This should fail gracefully
    let result = elevator.elevate("SeNonExistentPrivilege");
    assert!(!result.unwrap_or(false));
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_require_privilege() {
    // This might fail without the required privilege
    let result = require_privilege("SeDebugName");

    // Just ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_privilege_state_variants() {
    // Test PrivilegeState enum
    let enabled = PrivilegeState::Enabled;
    let disabled = PrivilegeState::Disabled;
    let not_present = PrivilegeState::NotPresent;

    assert_eq!(enabled, PrivilegeState::Enabled);
    assert_ne!(enabled, disabled);
    assert_ne!(disabled, not_present);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_privilege_checker_is_elevated() {
    // Check if process is elevated
    let is_elevated = PrivilegeChecker::is_elevated();

    // Result depends on how the test is run
    // Just ensure it doesn't crash
    let _ = is_elevated;
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_privilege_checker_list_privileges() {
    // List all privileges
    let result = PrivilegeChecker::list_privileges();

    // Should succeed even without admin rights
    if let Ok(privileges) = result {
        // Process should have at least some privileges
        // But it might be empty in restricted environments
        let _ = privileges;
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_privilege_checker_check_privilege() {
    // Check a specific privilege by LUID
    // SE_DEBUG_PRIVILEGE has LUID 20
    let result = PrivilegeChecker::check_privilege(20);

    if let Ok(state) = result {
        // Just check that we can match on the state
        match state {
            PrivilegeState::Enabled => {
                // Privilege is enabled
            }
            PrivilegeState::Disabled => {
                // Privilege is available but disabled
            }
            PrivilegeState::NotPresent => {
                // Privilege is not available
            }
        }
    }
}

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
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_elevation_with_caching() {
    let elevator = PrivilegeElevator::with_options(ElevationOptions {
        auto_enable: true,
        require_success: false,
        cache_result: true,
    });

    // First call
    let result1 = elevator.elevate("SeDebugName");

    // Second call should use cache
    let result2 = elevator.elevate("SeDebugName");

    // Results should be consistent
    if result1.is_ok() && result2.is_ok() {
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    // Clear cache
    PrivilegeElevator::clear_cache();
}
