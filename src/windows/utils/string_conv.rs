//! String conversion utilities for Windows API

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

/// Convert a Rust string to Windows wide string (UTF-16)
pub fn string_to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert Windows wide string (UTF-16) to Rust string
pub fn wide_to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    let os_string = OsString::from_wide(&wide[..len]);
    os_string.to_string_lossy().into_owned()
}

/// Convert Windows wide string pointer to Rust string
///
/// # Safety
/// The pointer must be valid and point to a null-terminated UTF-16 string
pub unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    let mut len = 0;
    while *ptr.offset(len) != 0 {
        len += 1;
    }

    let slice = std::slice::from_raw_parts(ptr, len as usize);
    wide_to_string(slice)
}

/// Extract filename from full path
pub fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

/// Normalize Windows path
pub fn normalize_path(path: &str) -> String {
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_wide() {
        let wide = string_to_wide("Hello");
        assert_eq!(wide, vec![72, 101, 108, 108, 111, 0]);

        let empty = string_to_wide("");
        assert_eq!(empty, vec![0]);
    }

    #[test]
    fn test_wide_to_string() {
        let wide = vec![72, 101, 108, 108, 111, 0];
        assert_eq!(wide_to_string(&wide), "Hello");

        let no_null = vec![72, 101, 108, 108, 111];
        assert_eq!(wide_to_string(&no_null), "Hello");
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            extract_filename("C:\\Windows\\System32\\kernel32.dll"),
            "kernel32.dll"
        );
        assert_eq!(extract_filename("kernel32.dll"), "kernel32.dll");
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("C:/Windows/System32"),
            "C:\\Windows\\System32"
        );
        assert_eq!(
            normalize_path("C:\\Windows\\System32"),
            "C:\\Windows\\System32"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore = "Unsafe pointer operations")]
    fn test_wide_ptr_to_string() {
        // Test null pointer
        unsafe {
            assert_eq!(wide_ptr_to_string(std::ptr::null()), "");
        }

        // Test valid string
        let wide_str = vec![72u16, 101, 108, 108, 111, 0]; // "Hello\0"
        unsafe {
            assert_eq!(wide_ptr_to_string(wide_str.as_ptr()), "Hello");
        }
    }

    #[test]
    fn test_unicode_strings() {
        // Test unicode characters
        let unicode_str = "Hello 世界 🌍";
        let wide = string_to_wide(unicode_str);
        let back = wide_to_string(&wide);
        assert_eq!(back, unicode_str);
    }

    #[test]
    fn test_path_edge_cases() {
        // Test various path formats
        assert_eq!(extract_filename("C:\\"), "");
        assert_eq!(extract_filename("\\\\server\\share\\file.txt"), "file.txt");
        assert_eq!(
            normalize_path("C:/Windows//System32"),
            "C:\\Windows\\\\System32"
        );
        assert_eq!(normalize_path("/usr/local/bin"), "\\usr\\local\\bin");
    }
}
