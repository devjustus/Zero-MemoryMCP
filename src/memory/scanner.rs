//! Memory scanning functionality for pattern matching

use crate::core::types::{Address, MemoryError, MemoryResult};
use crate::process::ProcessHandle;
use crate::windows::bindings::kernel32;
use rayon::prelude::*;
use std::collections::HashMap;

/// Pattern for memory scanning
#[derive(Debug, Clone)]
pub enum ScanPattern {
    /// Exact byte pattern
    Exact(Vec<u8>),
    /// Pattern with wildcards (None = wildcard)
    Masked(Vec<Option<u8>>),
    /// String pattern
    String(String),
    /// Wide string pattern (UTF-16)
    WideString(String),
}

impl ScanPattern {
    /// Create pattern from hex string (e.g., "48 8B ?? ?? 89")
    pub fn from_hex_string(pattern: &str) -> MemoryResult<Self> {
        let mut bytes = Vec::new();
        let parts: Vec<&str> = pattern.split_whitespace().collect();
        
        for part in parts {
            if part == "??" || part == "?" {
                bytes.push(None);
            } else {
                let byte = u8::from_str_radix(part, 16)
                    .map_err(|_| MemoryError::InvalidPattern(format!("Invalid hex: {}", part)))?;
                bytes.push(Some(byte));
            }
        }
        
        if bytes.is_empty() {
            return Err(MemoryError::InvalidPattern("Empty pattern".to_string()));
        }
        
        Ok(ScanPattern::Masked(bytes))
    }
    
    /// Get the pattern length
    pub fn len(&self) -> usize {
        match self {
            ScanPattern::Exact(v) => v.len(),
            ScanPattern::Masked(v) => v.len(),
            ScanPattern::String(s) => s.len() + 1, // +1 for null terminator
            ScanPattern::WideString(s) => (s.len() + 1) * 2, // UTF-16 + null
        }
    }
    
    /// Check if pattern is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Convert to byte pattern for matching
    fn to_match_pattern(&self) -> (Vec<u8>, Vec<bool>) {
        match self {
            ScanPattern::Exact(bytes) => {
                let mask = vec![true; bytes.len()];
                (bytes.clone(), mask)
            }
            ScanPattern::Masked(pattern) => {
                let mut bytes = Vec::new();
                let mut mask = Vec::new();
                
                for byte_opt in pattern {
                    if let Some(byte) = byte_opt {
                        bytes.push(*byte);
                        mask.push(true);
                    } else {
                        bytes.push(0);
                        mask.push(false);
                    }
                }
                
                (bytes, mask)
            }
            ScanPattern::String(s) => {
                let mut bytes = s.as_bytes().to_vec();
                bytes.push(0); // Add null terminator
                let mask = vec![true; bytes.len()];
                (bytes, mask)
            }
            ScanPattern::WideString(s) => {
                let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes: Vec<u8> = wide.iter().flat_map(|&w| w.to_le_bytes()).collect();
                let mask = vec![true; bytes.len()];
                (bytes, mask)
            }
        }
    }
}

/// Options for memory scanning
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Start address for scanning
    pub start_address: Option<Address>,
    /// End address for scanning
    pub end_address: Option<Address>,
    /// Scan only executable regions
    pub executable_only: bool,
    /// Scan only writable regions
    pub writable_only: bool,
    /// Use parallel scanning for large regions
    pub parallel: bool,
    /// Alignment for scan (1, 2, 4, 8)
    pub alignment: usize,
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        ScanOptions {
            start_address: None,
            end_address: None,
            executable_only: false,
            writable_only: false,
            parallel: true,
            alignment: 1,
            max_results: Some(1000),
        }
    }
}

/// Memory scanner for pattern matching
pub struct MemoryScanner {
    handle: *const ProcessHandle,
}

impl MemoryScanner {
    /// Create a new memory scanner
    pub fn new(handle: &ProcessHandle) -> Self {
        MemoryScanner {
            handle: handle as *const ProcessHandle,
        }
    }
    
    /// Scan memory for a pattern
    pub fn scan(&self, pattern: &ScanPattern, options: ScanOptions) -> MemoryResult<Vec<Address>> {
        let (pattern_bytes, mask) = pattern.to_match_pattern();
        let regions = self.enumerate_regions(&options)?;
        
        if options.parallel && regions.len() > 1 {
            self.scan_parallel(&regions, &pattern_bytes, &mask, &options)
        } else {
            self.scan_sequential(&regions, &pattern_bytes, &mask, &options)
        }
    }
    
