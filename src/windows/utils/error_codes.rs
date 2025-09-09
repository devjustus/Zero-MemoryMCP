//! Windows error code handling utilities

use crate::core::types::MemoryError;
use std::fmt;
use winapi::um::errhandlingapi::GetLastError;

/// Common Windows error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    Success = 0,
    AccessDenied = 5,
    InvalidHandle = 6,
    InvalidParameter = 87,
    InsufficientBuffer = 122,
    InvalidAddress = 487,
    PartialCopy = 299,
    Unknown(u32),
}

impl From<u32> for ErrorCode {
    fn from(code: u32) -> Self {
        match code {
            0 => ErrorCode::Success,
            5 => ErrorCode::AccessDenied,
            6 => ErrorCode::InvalidHandle,
            87 => ErrorCode::InvalidParameter,
            122 => ErrorCode::InsufficientBuffer,
            299 => ErrorCode::PartialCopy,
            487 => ErrorCode::InvalidAddress,
            _ => ErrorCode::Unknown(code),
        }
    }
}

impl ErrorCode {
    /// Get the last Windows error
    pub fn last_error() -> Self {
        unsafe { ErrorCode::from(GetLastError()) }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Success => write!(f, "Success"),
            ErrorCode::AccessDenied => write!(f, "Access denied"),
            ErrorCode::InvalidHandle => write!(f, "Invalid handle"),
            ErrorCode::InvalidParameter => write!(f, "Invalid parameter"),
            ErrorCode::InsufficientBuffer => write!(f, "Insufficient buffer"),
            ErrorCode::InvalidAddress => write!(f, "Invalid address"),
            ErrorCode::PartialCopy => write!(f, "Partial copy"),
            ErrorCode::Unknown(code) => write!(f, "Unknown error: {}", code),
        }
    }
}

/// Windows error wrapper
pub struct WinError {
    code: ErrorCode,
    context: String,
}

impl WinError {
    /// Create a new Windows error with context
    pub fn new(context: impl Into<String>) -> Self {
        WinError {
            code: ErrorCode::last_error(),
            context: context.into(),
        }
    }

    /// Create with specific error code
    pub fn with_code(code: ErrorCode, context: impl Into<String>) -> Self {
        WinError {
            code,
            context: context.into(),
        }
    }

    /// Convert to MemoryError
    pub fn to_memory_error(self) -> MemoryError {
        MemoryError::WindowsApi(format!("{}: {}", self.context, self.code))
    }
}

/// Get last Windows error as MemoryError
pub fn last_error_as_memory_error(context: impl Into<String>) -> MemoryError {
    WinError::new(context).to_memory_error()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_conversion() {
        // Test all error code conversions
        assert_eq!(ErrorCode::from(0), ErrorCode::Success);
        assert_eq!(ErrorCode::from(5), ErrorCode::AccessDenied);
        assert_eq!(ErrorCode::from(6), ErrorCode::InvalidHandle);
        assert_eq!(ErrorCode::from(87), ErrorCode::InvalidParameter);
        assert_eq!(ErrorCode::from(122), ErrorCode::InsufficientBuffer);
        assert_eq!(ErrorCode::from(299), ErrorCode::PartialCopy);
        assert_eq!(ErrorCode::from(487), ErrorCode::InvalidAddress);
        assert_eq!(ErrorCode::from(999), ErrorCode::Unknown(999));
    }

    #[test]
    fn test_error_code_display() {
        // Test all error code display strings
        assert_eq!(format!("{}", ErrorCode::Success), "Success");
        assert_eq!(format!("{}", ErrorCode::AccessDenied), "Access denied");
        assert_eq!(format!("{}", ErrorCode::InvalidHandle), "Invalid handle");
        assert_eq!(
            format!("{}", ErrorCode::InvalidParameter),
            "Invalid parameter"
        );
        assert_eq!(
            format!("{}", ErrorCode::InsufficientBuffer),
            "Insufficient buffer"
        );
        assert_eq!(format!("{}", ErrorCode::InvalidAddress), "Invalid address");
        assert_eq!(format!("{}", ErrorCode::PartialCopy), "Partial copy");
        assert_eq!(format!("{}", ErrorCode::Unknown(123)), "Unknown error: 123");
    }

    #[test]
    fn test_win_error() {
        // Test WinError with specific code
        let err = WinError::with_code(ErrorCode::InvalidHandle, "test operation");
        let mem_err = err.to_memory_error();
        assert!(mem_err.to_string().contains("Invalid handle"));
        assert!(mem_err.to_string().contains("test operation"));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_win_error_new() {
        // Test WinError::new which gets last error
        let err = WinError::new("operation failed");
        let mem_err = err.to_memory_error();
        assert!(mem_err.to_string().contains("operation failed"));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_last_error_as_memory_error() {
        // Test the helper function
        let mem_err = last_error_as_memory_error("system call failed");
        assert!(mem_err.to_string().contains("system call failed"));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_error_code_last_error() {
        // Test ErrorCode::last_error()
        let _code = ErrorCode::last_error();
        // Just ensure it doesn't crash
    }

    #[test]
    fn test_error_code_equality() {
        // Test Clone, Copy, PartialEq, Eq derives
        let err1 = ErrorCode::AccessDenied;
        let err2 = err1; // Copy
        let err3 = err1.clone(); // Clone
        assert_eq!(err1, err2); // PartialEq
        assert_eq!(err1, err3);

        let err4 = ErrorCode::InvalidHandle;
        assert_ne!(err1, err4);
    }

    #[test]
    fn test_error_code_debug() {
        // Test Debug implementation
        let err = ErrorCode::AccessDenied;
        let debug_str = format!("{:?}", err);
        assert_eq!(debug_str, "AccessDenied");

        let unknown = ErrorCode::Unknown(42);
        let debug_str = format!("{:?}", unknown);
        assert_eq!(debug_str, "Unknown(42)");
    }
}
