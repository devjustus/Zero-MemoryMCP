//! Custom error types for Memory-MCP

use std::fmt;
use thiserror::Error;

/// Main error type for memory operations
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Invalid memory address: {0}")]
    InvalidAddress(String),

    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Access denied to process {pid}: {reason}")]
    AccessDenied { pid: u32, reason: String },

    #[error("Failed to read memory at {address}: {reason}")]
    ReadFailed { address: String, reason: String },

    #[error("Failed to write memory at {address}: {reason}")]
    WriteFailed { address: String, reason: String },

    #[error("Invalid value type: {0}")]
    InvalidValueType(String),

    #[error("Scan session not found: {0}")]
    SessionNotFound(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Pattern not found in memory")]
    PatternNotFound,

    #[error("Invalid pattern format: {0}")]
    InvalidPattern(String),

    #[error("Pointer chain broken at level {level}: {reason}")]
    PointerChainBroken { level: usize, reason: String },

    #[error("Insufficient privileges: {0}")]
    InsufficientPrivileges(String),

    #[error("Memory protection error: {0}")]
    ProtectionError(String),

    #[error("Buffer too small: expected {expected}, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Windows API error: {0}")]
    WindowsApiError(#[from] windows::core::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

impl MemoryError {
    /// Creates a new Windows API error with the last error code
    pub fn last_os_error() -> Self {
        MemoryError::WindowsApiError(windows::core::Error::from_win32())
    }

    /// Creates an access denied error for a process
    pub fn access_denied(pid: u32, reason: impl Into<String>) -> Self {
        MemoryError::AccessDenied {
            pid,
            reason: reason.into(),
        }
    }

    /// Creates a read failed error
    pub fn read_failed(address: impl fmt::Display, reason: impl Into<String>) -> Self {
        MemoryError::ReadFailed {
            address: address.to_string(),
            reason: reason.into(),
        }
    }

    /// Creates a write failed error
    pub fn write_failed(address: impl fmt::Display, reason: impl Into<String>) -> Self {
        MemoryError::WriteFailed {
            address: address.to_string(),
            reason: reason.into(),
        }
    }

    /// Creates a pointer chain broken error
    pub fn pointer_chain_broken(level: usize, reason: impl Into<String>) -> Self {
        MemoryError::PointerChainBroken {
            level,
            reason: reason.into(),
        }
    }

    /// Creates a buffer too small error
    pub fn buffer_too_small(expected: usize, actual: usize) -> Self {
        MemoryError::BufferTooSmall { expected, actual }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MemoryError::InvalidAddress("0xDEADBEEF".to_string());
        assert_eq!(err.to_string(), "Invalid memory address: 0xDEADBEEF");

        let err = MemoryError::access_denied(1234, "SeDebugPrivilege required");
        assert_eq!(err.to_string(), "Access denied to process 1234: SeDebugPrivilege required");
    }
}