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

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    #[error("Process already attached: {0}")]
    ProcessAlreadyAttached(u32),

    #[error("Windows API error: {0}")]
    WindowsApiError(#[from] windows::core::Error),

    #[error("Windows API: {0}")]
    WindowsApi(String),

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
        assert_eq!(
            err.to_string(),
            "Access denied to process 1234: SeDebugPrivilege required"
        );
    }

    #[test]
    fn test_all_error_variants() {
        let errors: Vec<(MemoryError, &str)> = vec![
            (
                MemoryError::InvalidAddress("0x123".to_string()),
                "Invalid memory address: 0x123",
            ),
            (
                MemoryError::ProcessNotFound("notepad.exe".to_string()),
                "Process not found: notepad.exe",
            ),
            (
                MemoryError::AccessDenied {
                    pid: 999,
                    reason: "denied".to_string(),
                },
                "Access denied to process 999: denied",
            ),
            (
                MemoryError::ReadFailed {
                    address: "0x1000".to_string(),
                    reason: "page fault".to_string(),
                },
                "Failed to read memory at 0x1000: page fault",
            ),
            (
                MemoryError::WriteFailed {
                    address: "0x2000".to_string(),
                    reason: "write protected".to_string(),
                },
                "Failed to write memory at 0x2000: write protected",
            ),
            (
                MemoryError::InvalidValueType("unknown".to_string()),
                "Invalid value type: unknown",
            ),
            (
                MemoryError::SessionNotFound("session123".to_string()),
                "Scan session not found: session123",
            ),
            (
                MemoryError::ModuleNotFound("kernel32.dll".to_string()),
                "Module not found: kernel32.dll",
            ),
            (MemoryError::PatternNotFound, "Pattern not found in memory"),
            (
                MemoryError::InvalidPattern("?? ?? XX".to_string()),
                "Invalid pattern format: ?? ?? XX",
            ),
            (
                MemoryError::PointerChainBroken {
                    level: 3,
                    reason: "null pointer".to_string(),
                },
                "Pointer chain broken at level 3: null pointer",
            ),
            (
                MemoryError::InsufficientPrivileges("admin required".to_string()),
                "Insufficient privileges: admin required",
            ),
            (
                MemoryError::ProtectionError("PAGE_NOACCESS".to_string()),
                "Memory protection error: PAGE_NOACCESS",
            ),
            (
                MemoryError::BufferTooSmall {
                    expected: 100,
                    actual: 50,
                },
                "Buffer too small: expected 100, got 50",
            ),
            (
                MemoryError::UnsupportedOperation("AOB scan".to_string()),
                "Unsupported operation: AOB scan",
            ),
            (
                MemoryError::Unknown("something went wrong".to_string()),
                "Unknown error: something went wrong",
            ),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_helper_methods() {
        let err = MemoryError::access_denied(42, "test reason");
        match err {
            MemoryError::AccessDenied { pid, reason } => {
                assert_eq!(pid, 42);
                assert_eq!(reason, "test reason");
            }
            _ => panic!("Wrong error type"),
        }

        let err = MemoryError::read_failed("0xABCD", "invalid page");
        match err {
            MemoryError::ReadFailed { address, reason } => {
                assert_eq!(address, "0xABCD");
                assert_eq!(reason, "invalid page");
            }
            _ => panic!("Wrong error type"),
        }

        let err = MemoryError::write_failed("0xDEAD", "protected memory");
        match err {
            MemoryError::WriteFailed { address, reason } => {
                assert_eq!(address, "0xDEAD");
                assert_eq!(reason, "protected memory");
            }
            _ => panic!("Wrong error type"),
        }

        let err = MemoryError::pointer_chain_broken(5, "dereferenced null");
        match err {
            MemoryError::PointerChainBroken { level, reason } => {
                assert_eq!(level, 5);
                assert_eq!(reason, "dereferenced null");
            }
            _ => panic!("Wrong error type"),
        }

        let err = MemoryError::buffer_too_small(256, 128);
        match err {
            MemoryError::BufferTooSmall { expected, actual } => {
                assert_eq!(expected, 256);
                assert_eq!(actual, 128);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_from_implementations() {
        use std::io;

        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "test");
        let mem_err: MemoryError = io_err.into();
        assert!(matches!(mem_err, MemoryError::IoError(_)));

        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let mem_err: MemoryError = json_err.into();
        assert!(matches!(mem_err, MemoryError::JsonError(_)));

        let utf8_err = String::from_utf8(vec![0xFF, 0xFE, 0xFD]).unwrap_err();
        let mem_err: MemoryError = utf8_err.into();
        assert!(matches!(mem_err, MemoryError::Utf8Error(_)));
    }

    #[test]
    fn test_memory_result_type() {
        fn example_function() -> MemoryResult<u32> {
            Ok(42)
        }

        fn failing_function() -> MemoryResult<u32> {
            Err(MemoryError::Unknown("test".to_string()))
        }

        assert_eq!(example_function().unwrap(), 42);
        assert!(failing_function().is_err());
    }

    #[test]
    fn test_error_debug_format() {
        let err = MemoryError::InvalidAddress("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidAddress"));
        assert!(debug_str.contains("test"));
    }
}
