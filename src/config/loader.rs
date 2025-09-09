//! Configuration loader for Memory-MCP
//!
//! Handles loading configuration from TOML files and merging with defaults.

use super::defaults::{default_config, ConfigDefaults};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,

    #[serde(default = "default_scanner")]
    pub scanner: ScannerConfig,

    #[serde(default = "default_memory")]
    pub memory: MemoryConfig,

    #[serde(default = "default_logging")]
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    #[serde(default = "default_max_threads")]
    pub max_threads: usize,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_max_read_size")]
    pub max_read_size: usize,
    #[serde(default = "default_enable_write_protection")]
    pub enable_write_protection: bool,
    #[serde(default = "default_backup_before_write")]
    pub backup_before_write: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_file")]
    pub file: String,
}

/// Configuration loader
pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    /// Creates a new configuration loader
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ConfigLoader {
            config_path: path.as_ref().to_path_buf(),
        }
    }

    /// Loads configuration from file
    pub fn load(&self) -> Result<Config, ConfigError> {
        if !self.config_path.exists() {
            return Err(ConfigError::FileNotFound(
                self.config_path.display().to_string(),
            ));
        }

        let contents = fs::read_to_string(&self.config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Loads configuration or returns defaults if file doesn't exist
    pub fn load_or_default(&self) -> Config {
        self.load().unwrap_or_else(|_| Config::default())
    }

    /// Saves configuration to file
    pub fn save(&self, config: &Config) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, contents)?;
        Ok(())
    }
}

/// Loads configuration from the default location
pub fn load_config() -> Result<Config, ConfigError> {
    let loader = ConfigLoader::new("config.toml");
    loader.load_or_default().into()
}

// Default functions for serde
fn default_server() -> ServerConfig {
    let defaults = default_config();
    ServerConfig {
        host: defaults.server.host,
        port: defaults.server.port,
        max_connections: defaults.server.max_connections,
    }
}

fn default_scanner() -> ScannerConfig {
    let defaults = default_config();
    ScannerConfig {
        max_threads: defaults.scanner.max_threads,
        chunk_size: defaults.scanner.chunk_size,
        cache_size: defaults.scanner.cache_size,
    }
}

fn default_memory() -> MemoryConfig {
    let defaults = default_config();
    MemoryConfig {
        max_read_size: defaults.memory.max_read_size,
        enable_write_protection: defaults.memory.enable_write_protection,
        backup_before_write: defaults.memory.backup_before_write,
    }
}

fn default_logging() -> LoggingConfig {
    let defaults = default_config();
    LoggingConfig {
        level: defaults.logging.level,
        file: defaults.logging.file,
    }
}

// Individual field defaults
fn default_host() -> String {
    default_config().server.host
}

fn default_port() -> u16 {
    default_config().server.port
}

fn default_max_connections() -> usize {
    default_config().server.max_connections
}

fn default_max_threads() -> usize {
    default_config().scanner.max_threads
}

fn default_chunk_size() -> usize {
    default_config().scanner.chunk_size
}

fn default_cache_size() -> usize {
    default_config().scanner.cache_size
}

fn default_max_read_size() -> usize {
    default_config().memory.max_read_size
}

fn default_enable_write_protection() -> bool {
    default_config().memory.enable_write_protection
}

fn default_backup_before_write() -> bool {
    default_config().memory.backup_before_write
}

fn default_log_level() -> String {
    default_config().logging.level
}

fn default_log_file() -> String {
    default_config().logging.file
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: default_server(),
            scanner: default_scanner(),
            memory: default_memory(),
            logging: default_logging(),
        }
    }
}

impl From<Config> for Result<Config, ConfigError> {
    fn from(config: Config) -> Self {
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert!(config.scanner.max_threads > 0);
    }

    #[test]
    fn test_load_missing_file() {
        let loader = ConfigLoader::new("nonexistent.toml");
        let result = loader.load();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::FileNotFound(_)));
    }

    #[test]
    fn test_load_or_default() {
        let loader = ConfigLoader::new("nonexistent.toml");
        let config = loader.load_or_default();
        assert_eq!(config.server.host, "127.0.0.1");
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        let config = Config::default();
        let loader = ConfigLoader::new(&config_path);

        loader.save(&config).unwrap();
        assert!(config_path.exists());

        let loaded = loader.load().unwrap();
        assert_eq!(loaded.server.host, config.server.host);
        assert_eq!(loaded.server.port, config.server.port);
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
            [server]
            host = "0.0.0.0"
            port = 8080
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        // Check defaults are applied
        assert!(config.scanner.max_threads > 0);
        assert_eq!(config.memory.max_read_size, 10485760);
    }
}
