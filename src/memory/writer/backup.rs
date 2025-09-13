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

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_backup_before_write_with_auto_backup() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Test with auto_backup enabled (default)
        assert!(backup.config().auto_backup);
        let result = backup.backup_before_write(Address::new(0x1000), 100);
        // Will fail because we can't read from arbitrary address, but that's expected
        assert!(result.is_err() || backup.count() == 1);

        // Test with auto_backup disabled
        backup.set_auto_backup(false);
        assert!(!backup.config().auto_backup);
        backup.clear();
        let result = backup.backup_before_write(Address::new(0x2000), 100);
        assert!(result.is_ok());
        assert_eq!(backup.count(), 0); // Should not create backup when disabled
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_set_max_entries_and_trim() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Create mock entries to test trimming
        for i in 0..10 {
            let entry = BackupEntry::new(
                Address::new(0x1000 + i * 0x100),
                vec![i as u8; 10],
                handle.pid(),
                Some(format!("Backup {}", i)),
            );
            backup.entries.push_back(entry);
        }

        assert_eq!(backup.count(), 10);

        // Set max entries to 5, should trim oldest entries
        backup.set_max_entries(5);
        assert_eq!(backup.count(), 5);

        // Verify the oldest entries were removed
        let first_entry = backup.entries.front().unwrap();
        assert_eq!(first_entry.description, Some("Backup 5".to_string()));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_backup() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Add test entries
        let entry1 = BackupEntry::new(
            Address::new(0x1000),
            vec![1, 2, 3],
            handle.pid(),
            Some("Entry 1".to_string()),
        );
        let entry2 = BackupEntry::new(
            Address::new(0x2000),
            vec![4, 5, 6],
            handle.pid(),
            Some("Entry 2".to_string()),
        );
        let entry3 = BackupEntry::new(
            Address::new(0x1000), // Same address as entry1
            vec![7, 8, 9],
            handle.pid(),
            Some("Entry 3".to_string()),
        );

        backup.entries.push_back(entry1);
        backup.entries.push_back(entry2);
        backup.entries.push_back(entry3);

        // Should find the most recent backup for address 0x1000
        let found = backup.find_backup(Address::new(0x1000));
        assert!(found.is_some());
        assert_eq!(found.unwrap().description, Some("Entry 3".to_string()));

        // Should find backup for address 0x2000
        let found = backup.find_backup(Address::new(0x2000));
        assert!(found.is_some());
        assert_eq!(found.unwrap().description, Some("Entry 2".to_string()));

        // Should not find backup for non-existent address
        let found = backup.find_backup(Address::new(0x3000));
        assert!(found.is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_find_backup_for_range() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Add test entry with 256 bytes
        let entry = BackupEntry::new(
            Address::new(0x1000),
            vec![0; 256],
            handle.pid(),
            Some("Range entry".to_string()),
        );
        backup.entries.push_back(entry);

        // Should find backup containing the range
        let found = backup.find_backup_for_range(Address::new(0x1000), 100);
        assert!(found.is_some());

        let found = backup.find_backup_for_range(Address::new(0x1050), 50);
        assert!(found.is_some());

        // Should not find backup for out-of-range address
        let found = backup.find_backup_for_range(Address::new(0x2000), 100);
        assert!(found.is_none());

        // Should not find backup for range extending beyond backup
        let found = backup.find_backup_for_range(Address::new(0x1000), 300);
        assert!(found.is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_clear_backups() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Add some entries
        for i in 0..5 {
            let entry = BackupEntry::new(
                Address::new(0x1000 + i * 0x100),
                vec![i as u8; 10],
                handle.pid(),
                None,
            );
            backup.entries.push_back(entry);
        }

        assert_eq!(backup.count(), 5);
        assert!(backup.total_size() > 0);

        // Clear all entries
        backup.clear();
        assert_eq!(backup.count(), 0);
        assert_eq!(backup.total_size(), 0);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_restore_entry_wrong_process() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let backup = MemoryBackup::new(&handle);

        // Create entry for different process
        let entry = BackupEntry::new(
            Address::new(0x1000),
            vec![1, 2, 3],
            9999, // Different PID
            None,
        );

        let result = backup.restore_entry(&entry);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MemoryError::UnsupportedOperation(_)
        ));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_restore_last_empty() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let backup = MemoryBackup::new(&handle);

        let result = backup.restore_last();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MemoryError::SessionNotFound(_)
        ));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_restore_all_empty() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let backup = MemoryBackup::new(&handle);

        // Should succeed even with no entries
        let result = backup.restore_all();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_entries_getter() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Initially empty
        assert_eq!(backup.entries().len(), 0);

        // Add an entry
        let entry = BackupEntry::new(Address::new(0x1000), vec![1, 2, 3], handle.pid(), None);
        backup.entries.push_back(entry);

        // Check entries
        assert_eq!(backup.entries().len(), 1);
        let first = backup.entries().front().unwrap();
        assert_eq!(first.address, Address::new(0x1000));
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_config_mut() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Modify config through config_mut
        backup.config_mut().max_entries = 200;
        backup.config_mut().auto_backup = false;
        backup.config_mut().compress = true;

        // Verify changes
        assert_eq!(backup.config().max_entries, 200);
        assert!(!backup.config().auto_backup);
        assert!(backup.config().compress);
    }

    #[test]
    #[cfg_attr(miri, ignore = "FFI not supported in Miri")]
    fn test_total_size_calculation() {
        let handle = ProcessHandle::open_for_read_write(std::process::id()).unwrap();
        let mut backup = MemoryBackup::new(&handle);

        // Add entries with known sizes
        backup.entries.push_back(BackupEntry::new(
            Address::new(0x1000),
            vec![0; 100],
            handle.pid(),
            None,
        ));
        backup.entries.push_back(BackupEntry::new(
            Address::new(0x2000),
            vec![0; 200],
            handle.pid(),
            None,
        ));
        backup.entries.push_back(BackupEntry::new(
            Address::new(0x3000),
            vec![0; 300],
            handle.pid(),
            None,
        ));

        assert_eq!(backup.total_size(), 600);
    }

    // Additional tests that don't require FFI for better coverage

    #[test]
    fn test_backup_entry_size_method() {
        let entry = BackupEntry::new(
            Address::new(0x5000),
            vec![0; 512],
            9999,
            Some("Large backup".to_string()),
        );
        assert_eq!(entry.size(), 512);

        let empty_entry = BackupEntry::new(Address::new(0x6000), vec![], 9999, None);
        assert_eq!(empty_entry.size(), 0);
    }

    #[test]
    fn test_backup_entry_timestamp() {
        let before = SystemTime::now();
        let entry = BackupEntry::new(Address::new(0x7000), vec![1, 2, 3], 1111, None);
        let after = SystemTime::now();

        // Verify timestamp is within reasonable bounds
        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }

    #[test]
    fn test_backup_config_fields() {
        let mut config = BackupConfig {
            max_entries: 42,
            auto_backup: false,
            compress: true,
        };

        assert_eq!(config.max_entries, 42);
        assert!(!config.auto_backup);
        assert!(config.compress);

        // Modify fields
        config.max_entries = 100;
        config.auto_backup = true;
        config.compress = false;

        assert_eq!(config.max_entries, 100);
        assert!(config.auto_backup);
        assert!(!config.compress);
    }

    #[test]
    fn test_backup_entry_contains_range_edge_cases() {
        let entry = BackupEntry::new(Address::new(0x2000), vec![0; 100], 5555, None);

        // Exact match
        assert!(entry.contains_range(Address::new(0x2000), 100));

        // Start at beginning, partial size
        assert!(entry.contains_range(Address::new(0x2000), 50));

        // Start in middle, fits within
        assert!(entry.contains_range(Address::new(0x2010), 80));

        // Start before entry
        assert!(!entry.contains_range(Address::new(0x1FFF), 10));

        // Extends past end
        assert!(!entry.contains_range(Address::new(0x2050), 60));

        // Zero size
        assert!(entry.contains_range(Address::new(0x2000), 0));
        assert!(entry.contains_range(Address::new(0x2050), 0));
    }

    #[test]
    fn test_trim_entries_logic() {
        // Create a mock backup with direct access to entries
        let mut entries: VecDeque<BackupEntry> = VecDeque::new();

        // Add 10 entries
        for i in 0..10 {
            entries.push_back(BackupEntry::new(
                Address::new(0x1000 * (i + 1)),
                vec![i as u8; 10],
                1234,
                Some(format!("Entry {}", i)),
            ));
        }

        assert_eq!(entries.len(), 10);

        // Simulate trimming to max 5
        let max_entries = 5;
        while entries.len() > max_entries {
            entries.pop_front();
        }

        assert_eq!(entries.len(), 5);

        // Check that oldest were removed
        let first = entries.front().unwrap();
        assert_eq!(first.description, Some("Entry 5".to_string()));

        let last = entries.back().unwrap();
        assert_eq!(last.description, Some("Entry 9".to_string()));
    }

    #[test]
    fn test_backup_entry_all_fields() {
        let data = vec![0xFF, 0xEE, 0xDD, 0xCC];
        let entry = BackupEntry {
            address: Address::new(0xABCD),
            original_data: data.clone(),
            timestamp: SystemTime::UNIX_EPOCH,
            process_id: 4321,
            description: Some("Custom entry".to_string()),
        };

        assert_eq!(entry.address.as_usize(), 0xABCD);
        assert_eq!(entry.original_data, data);
        assert_eq!(entry.timestamp, SystemTime::UNIX_EPOCH);
        assert_eq!(entry.process_id, 4321);
        assert_eq!(entry.description, Some("Custom entry".to_string()));
        assert_eq!(entry.size(), 4);
    }

    #[test]
    fn test_backup_entry_no_description() {
        let entry = BackupEntry::new(Address::new(0x9000), vec![0xAA], 7777, None);

        assert_eq!(entry.description, None);
    }

    #[test]
    fn test_vecdeque_operations() {
        let mut entries: VecDeque<BackupEntry> = VecDeque::new();

        // Test push_back
        entries.push_back(BackupEntry::new(
            Address::new(0x100),
            vec![1],
            111,
            Some("First".to_string()),
        ));

        entries.push_back(BackupEntry::new(
            Address::new(0x200),
            vec![2],
            222,
            Some("Second".to_string()),
        ));

        assert_eq!(entries.len(), 2);

        // Test iteration
        let mut count = 0;
        for _ in entries.iter() {
            count += 1;
        }
        assert_eq!(count, 2);

        // Test reverse iteration
        let mut reverse_count = 0;
        for entry in entries.iter().rev() {
            reverse_count += 1;
            if reverse_count == 1 {
                assert_eq!(entry.description, Some("Second".to_string()));
            }
        }
        assert_eq!(reverse_count, 2);

        // Test pop_front
        let first = entries.pop_front();
        assert!(first.is_some());
        assert_eq!(first.unwrap().description, Some("First".to_string()));
        assert_eq!(entries.len(), 1);

        // Test clear
        entries.clear();
        assert_eq!(entries.len(), 0);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_backup_entry_large_data() {
        let large_data = vec![0xAB; 10000];
        let entry = BackupEntry::new(
            Address::new(0xF000),
            large_data.clone(),
            8888,
            Some("Large data test".to_string()),
        );

        assert_eq!(entry.original_data.len(), 10000);
        assert_eq!(entry.size(), 10000);
        assert_eq!(entry.original_data, large_data);
    }

    #[test]
    fn test_address_calculations() {
        let entry = BackupEntry::new(Address::new(0x4000), vec![0; 0x100], 3333, None);

        // Test address arithmetic for contains_range
        let start_addr = entry.address.as_usize();
        let end_addr = start_addr + entry.original_data.len();

        assert_eq!(start_addr, 0x4000);
        assert_eq!(end_addr, 0x4100);

        // Test range that starts at exact end (should not be contained)
        assert!(!entry.contains_range(Address::new(0x4100), 1));

        // Test range that ends at exact end (should be contained)
        assert!(entry.contains_range(Address::new(0x40FF), 1));
    }
}
