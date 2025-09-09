//! Configuration validator for Memory-MCP
//!
//! Validates configuration values to ensure they are within acceptable ranges.

use super::loader::{Config, ConfigError};

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validates the entire configuration
    pub fn validate(config: &Config) -> Result<(), ConfigError> {
        Self::validate_server(&config.server)?;
        Self::validate_scanner(&config.scanner)?;
        Self::validate_memory(&config.memory)?;
        Self::validate_logging(&config.logging)?;
        Ok(())
    }

    /// Validates server configuration
    fn validate_server(server: &super::loader::ServerConfig) -> Result<(), ConfigError> {
        // Validate port range
        if server.port == 0 {
            return Err(ConfigError::Invalid("Server port cannot be 0".to_string()));
        }

        // Validate max connections
        if server.max_connections == 0 {
            return Err(ConfigError::Invalid(
                "Maximum connections must be at least 1".to_string(),
            ));
        }

        if server.max_connections > 1000 {
            return Err(ConfigError::Invalid(
                "Maximum connections cannot exceed 1000".to_string(),
            ));
        }

        // Validate host format (basic check)
        if server.host.is_empty() {
            return Err(ConfigError::Invalid(
                "Server host cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Validates scanner configuration
    fn validate_scanner(scanner: &super::loader::ScannerConfig) -> Result<(), ConfigError> {
        // Validate thread count
        if scanner.max_threads == 0 {
            return Err(ConfigError::Invalid(
                "Scanner threads must be at least 1".to_string(),
            ));
        }

        if scanner.max_threads > 128 {
            return Err(ConfigError::Invalid(
                "Scanner threads cannot exceed 128".to_string(),
            ));
        }

        // Validate chunk size (must be power of 2 for alignment)
        if scanner.chunk_size == 0 || !scanner.chunk_size.is_power_of_two() {
            return Err(ConfigError::Invalid(
                "Chunk size must be a power of 2".to_string(),
            ));
        }

        // Validate cache size
        if scanner.cache_size < scanner.chunk_size {
            return Err(ConfigError::Invalid(
                "Cache size must be at least as large as chunk size".to_string(),
            ));
        }

        Ok(())
    }

    /// Validates memory configuration
    fn validate_memory(memory: &super::loader::MemoryConfig) -> Result<(), ConfigError> {
        // Validate max read size
        if memory.max_read_size == 0 {
            return Err(ConfigError::Invalid(
                "Maximum read size must be greater than 0".to_string(),
            ));
        }

        // Warn if read size is very large (>100MB)
        if memory.max_read_size > 104857600 {
            // This is just a warning in production, but we validate it
            eprintln!("Warning: Maximum read size exceeds 100MB");
        }

        Ok(())
    }

    /// Validates logging configuration
    fn validate_logging(logging: &super::loader::LoggingConfig) -> Result<(), ConfigError> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error", "off"];
        if !valid_levels.contains(&logging.level.to_lowercase().as_str()) {
            return Err(ConfigError::Invalid(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                logging.level, valid_levels
            )));
        }

        // Validate log file path
        if logging.file.is_empty() {
            return Err(ConfigError::Invalid(
                "Log file path cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Validates a configuration
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    ConfigValidator::validate(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config = Config::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let mut config = Config::default();
        config.server.port = 0;
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_invalid_max_connections() {
        let mut config = Config::default();
        config.server.max_connections = 0;
        let result = validate_config(&config);
        assert!(result.is_err());

        config.server.max_connections = 1001;
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_thread_count() {
        let mut config = Config::default();
        config.scanner.max_threads = 0;
        let result = validate_config(&config);
        assert!(result.is_err());

        config.scanner.max_threads = 129;
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_chunk_size() {
        let mut config = Config::default();
        config.scanner.chunk_size = 0;
        let result = validate_config(&config);
        assert!(result.is_err());

        config.scanner.chunk_size = 1000; // Not power of 2
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = Config::default();
        config.logging.level = "invalid".to_string();
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("log level"));
    }

    #[test]
    fn test_edge_cases() {
        let mut config = Config::default();

        // Test minimum valid values
        config.server.port = 1;
        config.server.max_connections = 1;
        config.scanner.max_threads = 1;
        config.scanner.chunk_size = 1024; // Power of 2
        config.scanner.cache_size = 1024;
        config.memory.max_read_size = 1;

        assert!(validate_config(&config).is_ok());

        // Test maximum valid values
        config.server.max_connections = 1000;
        config.scanner.max_threads = 128;
        config.memory.max_read_size = 104857600; // 100MB

        assert!(validate_config(&config).is_ok());
    }
}
