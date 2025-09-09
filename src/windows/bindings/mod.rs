//! Windows API bindings
//!
//! Low-level FFI bindings to Windows system libraries.

pub mod kernel32;
pub mod ntdll;
pub mod psapi;

// Re-export all bindings
pub use kernel32::*;
pub use ntdll::*;
pub use psapi::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_bindings_available() {
        // Bindings should be available - compile-time test
    }
}
