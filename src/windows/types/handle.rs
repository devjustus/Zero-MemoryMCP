//! Safe HANDLE wrapper with automatic cleanup

use crate::windows::bindings::kernel32;
use std::ptr;
use winapi::um::winnt::HANDLE;

/// Safe wrapper around Windows HANDLE with RAII semantics
pub struct Handle {
    handle: HANDLE,
}

impl Handle {
    /// Create a new Handle wrapper
    pub fn new(handle: HANDLE) -> Self {
        Handle { handle }
    }

    /// Create a null handle
    pub fn null() -> Self {
        Handle {
            handle: ptr::null_mut(),
        }
    }

    /// Check if handle is null
    pub fn is_null(&self) -> bool {
        self.handle.is_null()
    }

    /// Get the raw handle
    pub fn raw(&self) -> HANDLE {
        self.handle
    }

    /// Take ownership of the handle, preventing automatic cleanup
    pub fn take(mut self) -> HANDLE {
        let handle = self.handle;
        self.handle = ptr::null_mut();
        handle
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // Ignore errors on cleanup
            unsafe {
                let _ = kernel32::close_handle(self.handle);
            }
        }
    }
}

// Send + Sync are safe because HANDLEs are process-local
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_creation() {
        let handle = Handle::null();
        assert!(handle.is_null());
        assert_eq!(handle.raw(), ptr::null_mut());
    }

    #[test]
    fn test_handle_take() {
        let handle = Handle::new(ptr::null_mut());
        let raw = handle.take();
        assert_eq!(raw, ptr::null_mut());
    }

    #[test]
    fn test_handle_drop() {
        // Create handle in scope and let it drop
        {
            let _handle = Handle::null();
        }
        // Should not crash
    }
}
