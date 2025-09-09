//! Windows utility functions

pub mod error_codes;
pub mod string_conv;

// Re-export commonly used utilities
pub use error_codes::{ErrorCode, WinError};
pub use string_conv::{string_to_wide, wide_to_string};

#[cfg(test)]
mod tests {
    #[test]
    fn test_utils_available() {
        // Utils should be available - compile-time test
    }
}
