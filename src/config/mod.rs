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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_module_exports() {
        // Test that we can access all exported items
        let _config = default_config();
        let _loader = ConfigLoader::new("test.toml");
        let _validator = ConfigValidator;
        
        // Test ConfigResult type alias
        let result: ConfigResult<String> = Ok("test".to_string());
        assert!(result.is_ok());
        
        let error_result: ConfigResult<String> = Err(ConfigError::Invalid("test".to_string()));
        assert!(error_result.is_err());
    }

    #[test]
    fn test_validate_config_export() {
        let config = Config::default();
        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_config_export() {
        // This will return default config since file doesn't exist
        let result = load_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_error_from_io() {
        use std::io;
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let config_error: ConfigError = io_error.into();
        assert!(matches!(config_error, ConfigError::Io(_)));
    }

    #[test]
    fn test_config_result_type() {
        fn returns_config_result() -> ConfigResult<Config> {
            Ok(Config::default())
        }
        
        let result = returns_config_result();
        assert!(result.is_ok());
    }
}
