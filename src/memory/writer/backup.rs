//! Memory backup system for safe write operations
//!
//! This module provides automatic and manual backup functionality for memory writes,
//! allowing rollback of changes and preventing corruption.

use crate::core::types::{Address, MemoryError, MemoryResult};
use crate::memory::reader::BasicMemoryReader;
use crate::memory::writer::{BasicMemoryWriter, MemoryWrite};
use crate::process::ProcessHandle;
use std::collections::VecDeque;
use std::time::SystemTime;

/// Maximum number of backup entries to keep by default
const DEFAULT_MAX_ENTRIES: usize = 100;

/// Entry representing a single memory backup
#[derive(Debug, Clone)]
pub struct BackupEntry {
    /// Address where the backup was taken
    pub address: Address,
    /// Original data before modification
    pub original_data: Vec<u8>,
    /// Time when backup was created
    pub timestamp: SystemTime,
    /// Process ID this backup belongs to
    pub process_id: u32,
    /// Optional description for this backup
    pub description: Option<String>,
}

impl BackupEntry {
    /// Create a new backup entry
    pub fn new(
        address: Address,
        original_data: Vec<u8>,
        process_id: u32,
        description: Option<String>,
    ) -> Self {
        BackupEntry {
            address,
            original_data,
            timestamp: SystemTime::now(),
            process_id,
            description,
        }
    }

    /// Get the size of backed up data
    pub fn size(&self) -> usize {
        self.original_data.len()
    }

    /// Check if this backup is for a specific address range
    pub fn contains_range(&self, address: Address, size: usize) -> bool {
        let backup_start = self.address.as_usize();
        let backup_end = backup_start + self.original_data.len();
        let range_start = address.as_usize();
        let range_end = range_start + size;

        range_start >= backup_start && range_end <= backup_end
    }
}

/// Configuration for the backup system
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Maximum number of entries to keep
    pub max_entries: usize,
    /// Whether to automatically backup before writes
    pub auto_backup: bool,
    /// Whether to compress backup data
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        BackupConfig {
            max_entries: DEFAULT_MAX_ENTRIES,
            auto_backup: true,
            compress: false,
        }
    }
}

/// Memory backup system for managing write operation backups
pub struct MemoryBackup<'a> {
    /// Stored backup entries
    entries: VecDeque<BackupEntry>,
    /// Configuration
    config: BackupConfig,
    /// Process handle for operations
    handle: &'a ProcessHandle,
}

impl<'a> MemoryBackup<'a> {
    /// Create a new memory backup system
    pub fn new(handle: &'a ProcessHandle) -> Self {
        MemoryBackup {
            entries: VecDeque::new(),
            config: BackupConfig::default(),
            handle,
        }
    }

    /// Create with custom configuration
    pub fn with_config(handle: &'a ProcessHandle, config: BackupConfig) -> Self {
        MemoryBackup {
            entries: VecDeque::with_capacity(config.max_entries),
            config,
            handle,
        }
    }

    /// Set maximum number of backup entries
    pub fn set_max_entries(&mut self, max: usize) {
        self.config.max_entries = max;
        self.trim_entries();
    }

    /// Enable or disable automatic backup
    pub fn set_auto_backup(&mut self, enabled: bool) {
        self.config.auto_backup = enabled;
    }

    /// Create a backup of memory region
    pub fn backup_region(
        &mut self,
        address: Address,
        size: usize,
        description: Option<String>,
    ) -> MemoryResult<()> {
        if size == 0 {
            return Err(MemoryError::InvalidValueType(
                "Backup size cannot be zero".to_string(),
            ));
        }

        // Read current memory content
        let reader = BasicMemoryReader::new(self.handle);
        let original_data = reader.read_raw(address, size)?;

        // Create backup entry
        let entry = BackupEntry::new(address, original_data, self.handle.pid(), description);

        // Add to entries
        self.entries.push_back(entry);

        // Trim if needed
        self.trim_entries();

        Ok(())
    }

    /// Backup before a write operation if auto-backup is enabled
    pub fn backup_before_write(&mut self, address: Address, size: usize) -> MemoryResult<()> {
        if self.config.auto_backup {
            self.backup_region(address, size, Some("Auto-backup before write".to_string()))
        } else {
            Ok(())
        }
    }

    /// Restore a specific backup entry
    pub fn restore_entry(&self, entry: &BackupEntry) -> MemoryResult<()> {
        // Verify process ID matches
        if entry.process_id != self.handle.pid() {
            return Err(MemoryError::UnsupportedOperation(
                "Backup entry is for a different process".to_string(),
            ));
        }

        // Write original data back
        let writer = BasicMemoryWriter::new(self.handle);
        writer.write_bytes(entry.address, &entry.original_data)?;

        Ok(())
    }

