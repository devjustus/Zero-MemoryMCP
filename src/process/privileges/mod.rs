//! Windows privilege management

pub mod checker;
pub mod debug;
pub mod elevate;

pub use checker::{PrivilegeChecker, PrivilegeState};
pub use debug::{enable_debug_privilege, has_debug_privilege, DebugPrivilegeGuard};
pub use elevate::{require_privilege, ElevationOptions, PrivilegeElevator};
