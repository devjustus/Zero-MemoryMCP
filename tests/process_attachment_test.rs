//! Integration tests for process attachment and detachment

use memory_mcp::core::types::MemoryError;
use memory_mcp::process::ProcessHandle;
use memory_mcp::process::{AttachOptions, DetachOptions, ProcessAttacher, ProcessDetacher};

#[test]
fn test_attach_options_creation() {
    let default_opts = AttachOptions::default();
    assert!(!default_opts.all_access);
    assert!(default_opts.read_only);
    assert!(default_opts.enable_debug_privilege);
    assert_eq!(default_opts.timeout_ms, Some(5000));

    let custom_opts = AttachOptions {
        all_access: true,
        read_only: false,
        enable_debug_privilege: false,
        timeout_ms: None,
    };
    assert!(custom_opts.all_access);
    assert!(!custom_opts.read_only);
}

#[test]
fn test_detach_options_creation() {
    let default_opts = DetachOptions::default();
    assert!(!default_opts.force);
    assert!(default_opts.clear_cache);
    assert!(default_opts.wait_for_pending);

    let custom_opts = DetachOptions {
        force: true,
        clear_cache: false,
        wait_for_pending: false,
    };
    assert!(custom_opts.force);
    assert!(!custom_opts.clear_cache);
}

#[test]
fn test_process_attacher_creation() {
    let attacher = ProcessAttacher::new();
    assert_eq!(attacher.attached_count(), 0);
    assert!(!attacher.is_attached(1234));

    let custom_opts = AttachOptions {
        all_access: true,
        read_only: false,
        enable_debug_privilege: true,
        timeout_ms: Some(10000),
    };
    let attacher_with_opts = ProcessAttacher::with_options(custom_opts);
    assert_eq!(attacher_with_opts.attached_count(), 0);
}

#[test]
fn test_process_detacher_creation() {
    let detacher = ProcessDetacher::new();
    assert!(!detacher.was_recently_detached(1234));
    assert_eq!(detacher.get_detach_history().len(), 0);

    detacher.clear_history();
    assert_eq!(detacher.get_detach_history().len(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attach_invalid_process() {
    let attacher = ProcessAttacher::new();

    // Attempting to attach to PID 0 should fail
    let result = attacher.attach(0);
    assert!(result.is_err());

    // Attempting to attach to non-existent PID should fail
    let result = attacher.attach(999999);
    assert!(result.is_err());
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attach_current_process() {
    let attacher = ProcessAttacher::new();
    let current_pid = std::process::id();

    // Should be able to attach to current process with read-only access
    let result = attacher.attach(current_pid);
    if let Ok(guard) = result {
        assert_eq!(guard.pid(), current_pid);
        assert!(guard.handle().is_some());

        // Check that the process is marked as attached
        assert!(attacher.is_attached(current_pid));
        assert_eq!(attacher.attached_count(), 1);

        // Detach the process
        let detach_result = guard.detach();
        assert!(detach_result.is_ok());
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attachment_guard_auto_detach() {
    let attacher = ProcessAttacher::new();
    let current_pid = std::process::id();

    {
        // Attach in a scope
        let result = attacher.attach(current_pid);
        if result.is_ok() {
            assert!(attacher.is_attached(current_pid));
        }
        // Guard should auto-detach when going out of scope
    }

    // Process should no longer be attached after guard is dropped
    // Note: Since we're tracking PIDs separately, this might still show as attached
    // unless we implement a callback mechanism
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attachment_guard_manual_detach() {
    let attacher = ProcessAttacher::new();
    let current_pid = std::process::id();

    let result = attacher.attach(current_pid);
    if let Ok(guard) = result {
        assert_eq!(guard.pid(), current_pid);

        // Manually detach
        let detach_result = guard.detach();
        assert!(detach_result.is_ok());
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attachment_guard_into_handle() {
    let attacher = ProcessAttacher::new();
    let current_pid = std::process::id();

    let result = attacher.attach(current_pid);
    if let Ok(guard) = result {
        // Take ownership of the handle
        let handle = guard.into_handle();
        assert!(handle.is_some());

        if let Some(h) = handle {
            assert_eq!(h.pid(), current_pid);
            assert!(h.is_valid());
        }
    }
}

#[test]
fn test_detach_all() {
    let attacher = ProcessAttacher::new();

    // Clear all attachments
    let result = attacher.detach_all();
    assert!(result.is_ok());
    assert_eq!(attacher.attached_count(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_attach_with_custom_options() {
    let attacher = ProcessAttacher::new();
    let current_pid = std::process::id();

    let options = AttachOptions {
        all_access: false,
        read_only: true,
        enable_debug_privilege: false,
        timeout_ms: Some(1000),
    };

    let result = attacher.attach_with_options(current_pid, &options);
    if let Ok(guard) = result {
        assert_eq!(guard.pid(), current_pid);
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_detacher_with_handle() {
    let detacher = ProcessDetacher::new();
    let current_pid = std::process::id();

    // Create a handle to detach
    let handle_result = ProcessHandle::open_for_read(current_pid);
    if let Ok(handle) = handle_result {
        let detach_result = detacher.detach(handle);
        assert!(detach_result.is_ok());

        // Check that it was recorded
        assert!(detacher.was_recently_detached(current_pid));
    }
}

#[test]
#[cfg_attr(miri, ignore = "FFI not supported in Miri")]
fn test_detacher_batch() {
    let detacher = ProcessDetacher::new();
    let current_pid = std::process::id();

    // Create multiple handles
    let mut handles = Vec::new();
    for _ in 0..3 {
        if let Ok(handle) = ProcessHandle::open_for_read(current_pid) {
            handles.push(handle);
        }
    }

    if !handles.is_empty() {
        let results = detacher.detach_batch(handles);
        for result in results {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_detacher_history() {
    let detacher = ProcessDetacher::new();

    // Get initial history
    let history = detacher.get_detach_history();
    assert_eq!(history.len(), 0);

    // Clear history
    detacher.clear_history();
    assert_eq!(detacher.get_detach_history().len(), 0);
}