    /// Restore the most recent backup
    pub fn restore_last(&self) -> MemoryResult<()> {
        match self.entries.back() {
            Some(entry) => self.restore_entry(entry),
            None => Err(MemoryError::SessionNotFound(
                "No backups available".to_string(),
            )),
        }
    }

    /// Restore all backups in reverse order
    pub fn restore_all(&self) -> MemoryResult<()> {
        // Restore in reverse order (newest first)
        for entry in self.entries.iter().rev() {
            self.restore_entry(entry)?;
        }
        Ok(())
    }

    /// Find backup for specific address
    pub fn find_backup(&self, address: Address) -> Option<&BackupEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.address == address)
    }

    /// Find backup containing address range
    pub fn find_backup_for_range(&self, address: Address, size: usize) -> Option<&BackupEntry> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.contains_range(address, size))
    }

    /// Clear all backup entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get number of backup entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Get total size of all backups
    pub fn total_size(&self) -> usize {
        self.entries.iter().map(|e| e.size()).sum()
    }

    /// Get all backup entries
    pub fn entries(&self) -> &VecDeque<BackupEntry> {
        &self.entries
    }

    /// Remove old entries if over limit
    fn trim_entries(&mut self) {
        while self.entries.len() > self.config.max_entries {
            self.entries.pop_front();
        }
    }

    /// Get configuration
    pub fn config(&self) -> &BackupConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut BackupConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessHandle;

    #[test]
    fn test_backup_entry_creation() {
        let entry = BackupEntry::new(
            Address::new(0x1000),
            vec![0x41, 0x42, 0x43],
            1234,
            Some("Test backup".to_string()),
        );

        assert_eq!(entry.address, Address::new(0x1000));
        assert_eq!(entry.original_data, vec![0x41, 0x42, 0x43]);
        assert_eq!(entry.process_id, 1234);
        assert_eq!(entry.description, Some("Test backup".to_string()));
        assert_eq!(entry.size(), 3);
    }

    #[test]
    fn test_backup_entry_contains_range() {
        let entry = BackupEntry::new(Address::new(0x1000), vec![0; 256], 1234, None);

        // Test contained ranges
        assert!(entry.contains_range(Address::new(0x1000), 256));
        assert!(entry.contains_range(Address::new(0x1000), 128));
        assert!(entry.contains_range(Address::new(0x1080), 128));

        // Test out of range
        assert!(!entry.contains_range(Address::new(0x0FFF), 2));
        assert!(!entry.contains_range(Address::new(0x1000), 257));
        assert!(!entry.contains_range(Address::new(0x1100), 1));
    }

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert_eq!(config.max_entries, DEFAULT_MAX_ENTRIES);
        assert!(config.auto_backup);
        assert!(!config.compress);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_backup_creation() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let backup = MemoryBackup::new(&handle);

        assert_eq!(backup.count(), 0);
        assert_eq!(backup.total_size(), 0);
        assert!(backup.config().auto_backup);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_memory_backup_with_config() {
        let config = BackupConfig {
            max_entries: 50,
            auto_backup: false,
            compress: true,
        };

        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let backup = MemoryBackup::with_config(&handle, config);

        assert_eq!(backup.config().max_entries, 50);
        assert!(!backup.config().auto_backup);
        assert!(backup.config().compress);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_backup_region_zero_size() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        let result = backup.backup_region(Address::new(0x1000), 0, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MemoryError::InvalidValueType(_)
        ));
    }

    #[test]
    fn test_backup_entry_debug() {
        let entry = BackupEntry::new(
            Address::new(0x1000),
            vec![0x41],
            1234,
            Some("Debug test".to_string()),
        );

        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("BackupEntry"));
        assert!(debug_str.contains("address"));
        assert!(debug_str.contains("process_id"));
    }

    #[test]
    fn test_backup_config_clone() {
        let config = BackupConfig {
            max_entries: 200,
            auto_backup: false,
            compress: true,
        };

        let cloned = config.clone();
        assert_eq!(cloned.max_entries, 200);
        assert!(!cloned.auto_backup);
        assert!(cloned.compress);
    }

    #[test]
    fn test_backup_entry_clone() {
        let entry = BackupEntry::new(Address::new(0x2000), vec![1, 2, 3], 5678, None);

        let cloned = entry.clone();
        assert_eq!(cloned.address, Address::new(0x2000));
        assert_eq!(cloned.original_data, vec![1, 2, 3]);
        assert_eq!(cloned.process_id, 5678);
    }
}
