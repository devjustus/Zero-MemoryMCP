//! Memory region filtering functionality

use crate::core::types::Address;
use crate::memory::regions::{RegionInfo, RegionState, RegionType};

/// Criteria for filtering memory regions
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    /// Filter by minimum size
    pub min_size: Option<usize>,
    /// Filter by maximum size
    pub max_size: Option<usize>,
    /// Filter by state
    pub state: Option<RegionState>,
    /// Filter by type
    pub region_type: Option<RegionType>,
    /// Filter by readable regions only
    pub readable_only: bool,
    /// Filter by writable regions only
    pub writable_only: bool,
    /// Filter by executable regions only
    pub executable_only: bool,
    /// Filter by address range
    pub address_range: Option<(Address, Address)>,
    /// Exclude guarded pages
    pub exclude_guarded: bool,
    /// Include only committed memory
    pub committed_only: bool,
}

impl FilterCriteria {
    /// Create a new filter criteria builder
    pub fn new() -> Self {
        FilterCriteria::default()
    }

    /// Set minimum size filter
    pub fn with_min_size(mut self, size: usize) -> Self {
        self.min_size = Some(size);
        self
    }

    /// Set maximum size filter
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Set state filter
    pub fn with_state(mut self, state: RegionState) -> Self {
        self.state = Some(state);
        self
    }

    /// Set type filter
    pub fn with_type(mut self, region_type: RegionType) -> Self {
        self.region_type = Some(region_type);
        self
    }

    /// Filter for readable regions only
    pub fn readable(mut self) -> Self {
        self.readable_only = true;
        self
    }

    /// Filter for writable regions only
    pub fn writable(mut self) -> Self {
        self.writable_only = true;
        self
    }

    /// Filter for executable regions only
    pub fn executable(mut self) -> Self {
        self.executable_only = true;
        self
    }

    /// Set address range filter
    pub fn with_address_range(mut self, start: Address, end: Address) -> Self {
        self.address_range = Some((start, end));
        self
    }

    /// Exclude guarded pages
    pub fn exclude_guarded_pages(mut self) -> Self {
        self.exclude_guarded = true;
        self
    }

    /// Include only committed memory
    pub fn committed_memory_only(mut self) -> Self {
        self.committed_only = true;
        self
    }
}

/// Filter for memory regions
pub struct RegionFilter {
    criteria: FilterCriteria,
}

impl RegionFilter {
    /// Create a new region filter with the given criteria
    pub fn new(criteria: FilterCriteria) -> Self {
        RegionFilter { criteria }
    }

    /// Apply the filter to a list of regions
    pub fn apply(&self, regions: &[RegionInfo]) -> Vec<RegionInfo> {
        regions
            .iter()
            .filter(|region| self.matches(region))
            .cloned()
            .collect()
    }

    /// Check if a region matches the filter criteria
    pub fn matches(&self, region: &RegionInfo) -> bool {
        // Check size criteria
        if let Some(min) = self.criteria.min_size {
            if region.size < min {
                return false;
            }
        }
        
        if let Some(max) = self.criteria.max_size {
            if region.size > max {
                return false;
            }
        }

        // Check state
        if let Some(state) = self.criteria.state {
            if region.state != state {
                return false;
            }
        }

        // Check type
        if let Some(region_type) = self.criteria.region_type {
            if region.region_type != region_type {
                return false;
            }
        }

        // Check permissions
        if self.criteria.readable_only && !region.is_readable() {
            return false;
        }

        if self.criteria.writable_only && !region.is_writable() {
            return false;
        }

        if self.criteria.executable_only && !region.is_executable() {
            return false;
        }

        // Check address range
        if let Some((start, end)) = self.criteria.address_range {
            if region.base_address < start || region.end_address() > end {
                return false;
            }
        }

        // Check guarded pages
        if self.criteria.exclude_guarded && region.is_guarded() {
            return false;
        }

        // Check committed only
        if self.criteria.committed_only && region.state != RegionState::Committed {
            return false;
        }

        true
    }

    /// Count regions matching the filter
    pub fn count(&self, regions: &[RegionInfo]) -> usize {
        regions.iter().filter(|region| self.matches(region)).count()
    }

    /// Get total size of regions matching the filter
    pub fn total_size(&self, regions: &[RegionInfo]) -> usize {
        regions
            .iter()
            .filter(|region| self.matches(region))
            .map(|region| region.size)
            .sum()
    }
}

/// Common filter presets
pub mod presets {
    use super::*;

    /// Get filter for executable code regions
    pub fn executable_code() -> FilterCriteria {
        FilterCriteria::new()
            .executable()
            .with_state(RegionState::Committed)
            .exclude_guarded_pages()
    }

    /// Get filter for heap regions
    pub fn heap_regions() -> FilterCriteria {
        FilterCriteria::new()
            .with_type(RegionType::Private)
            .writable()
            .with_state(RegionState::Committed)
            .exclude_guarded_pages()
    }

    /// Get filter for stack regions
    pub fn stack_regions() -> FilterCriteria {
        FilterCriteria::new()
            .with_type(RegionType::Private)
            .readable()
            .writable()
            .with_state(RegionState::Committed)
    }

    /// Get filter for image (DLL/EXE) regions
    pub fn image_regions() -> FilterCriteria {
        FilterCriteria::new()
            .with_type(RegionType::Image)
            .with_state(RegionState::Committed)
    }

    /// Get filter for large memory regions (> 1MB)
    pub fn large_regions() -> FilterCriteria {
        FilterCriteria::new()
            .with_min_size(1024 * 1024)
            .with_state(RegionState::Committed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_criteria_builder() {
        let criteria = FilterCriteria::new()
            .with_min_size(4096)
            .with_max_size(1024 * 1024)
            .readable()
            .writable()
            .exclude_guarded_pages();

        assert_eq!(criteria.min_size, Some(4096));
        assert_eq!(criteria.max_size, Some(1024 * 1024));
        assert!(criteria.readable_only);
        assert!(criteria.writable_only);
        assert!(criteria.exclude_guarded);
    }

    #[test]
    fn test_region_filter_matching() {
        let region = RegionInfo {
            base_address: Address::new(0x1000),
            size: 8192,
            state: RegionState::Committed,
            region_type: RegionType::Private,
            protection: 0x04, // PAGE_READWRITE
            allocation_protection: 0x04,
            allocation_base: Address::new(0x1000),
        };

        let filter = RegionFilter::new(
            FilterCriteria::new()
                .with_min_size(4096)
                .readable()
                .with_state(RegionState::Committed)
        );

        assert!(filter.matches(&region));

        let filter2 = RegionFilter::new(
            FilterCriteria::new()
                .executable()
        );

        assert!(!filter2.matches(&region));
    }

    #[test]
    fn test_filter_presets() {
        let exec_filter = presets::executable_code();
        assert!(exec_filter.executable_only);
        assert_eq!(exec_filter.state, Some(RegionState::Committed));
        assert!(exec_filter.exclude_guarded);

        let heap_filter = presets::heap_regions();
        assert!(heap_filter.writable_only);
        assert_eq!(heap_filter.region_type, Some(RegionType::Private));

        let image_filter = presets::image_regions();
        assert_eq!(image_filter.region_type, Some(RegionType::Image));
    }
}