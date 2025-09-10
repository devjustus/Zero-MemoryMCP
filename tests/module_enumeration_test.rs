//! Integration tests for module enumeration

use memory_mcp::process::{
    enumerate_modules, find_module_by_name, get_process_main_module, ModuleEnumerator,
    ProcessHandle,
};
use std::process;

#[test]
fn test_module_enumerator_with_current_process() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    // Enumerator should be created without issues
    drop(enumerator);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_enumerate_current_process_modules() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    let modules = enumerator.enumerate().expect("Failed to enumerate modules");

    // Current process should have at least one module (the executable)
    assert!(!modules.is_empty(), "No modules found in current process");

    // Check the main module
    let main_module = &modules[0];
    assert!(!main_module.name.is_empty(), "Main module has no name");
    assert!(main_module.size > 0, "Main module has zero size");
    assert!(
        !main_module.base_address.is_null(),
        "Main module has null base address"
    );
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_find_system_dlls() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    // These DLLs should be present in every Windows process
    let system_dlls = ["kernel32.dll", "ntdll.dll", "kernelbase.dll"];

    for dll_name in &system_dlls {
        let module = enumerator
            .find_by_name(dll_name)
            .expect("Failed to search for module");

        assert!(
            module.is_some(),
            "System DLL {} not found in process",
            dll_name
        );

        if let Some(module) = module {
            assert_eq!(
                module.name.to_lowercase(),
                dll_name.to_lowercase(),
                "Module name mismatch"
            );
            assert!(module.size > 0, "Module {} has zero size", dll_name);
        }
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_get_main_module() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    let main_module = enumerator
        .get_main_module()
        .expect("Failed to get main module");

    // Main module should have valid properties
    assert!(!main_module.name.is_empty(), "Main module has no name");
    assert!(main_module.size > 0, "Main module has zero size");
    assert!(
        !main_module.base_address.is_null(),
        "Main module has null base address"
    );

    // Main module name should end with .exe
    assert!(
        main_module.name.to_lowercase().ends_with(".exe"),
        "Main module is not an executable: {}",
        main_module.name
    );
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_module_path_information() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    let modules = enumerator.enumerate().expect("Failed to enumerate modules");

    // At least some modules should have path information
    let modules_with_paths: Vec<_> = modules
        .iter()
        .filter(|m| !m.path.as_os_str().is_empty())
        .collect();

    assert!(
        !modules_with_paths.is_empty(),
        "No modules have path information"
    );

    // Paths should be valid Windows paths
    for module in modules_with_paths {
        let path = module.path.to_string_lossy();
        assert!(
            path.contains("\\") || path.contains("/"),
            "Invalid path format: {}",
            path
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_enumerate_modules_helper_function() {
    let pid = process::id();

    let modules = enumerate_modules(pid).expect("Failed to enumerate modules");

    assert!(
        !modules.is_empty(),
        "No modules found using helper function"
    );

    // Verify we can find system DLLs
    let has_kernel32 = modules
        .iter()
        .any(|m| m.name.eq_ignore_ascii_case("kernel32.dll"));
    assert!(has_kernel32, "kernel32.dll not found in module list");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_find_module_by_name_helper() {
    let pid = process::id();

    // Test finding a module that should exist
    let module = find_module_by_name(pid, "ntdll.dll").expect("Failed to search for module");

    assert!(module.is_some(), "ntdll.dll not found");

    if let Some(module) = module {
        assert_eq!(module.name.to_lowercase(), "ntdll.dll");
        assert!(module.size > 0);
    }

    // Test finding a module that shouldn't exist
    let nonexistent =
        find_module_by_name(pid, "nonexistent.dll").expect("Failed to search for module");

    assert!(nonexistent.is_none(), "Found non-existent module");
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_get_process_main_module_helper() {
    let pid = process::id();

    let main_module = get_process_main_module(pid).expect("Failed to get main module");

    assert!(!main_module.name.is_empty());
    assert!(main_module.name.to_lowercase().ends_with(".exe"));
    assert!(main_module.size > 0);
    assert!(!main_module.base_address.is_null());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_case_insensitive_module_search() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    // Test different case variations
    let test_cases = [
        "KERNEL32.DLL",
        "kernel32.dll",
        "Kernel32.dll",
        "KerNeL32.DlL",
    ];

    for test_case in &test_cases {
        let module = enumerator
            .find_by_name(test_case)
            .expect("Failed to search for module");

        assert!(
            module.is_some(),
            "Failed to find module with case variation: {}",
            test_case
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_module_address_ranges() {
    let handle =
        ProcessHandle::open_for_read(process::id()).expect("Failed to open current process");
    let enumerator = ModuleEnumerator::new(handle);

    let modules = enumerator.enumerate().expect("Failed to enumerate modules");

    // Check that modules have non-overlapping address ranges
    for (i, module_a) in modules.iter().enumerate() {
        let base_a = module_a.base_address.as_usize();
        let end_a = base_a + module_a.size;

        for module_b in modules.iter().skip(i + 1) {
            let base_b = module_b.base_address.as_usize();
            let end_b = base_b + module_b.size;

            // Check for no overlap
            let no_overlap = end_a <= base_b || end_b <= base_a;
            assert!(
                no_overlap,
                "Modules {} and {} have overlapping address ranges",
                module_a.name, module_b.name
            );
        }
    }
}
