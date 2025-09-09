//! Scan result and session types

use super::{Address, MemoryValue};
use serde::{Deserialize, Serialize};

/// Result from a memory scan operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionInfo {
    pub base_address: Address,
    pub size: usize,
    pub protection: u32,
    pub state: u32,
    pub region_type: u32,
}

/// Represents a scanning session with results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::value::ValueType;

    #[test]
    fn test_scan_result_creation() {
        let result = ScanResult::new(Address::new(0x1000), MemoryValue::U32(42));
        assert_eq!(result.address, Address::new(0x1000));
        assert!(matches!(result.value, MemoryValue::U32(42)));
        assert_eq!(result.previous_value, None);
        assert_eq!(result.region_info, None);
    }

    #[test]
    fn test_scan_result_with_previous() {
        let result = ScanResult::with_previous(
            Address::new(0x2000),
            MemoryValue::I32(100),
            MemoryValue::I32(50),
        );
        assert_eq!(result.address, Address::new(0x2000));
        assert!(matches!(result.value, MemoryValue::I32(100)));
        assert!(matches!(result.previous_value, Some(MemoryValue::I32(50))));
        assert_eq!(result.region_info, None);
    }

    #[test]
    fn test_scan_result_with_region_info() {
        let mut result =
            ScanResult::new(Address::new(0x3000), MemoryValue::F32(std::f32::consts::PI));
        let region = RegionInfo {
            base_address: Address::new(0x3000),
            size: 0x1000,
            protection: 0x20,
            state: 0x1000,
            region_type: 0x20000,
        };
        result.region_info = Some(region.clone());

        assert!(result.region_info.is_some());
        let info = result.region_info.unwrap();
        assert_eq!(info.base_address, Address::new(0x3000));
        assert_eq!(info.size, 0x1000);
        assert_eq!(info.protection, 0x20);
        assert_eq!(info.state, 0x1000);
        assert_eq!(info.region_type, 0x20000);
    }

    #[test]
    fn test_scan_session_creation() {
        let session = ScanSession::new("test-session".to_string(), ScanType::Exact, ValueType::U32);

        assert_eq!(session.id, "test-session");
        assert_eq!(session.scan_type, ScanType::Exact);
        assert_eq!(session.value_type, ValueType::U32);
        assert!(session.results.is_empty());
        assert_eq!(session.scan_count, 0);
        assert!(session.created_at > 0);
        assert_eq!(session.created_at, session.last_scan_at);
    }

    #[test]
    fn test_scan_session_add_results() {
        let mut session =
            ScanSession::new("session-2".to_string(), ScanType::Unknown, ValueType::I64);

        let results = vec![
            ScanResult::new(Address::new(0x1000), MemoryValue::I64(100)),
            ScanResult::new(Address::new(0x2000), MemoryValue::I64(200)),
            ScanResult::new(Address::new(0x3000), MemoryValue::I64(300)),
        ];

        let initial_time = session.last_scan_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.add_results(results);

        assert_eq!(session.results.len(), 3);
        assert_eq!(session.scan_count, 1);
        assert!(session.last_scan_at >= initial_time);
    }

    #[test]
    fn test_scan_session_filter_results() {
        let mut session = ScanSession::new(
            "filter-session".to_string(),
            ScanType::Between,
            ValueType::U32,
        );

        let results = vec![
            ScanResult::new(Address::new(0x1000), MemoryValue::U32(10)),
            ScanResult::new(Address::new(0x2000), MemoryValue::U32(50)),
            ScanResult::new(Address::new(0x3000), MemoryValue::U32(100)),
            ScanResult::new(Address::new(0x4000), MemoryValue::U32(25)),
        ];

        session.add_results(results);
        assert_eq!(session.results.len(), 4);

        session.filter_results(|result| {
            if let MemoryValue::U32(val) = result.value {
                val >= 50
            } else {
                false
            }
        });

        assert_eq!(session.results.len(), 2);
    }

    #[test]
    fn test_scan_type_requires_previous() {
        assert!(ScanType::Increased.requires_previous());
        assert!(ScanType::IncreasedBy.requires_previous());
        assert!(ScanType::Decreased.requires_previous());
        assert!(ScanType::DecreasedBy.requires_previous());
        assert!(ScanType::Changed.requires_previous());
        assert!(ScanType::Unchanged.requires_previous());

        assert!(!ScanType::Exact.requires_previous());
        assert!(!ScanType::Unknown.requires_previous());
        assert!(!ScanType::Between.requires_previous());
        assert!(!ScanType::BiggerThan.requires_previous());
        assert!(!ScanType::SmallerThan.requires_previous());
    }

    #[test]
    fn test_scan_type_requires_value() {
        assert!(ScanType::Exact.requires_value());
        assert!(ScanType::IncreasedBy.requires_value());
        assert!(ScanType::DecreasedBy.requires_value());
        assert!(ScanType::Between.requires_value());
        assert!(ScanType::BiggerThan.requires_value());
        assert!(ScanType::SmallerThan.requires_value());

        assert!(!ScanType::Unknown.requires_value());
        assert!(!ScanType::Increased.requires_value());
        assert!(!ScanType::Decreased.requires_value());
        assert!(!ScanType::Changed.requires_value());
        assert!(!ScanType::Unchanged.requires_value());
    }

    #[test]
    fn test_serialization() {
        let result = ScanResult::new(Address::new(0x5000), MemoryValue::U8(255));
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ScanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.address, deserialized.address);

        let session = ScanSession::new("json-session".to_string(), ScanType::Exact, ValueType::U16);
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: ScanSession = serde_json::from_str(&json).unwrap();
        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.scan_type, deserialized.scan_type);

        let scan_type = ScanType::Between;
        let json = serde_json::to_string(&scan_type).unwrap();
        assert_eq!(json, "\"between\"");
        let deserialized: ScanType = serde_json::from_str(&json).unwrap();
        assert_eq!(scan_type, deserialized);
    }

    #[test]
    fn test_clone_and_debug() {
        let result = ScanResult::new(Address::new(0x6000), MemoryValue::F64(std::f64::consts::E));
        let cloned = result.clone();
        assert_eq!(result.address, cloned.address);

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("ScanResult"));
        assert!(
            debug_str.contains("Address")
                || debug_str.contains("6000")
                || debug_str.contains("0x6000")
        );

        let session = ScanSession::new(
            "debug-session".to_string(),
            ScanType::Unknown,
            ValueType::String,
        );
        let cloned = session.clone();
        assert_eq!(session.id, cloned.id);

        let scan_type = ScanType::Increased;
        let cloned = scan_type.clone();
        assert_eq!(scan_type, cloned);

        let region = RegionInfo {
            base_address: Address::new(0x7000),
            size: 0x2000,
            protection: 0x40,
            state: 0x2000,
            region_type: 0x40000,
        };
        let cloned = region.clone();
        assert_eq!(region.base_address, cloned.base_address);
    }

    #[test]
    fn test_scan_type_all_variants() {
        let types = vec![
            ScanType::Exact,
            ScanType::Unknown,
            ScanType::Increased,
            ScanType::IncreasedBy,
            ScanType::Decreased,
            ScanType::DecreasedBy,
            ScanType::Changed,
            ScanType::Unchanged,
            ScanType::Between,
            ScanType::BiggerThan,
            ScanType::SmallerThan,
        ];

        for scan_type in types {
            let json = serde_json::to_string(&scan_type).unwrap();
            let deserialized: ScanType = serde_json::from_str(&json).unwrap();
            assert_eq!(scan_type, deserialized);
        }
    }
}
