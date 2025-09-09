#![allow(dead_code)]
#![allow(unused_imports)]

mod core;

use anyhow::Result;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Memory-MCP server v{}", env!("CARGO_PKG_VERSION"));

    // Verify Windows platform
    #[cfg(not(target_os = "windows"))]
    {
        anyhow::bail!("Memory-MCP only supports Windows platform");
    }

    info!("Platform check: Windows ✓");
    info!("Architecture: {}", std::env::consts::ARCH);

    // TODO: Initialize MCP server
    info!("MCP server initialization pending implementation");

    // Placeholder for keeping server running
    info!("Memory-MCP ready. Press Ctrl+C to shutdown.");
    tokio::signal::ctrl_c().await?;

    info!("Shutting down Memory-MCP server");
    Ok(())
}