    /// Scan a specific memory region
    pub fn scan_region(
        &self,
        start: Address,
        size: usize,
        pattern: &ScanPattern,
        options: &ScanOptions,
    ) -> MemoryResult<Vec<Address>> {
        let (pattern_bytes, mask) = pattern.to_match_pattern();
        let mut buffer = vec![0u8; size];
        
        unsafe {
            let handle = &*self.handle;
            handle.read_memory(start.as_usize(), &mut buffer)?;
        }
        
        let mut results = Vec::new();
        let pattern_len = pattern_bytes.len();
        
        for i in (0..buffer.len().saturating_sub(pattern_len - 1)).step_by(options.alignment) {
            if self.matches_pattern(&buffer[i..], &pattern_bytes, &mask) {
                results.push(Address::new(start.as_usize() + i));
                
                if let Some(max) = options.max_results {
                    if results.len() >= max {
                        break;
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Find all occurrences of a value
    pub fn find_value<T: Copy>(&self, value: T, options: ScanOptions) -> MemoryResult<Vec<Address>> {
        let size = std::mem::size_of::<T>();
        let ptr = &value as *const T as *const u8;
        let pattern_bytes = unsafe { std::slice::from_raw_parts(ptr, size).to_vec() };
        
        self.scan(&ScanPattern::Exact(pattern_bytes), options)
    }
    
    /// Compare scan - find changed values
    pub fn compare_scan(
        &self,
        previous: &HashMap<Address, Vec<u8>>,
        comparison: ComparisonType,
    ) -> MemoryResult<Vec<Address>> {
        let mut results = Vec::new();
        
        for (addr, old_value) in previous {
            let mut new_value = vec![0u8; old_value.len()];
            
            unsafe {
                let handle = &*self.handle;
                if handle.read_memory(addr.as_usize(), &mut new_value).is_ok() {
                    if self.compare_values(old_value, &new_value, &comparison) {
                        results.push(*addr);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn enumerate_regions(&self, options: &ScanOptions) -> MemoryResult<Vec<(Address, usize)>> {
        let mut regions = Vec::new();
        let mut current = options.start_address.unwrap_or(Address::new(0x10000));
        let end = options.end_address.unwrap_or(Address::new(0x7FFFFFFFFFFF));
        
        unsafe {
            let handle = &*self.handle;
            
            while current < end {
                match kernel32::virtual_query_ex(handle.raw(), current.as_usize()) {
                    Ok(mbi) => {
                        const MEM_COMMIT: u32 = 0x1000;
                        const PAGE_EXECUTE: u32 = 0x10;
                        const PAGE_EXECUTE_READ: u32 = 0x20;
                        const PAGE_EXECUTE_READWRITE: u32 = 0x40;
                        const PAGE_EXECUTE_WRITECOPY: u32 = 0x80;
                        const PAGE_READWRITE: u32 = 0x04;
                        const PAGE_WRITECOPY: u32 = 0x08;
                        
                        if mbi.State == MEM_COMMIT {
                            let is_executable = mbi.Protect & (PAGE_EXECUTE | PAGE_EXECUTE_READ | 
                                                              PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0;
                            let is_writable = mbi.Protect & (PAGE_READWRITE | PAGE_WRITECOPY | 
                                                            PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY) != 0;
                            
                            let include = (!options.executable_only || is_executable) &&
                                        (!options.writable_only || is_writable);
                            
                            if include {
                                regions.push((Address::new(mbi.BaseAddress), mbi.RegionSize));
                            }
                        }
                        
                        current = Address::new(mbi.BaseAddress + mbi.RegionSize);
                    }
                    Err(_) => break,
                }
            }
        }
        
        Ok(regions)
    }
    
    fn scan_sequential(
        &self,
        regions: &[(Address, usize)],
        pattern: &[u8],
        mask: &[bool],
        options: &ScanOptions,
    ) -> MemoryResult<Vec<Address>> {
        let mut all_results = Vec::new();
        
        for (addr, size) in regions {
            let results = self.scan_region(*addr, *size, 
                &ScanPattern::Exact(pattern.to_vec()), options)?;
            all_results.extend(results);
            
            if let Some(max) = options.max_results {
                if all_results.len() >= max {
                    all_results.truncate(max);
                    break;
                }
            }
        }
        
        Ok(all_results)
    }
    
    fn scan_parallel(
        &self,
        regions: &[(Address, usize)],
        pattern: &[u8],
        mask: &[bool],
        options: &ScanOptions,
    ) -> MemoryResult<Vec<Address>> {
        let pattern = pattern.to_vec();
        let mask = mask.to_vec();
        let opt = options.clone();
        
        let results: Vec<Vec<Address>> = regions
            .par_iter()
            .map(|(addr, size)| {
                self.scan_region(*addr, *size, 
                    &ScanPattern::Exact(pattern.clone()), &opt)
                    .unwrap_or_default()
            })
            .collect();
        
        let mut all_results: Vec<Address> = results.into_iter().flatten().collect();
        
        if let Some(max) = options.max_results {
            all_results.truncate(max);
        }
        
        Ok(all_results)
    }
    
    fn matches_pattern(&self, data: &[u8], pattern: &[u8], mask: &[bool]) -> bool {
        if data.len() < pattern.len() {
            return false;
        }
        
        for i in 0..pattern.len() {
            if mask[i] && data[i] != pattern[i] {
                return false;
            }
        }
        
        true
    }
    
    fn compare_values(&self, old: &[u8], new: &[u8], comparison: &ComparisonType) -> bool {
        match comparison {
            ComparisonType::Equal => old == new,
            ComparisonType::NotEqual => old != new,
            ComparisonType::Greater => new > old,
            ComparisonType::Less => new < old,
            ComparisonType::GreaterOrEqual => new >= old,
            ComparisonType::LessOrEqual => new <= old,
        }
    }
}

/// Comparison type for compare scans
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonType {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_from_hex_string() {
        let pattern = ScanPattern::from_hex_string("48 8B ?? ?? 89").unwrap();
        match pattern {
            ScanPattern::Masked(bytes) => {
                assert_eq!(bytes.len(), 5);
                assert_eq!(bytes[0], Some(0x48));
                assert_eq!(bytes[1], Some(0x8B));
                assert_eq!(bytes[2], None);
                assert_eq!(bytes[3], None);
                assert_eq!(bytes[4], Some(0x89));
            }
            _ => panic!("Wrong pattern type"),
        }
        
        assert!(ScanPattern::from_hex_string("").is_err());
        assert!(ScanPattern::from_hex_string("GG").is_err());
    }
    
    #[test]
    fn test_pattern_length() {
        let exact = ScanPattern::Exact(vec![1, 2, 3]);
        assert_eq!(exact.len(), 3);
        
        let masked = ScanPattern::Masked(vec![Some(1), None, Some(3)]);
        assert_eq!(masked.len(), 3);
        
        let string = ScanPattern::String("test".to_string());
        assert_eq!(string.len(), 5); // "test" + null
        
        let wide = ScanPattern::WideString("test".to_string());
        assert_eq!(wide.len(), 10); // "test" in UTF-16 + null = 5 * 2
    }
    
    #[test]
    fn test_scan_options_default() {
        let opts = ScanOptions::default();
        assert_eq!(opts.alignment, 1);
        assert!(opts.parallel);
        assert_eq!(opts.max_results, Some(1000));
        assert!(!opts.executable_only);
        assert!(!opts.writable_only);
    }
    
    #[test]
    fn test_comparison_types() {
        let scanner = MemoryScanner {
            handle: std::ptr::null(),
        };
        
        assert!(scanner.compare_values(&[1, 2], &[1, 2], &ComparisonType::Equal));
        assert!(!scanner.compare_values(&[1, 2], &[1, 3], &ComparisonType::Equal));
        assert!(scanner.compare_values(&[1, 2], &[1, 3], &ComparisonType::NotEqual));
        assert!(scanner.compare_values(&[1, 2], &[1, 3], &ComparisonType::Less));
        assert!(scanner.compare_values(&[1, 3], &[1, 2], &ComparisonType::Greater));
    }
    
    #[test]
    fn test_pattern_matching() {
        let scanner = MemoryScanner {
            handle: std::ptr::null(),
        };
        
        let data = vec![0x48, 0x8B, 0xC1, 0xFF, 0x89];
        let pattern = vec![0x48, 0x8B, 0x00, 0x00, 0x89];
        let mask = vec![true, true, false, false, true];
        
        assert!(scanner.matches_pattern(&data, &pattern, &mask));
        
        let pattern2 = vec![0x48, 0x8C, 0x00, 0x00, 0x89];
        assert!(!scanner.matches_pattern(&data, &pattern2, &mask));
    }
}