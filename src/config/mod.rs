//! Configuration module for Memory-MCP
//!
//! Provides configuration loading, validation, and default settings
//! for the Memory-MCP server.

mod defaults;
mod loader;
mod validator;

pub use defaults::{default_config, ConfigDefaults};
pub use loader::{load_config, ConfigLoader};
pub use validator::{validate_config, ConfigValidator};

// Re-export the main configuration structure
pub use loader::Config;

// Configuration-related error type
pub use loader::ConfigError;

// Configuration result type
pub type ConfigResult<T> = Result<T, ConfigError>;
