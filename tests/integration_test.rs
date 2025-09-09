#[test]
fn test_basic_setup() {
    // Verify project compiles and basic setup works
    assert_eq!(2 + 2, 4);
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_platform() {
    // Verify we're on Windows
    assert!(cfg!(target_os = "windows"));
}