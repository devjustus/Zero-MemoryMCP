//! Scan result and session types

use super::{Address, MemoryValue};
use serde::{Deserialize, Serialize};

/// Result from a memory scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub address: Address,
    pub value: MemoryValue,
    pub previous_value: Option<MemoryValue>,
    pub region_info: Option<RegionInfo>,
}

impl ScanResult {
    /// Creates a new scan result
    pub fn new(address: Address, value: MemoryValue) -> Self {
        ScanResult {
            address,
            value,
            previous_value: None,
            region_info: None,
        }
    }

    /// Creates a scan result with previous value for comparison
    pub fn with_previous(address: Address, value: MemoryValue, previous: MemoryValue) -> Self {
        ScanResult {
            address,
            value,
            previous_value: Some(previous),
            region_info: None,
        }
    }
}

/// Information about a memory region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub base_address: Address,
    pub size: usize,
    pub protection: u32,
    pub state: u32,
    pub region_type: u32,
}

/// Represents a scanning session with results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSession {
    pub id: String,
    pub scan_type: ScanType,
    pub value_type: super::value::ValueType,
    pub results: Vec<ScanResult>,
    pub scan_count: u32,
    pub created_at: u64,
    pub last_scan_at: u64,
}

impl ScanSession {
    /// Creates a new scan session
    pub fn new(id: String, scan_type: ScanType, value_type: super::value::ValueType) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ScanSession {
            id,
            scan_type,
            value_type,
            results: Vec::new(),
            scan_count: 0,
            created_at: now,
            last_scan_at: now,
        }
    }

    /// Adds results to the session
    pub fn add_results(&mut self, results: Vec<ScanResult>) {
        self.results = results;
        self.scan_count += 1;
        self.last_scan_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Filters results based on a predicate
    pub fn filter_results<F>(&mut self, predicate: F)
    where
        F: Fn(&ScanResult) -> bool,
    {
        self.results.retain(predicate);
    }
}

/// Type of memory scan to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Exact,
    Unknown,
    Increased,
    IncreasedBy,
    Decreased,
    DecreasedBy,
    Changed,
    Unchanged,
    Between,
    BiggerThan,
    SmallerThan,
}

impl ScanType {
    /// Checks if this scan type requires a previous value
    pub fn requires_previous(&self) -> bool {
        matches!(
            self,
            ScanType::Increased
                | ScanType::IncreasedBy
                | ScanType::Decreased
                | ScanType::DecreasedBy
                | ScanType::Changed
                | ScanType::Unchanged
        )
    }

    /// Checks if this scan type requires a value parameter
    pub fn requires_value(&self) -> bool {
        matches!(
            self,
            ScanType::Exact
                | ScanType::IncreasedBy
                | ScanType::DecreasedBy
                | ScanType::Between
                | ScanType::BiggerThan
                | ScanType::SmallerThan
        )
    }
}
