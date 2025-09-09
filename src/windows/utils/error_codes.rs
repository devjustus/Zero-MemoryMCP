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
        assert_eq!(ErrorCode::from(0), ErrorCode::Success);
        assert_eq!(ErrorCode::from(5), ErrorCode::AccessDenied);
        assert_eq!(ErrorCode::from(999), ErrorCode::Unknown(999));
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(format!("{}", ErrorCode::Success), "Success");
        assert_eq!(format!("{}", ErrorCode::AccessDenied), "Access denied");
        assert_eq!(format!("{}", ErrorCode::Unknown(123)), "Unknown error: 123");
    }

    #[test]
    fn test_win_error() {
        let err = WinError::with_code(ErrorCode::InvalidHandle, "test operation");
        let mem_err = err.to_memory_error();
        assert!(mem_err.to_string().contains("Invalid handle"));
    }
}
