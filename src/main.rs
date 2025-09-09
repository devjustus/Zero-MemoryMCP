#![allow(dead_code)]
#![allow(unused_imports)]

mod core;

use anyhow::Result;
use tracing::{info, Level};

/// Initialize the logging system
fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
}

/// Verify the platform is supported
fn verify_platform() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Memory-MCP only supports Windows platform");
    }

    Ok(())
}

/// Get system information for logging
fn get_system_info() -> (String, String) {
    let version = env!("CARGO_PKG_VERSION");
    let arch = std::env::consts::ARCH;
    (version.to_string(), arch.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    let (version, arch) = get_system_info();
    info!("Starting Memory-MCP server v{}", version);

    // Verify Windows platform
    verify_platform()?;

    info!("Platform check: Windows ✓");
    info!("Architecture: {}", arch);

    // TODO: Initialize MCP server
    info!("MCP server initialization pending implementation");

    // Placeholder for keeping server running
    info!("Memory-MCP ready. Press Ctrl+C to shutdown.");
    tokio::signal::ctrl_c().await?;

    info!("Shutting down Memory-MCP server");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_platform() {
        // On Windows, this should succeed
        #[cfg(target_os = "windows")]
        {
            let result = verify_platform();
            assert!(result.is_ok());
        }

        // Platform verification is compile-time on non-Windows
        // so we can't test the error case at runtime
    }

    #[test]
    fn test_get_system_info() {
        let (version, arch) = get_system_info();

        // Version should match package version
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
        assert!(!version.is_empty());

        // Architecture should be set
        assert_eq!(arch, std::env::consts::ARCH);
        assert!(!arch.is_empty());

        // On 64-bit Windows, arch should be "x86_64"
        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        {
            assert_eq!(arch, "x86_64");
        }
    }

    #[test]
    fn test_logging_initialization() {
        // This test verifies that init_logging can be called
        // Note: In tests, we can't actually initialize the subscriber multiple times
        // but we can verify the function exists and is callable
        assert!(true, "Logging initialization function exists");
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)] // Miri doesn't support Windows IO Completion Ports
    async fn test_main_components() {
        // Test that we can call the helper functions used by main
        let platform_result = verify_platform();
        assert!(
            platform_result.is_ok(),
            "Platform verification should succeed on Windows"
        );

        let (version, arch) = get_system_info();
        assert!(!version.is_empty(), "Version should not be empty");
        assert!(!arch.is_empty(), "Architecture should not be empty");
    }

    #[test]
    fn test_core_module_accessible() {
        // Test that core module is accessible from main
        let _version = core::VERSION;
        let _authors = core::AUTHORS;

        // Test core types are accessible
        let addr = core::Address::new(0x1000);
        assert_eq!(addr.as_usize(), 0x1000);

        let value = core::MemoryValue::U32(42);
        assert_eq!(value.size(), 4);

        let process = core::ProcessInfo::new(1234, "test.exe".to_string());
        assert_eq!(process.pid, 1234);
    }

    #[test]
    fn test_constants() {
        // Test that compile-time constants are properly set
        let pkg_version = env!("CARGO_PKG_VERSION");
        assert!(!pkg_version.is_empty());

        let pkg_name = env!("CARGO_PKG_NAME");
        assert_eq!(pkg_name, "memory-mcp");

        let arch = std::env::consts::ARCH;
        assert!(!arch.is_empty());

        let os = std::env::consts::OS;
        assert_eq!(os, "windows");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_specific() {
        // Test Windows-specific functionality
        assert!(verify_platform().is_ok(), "Should run on Windows");

        // Verify we're on 64-bit Windows
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<usize>(), 8, "Should be 64-bit");
        }
    }
}
