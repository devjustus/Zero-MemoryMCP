//! Memory region management for Windows processes
//!
//! This module provides functionality for enumerating, filtering, and managing
//! memory regions within a Windows process. It supports querying region properties,
//! modifying protection flags, and mapping memory regions.

pub mod enumerator;
pub mod filter;
pub mod mapper;
pub mod protection;

pub use enumerator::{enumerate_regions, query_region_at, RegionEnumerator, RegionInfo};
pub use filter::{FilterCriteria, RegionFilter};
pub use mapper::{MappedRegion, MappingOptions, MemoryMapper};
pub use protection::{change_protection, ProtectionFlags, ProtectionManager};

use crate::core::types::{Address, MemoryResult};

/// State of a memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionState {
    /// Memory is committed and accessible
    Committed,
    /// Memory is reserved but not committed
    Reserved,
    /// Memory is free/unallocated
    Free,
}

/// Type of memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    /// Private memory
    Private,
    /// Mapped memory (file mapping)
    Mapped,
    /// Image memory (executable/DLL)
    Image,
}

/// Get information about a specific memory region
pub fn query_region(address: Address) -> MemoryResult<RegionInfo> {
    enumerator::query_region_at(address)
}

/// Get all memory regions for the current process
pub fn get_all_regions() -> MemoryResult<Vec<RegionInfo>> {
    enumerate_regions()
}

/// Get memory regions matching specific criteria
pub fn get_filtered_regions(criteria: FilterCriteria) -> MemoryResult<Vec<RegionInfo>> {
    let regions = enumerate_regions()?;
    let filter = RegionFilter::new(criteria);
    Ok(filter.apply(&regions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_state_equality() {
        assert_eq!(RegionState::Committed, RegionState::Committed);
        assert_ne!(RegionState::Committed, RegionState::Reserved);
        assert_ne!(RegionState::Reserved, RegionState::Free);
    }

    #[test]
    fn test_region_type_equality() {
        assert_eq!(RegionType::Private, RegionType::Private);
        assert_ne!(RegionType::Private, RegionType::Mapped);
        assert_ne!(RegionType::Mapped, RegionType::Image);
    }
}
