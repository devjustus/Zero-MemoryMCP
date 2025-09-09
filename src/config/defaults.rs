//! Default configuration values for Memory-MCP

use serde::{Deserialize, Serialize};

/// Default configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDefaults {
    pub server: ServerDefaults,
    pub scanner: ScannerDefaults,
    pub memory: MemoryDefaults,
    pub logging: LoggingDefaults,
}

/// Default server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDefaults {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

/// Default scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerDefaults {
    pub max_threads: usize,
    pub chunk_size: usize,
    pub cache_size: usize,
}

/// Default memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDefaults {
    pub max_read_size: usize,
    pub enable_write_protection: bool,
    pub backup_before_write: bool,
}

/// Default logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingDefaults {
    pub level: String,
    pub file: String,
}

/// Returns the default configuration
pub fn default_config() -> ConfigDefaults {
    ConfigDefaults {
        server: ServerDefaults {
            host: "127.0.0.1".to_string(),
            port: 3000,
            max_connections: 10,
        },
        scanner: ScannerDefaults {
            max_threads: num_cpus::get().min(8),
            chunk_size: 65536,   // 64KB
            cache_size: 1048576, // 1MB
        },
        memory: MemoryDefaults {
            max_read_size: 10485760, // 10MB
            enable_write_protection: true,
            backup_before_write: true,
        },
        logging: LoggingDefaults {
            level: "info".to_string(),
            file: "memory-mcp.log".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.max_connections, 10);
    }

    #[test]
    fn test_scanner_defaults() {
        let config = default_config();
        assert!(config.scanner.max_threads > 0);
        assert!(config.scanner.max_threads <= 8);
        assert_eq!(config.scanner.chunk_size, 65536);
        assert_eq!(config.scanner.cache_size, 1048576);
    }

    #[test]
    fn test_memory_defaults() {
        let config = default_config();
        assert_eq!(config.memory.max_read_size, 10485760);
        assert!(config.memory.enable_write_protection);
        assert!(config.memory.backup_before_write);
    }

    #[test]
    fn test_logging_defaults() {
        let config = default_config();
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.file, "memory-mcp.log");
    }

    #[test]
    fn test_serialization() {
        let config = default_config();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("host"));
        assert!(serialized.contains("port"));

        let deserialized: ConfigDefaults = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.server.host, config.server.host);
        assert_eq!(deserialized.server.port, config.server.port);
    }
}
