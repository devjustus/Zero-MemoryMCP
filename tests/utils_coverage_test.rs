//! Additional tests to improve coverage for Windows utilities

use memory_mcp::windows::utils::error_codes::{last_error_as_memory_error, ErrorCode, WinError};
use memory_mcp::windows::utils::string_conv::{
    extract_filename, normalize_path, string_to_wide, wide_to_string,
};

#[test]
fn test_error_code_all_variants() {
    // Test all error code variants for complete coverage
    let codes = vec![
        (0, ErrorCode::Success),
        (5, ErrorCode::AccessDenied),
        (6, ErrorCode::InvalidHandle),
        (87, ErrorCode::InvalidParameter),
        (122, ErrorCode::InsufficientBuffer),
        (299, ErrorCode::PartialCopy),
        (487, ErrorCode::InvalidAddress),
        (999, ErrorCode::Unknown(999)),
    ];

    for (code, expected) in codes {
        let result = ErrorCode::from(code);
        assert_eq!(result, expected);

        // Also test Display implementation
        let display_str = format!("{}", result);
        assert!(!display_str.is_empty());
    }
}

#[test]
fn test_win_error_context() {
    // Test WinError with various contexts
    let contexts = vec![
        "memory read failed",
        "process open failed",
        "invalid handle",
        "access denied",
    ];

    for context in contexts {
        let err = WinError::with_code(ErrorCode::AccessDenied, context);
        let mem_err = err.to_memory_error();
        assert!(mem_err.to_string().contains(context));
        assert!(mem_err.to_string().contains("Access denied"));
    }
}

#[test]
fn test_string_conversions_edge_cases() {
    // Test empty string
    let empty_wide = string_to_wide("");
    assert_eq!(empty_wide, vec![0]);

    // Test string with special characters
    let special = "Hello\n\r\t\\World";
    let wide = string_to_wide(special);
    let back = wide_to_string(&wide);
    assert_eq!(back, special);

    // Test very long string
    let long_str = "a".repeat(1000);
    let wide = string_to_wide(&long_str);
    let back = wide_to_string(&wide);
    assert_eq!(back, long_str);
}

#[test]
fn test_path_extraction_edge_cases() {
    // Test various path formats
    let test_cases = vec![
        ("", ""),
        ("file.txt", "file.txt"),
        ("C:\\", ""),
        ("\\\\server\\share\\", ""),
        ("C:\\Windows\\System32\\", ""),
        ("relative\\path\\file.dll", "file.dll"),
        ("..\\..\\file.exe", "file.exe"),
        ("C:\\Path With Spaces\\file.txt", "file.txt"),
    ];

    for (input, expected) in test_cases {
        let result = extract_filename(input);
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_path_normalization_edge_cases() {
    // Test various path formats
    let test_cases = vec![
        ("", ""),
        ("/", "\\"),
        ("//", "\\\\"),
        ("C:/Windows/System32", "C:\\Windows\\System32"),
        ("mixed/path\\already", "mixed\\path\\already"),
        (
            "///multiple///slashes///",
            "\\\\\\multiple\\\\\\slashes\\\\\\",
        ),
    ];

    for (input, expected) in test_cases {
        let result = normalize_path(input);
        assert_eq!(result, expected, "Failed for input: {}", input);
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_last_error_context_variations() {
    // Test various contexts for last_error_as_memory_error
    let contexts = vec![
        "OpenProcess failed",
        "ReadProcessMemory failed",
        "VirtualQueryEx failed",
        "Invalid memory address",
        "",
    ];

    for context in contexts {
        let mem_err = last_error_as_memory_error(context);
        if !context.is_empty() {
            assert!(mem_err.to_string().contains(context));
        }
    }
}

#[test]
fn test_module_info_edge_cases() {
    use memory_mcp::core::types::Address;
    use memory_mcp::windows::types::module_info::ModuleInfo;
    use winapi::um::psapi::MODULEINFO;

    // Test with zero-sized module
    let info = MODULEINFO {
        lpBaseOfDll: 0x10000 as *mut _,
        SizeOfImage: 0,
        EntryPoint: 0x10000 as *mut _,
    };

    let module = ModuleInfo::new("zero_size.dll".to_string(), info);
    assert_eq!(module.size, 0);
    assert_eq!(module.end_address(), module.base_address);

    // Test contains_address at boundaries
    assert!(!module.contains_address(Address::new(0x10001))); // Size is 0, so only base address itself would be "contained" if size > 0

    // Test with max size
    let info = MODULEINFO {
        lpBaseOfDll: 0x10000 as *mut _,
        SizeOfImage: 0xFFFFFFFF,
        EntryPoint: 0x10000 as *mut _,
    };

    let module = ModuleInfo::new("max_size.dll".to_string(), info);
    assert_eq!(module.size, 0xFFFFFFFF);

    // Test system module detection with mixed case
    let system_names = vec![
        "KERNEL32.DLL",
        "kernel32.dll",
        "KerNel32.DLL",
        "ntdll.dll",
        "USER32.DLL",
        "advapi32.dll",
    ];

    for name in system_names {
        let module = ModuleInfo::new(name.to_string(), info);
        assert!(module.is_system_module(), "Failed for: {}", name);
    }

    // Test non-system modules
    let non_system_names = vec!["custom.dll", "game.exe", "my_app.dll", "32kernel.dll"];

    for name in non_system_names {
        let module = ModuleInfo::new(name.to_string(), info);
        assert!(!module.is_system_module(), "Failed for: {}", name);
    }
}
