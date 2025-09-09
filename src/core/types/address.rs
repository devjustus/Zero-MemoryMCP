//! Memory address wrapper type with hex parsing and validation

use super::error::{MemoryError, MemoryResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents a memory address with type-safe operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Address(pub usize);

impl Address {
    /// Creates a new address from a usize value
    pub const fn new(value: usize) -> Self {
        Address(value)
    }

    /// Creates a null address (0x0)
    pub const fn null() -> Self {
        Address(0)
    }

    /// Checks if the address is null
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Checks if the address is aligned to the specified boundary
    pub const fn is_aligned(&self, alignment: usize) -> bool {
        alignment != 0 && self.0 % alignment == 0
    }

    /// Aligns the address down to the specified boundary
    pub const fn align_down(&self, alignment: usize) -> Self {
        if alignment == 0 {
            return *self;
        }
        Address(self.0 & !(alignment - 1))
    }

    /// Aligns the address up to the specified boundary
    pub const fn align_up(&self, alignment: usize) -> Self {
        if alignment == 0 {
            return *self;
        }
        Address((self.0 + alignment - 1) & !(alignment - 1))
    }

    /// Adds an offset to the address
    pub const fn offset(&self, offset: isize) -> Self {
        Address((self.0 as isize + offset) as usize)
    }

    /// Returns the raw usize value
    pub const fn as_usize(&self) -> usize {
        self.0
    }

    /// Returns the address as a pointer
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    /// Returns the address as a mutable pointer
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl FromStr for Address {
    type Err = MemoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Handle hex prefix variations
        let value = if s.starts_with("0x") || s.starts_with("0X") {
            usize::from_str_radix(&s[2..], 16)
        } else if s.starts_with("$") {
            usize::from_str_radix(&s[1..], 16)
        } else if s.chars().any(|c| c.is_ascii_alphabetic()) {
            // Assume hex if contains letters
            usize::from_str_radix(s, 16)
        } else {
            // Try decimal first, then hex
            s.parse::<usize>().or_else(|_| usize::from_str_radix(s, 16))
        };

        value
            .map(Address::new)
            .map_err(|_| MemoryError::InvalidAddress(s.to_string()))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.0)
    }
}

impl fmt::UpperHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}

impl From<usize> for Address {
    fn from(value: usize) -> Self {
        Address::new(value)
    }
}

impl From<u64> for Address {
    fn from(value: u64) -> Self {
        Address::new(value as usize)
    }
}

impl From<*const u8> for Address {
    fn from(ptr: *const u8) -> Self {
        Address::new(ptr as usize)
    }
}

impl From<*mut u8> for Address {
    fn from(ptr: *mut u8) -> Self {
        Address::new(ptr as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_parsing() {
        assert_eq!(Address::from_str("0x1000").unwrap(), Address::new(0x1000));
        assert_eq!(Address::from_str("0X1000").unwrap(), Address::new(0x1000));
        assert_eq!(Address::from_str("$1000").unwrap(), Address::new(0x1000));
        assert_eq!(
            Address::from_str("DEADBEEF").unwrap(),
            Address::new(0xDEADBEEF)
        );
        assert_eq!(Address::from_str("4096").unwrap(), Address::new(4096));
    }

    #[test]
    fn test_address_alignment() {
        let addr = Address::new(0x1005);
        assert!(!addr.is_aligned(4));
        assert_eq!(addr.align_down(4), Address::new(0x1004));
        assert_eq!(addr.align_up(4), Address::new(0x1008));

        let aligned = Address::new(0x1000);
        assert!(aligned.is_aligned(16));
    }

    #[test]
    fn test_address_offset() {
        let addr = Address::new(0x1000);
        assert_eq!(addr.offset(0x10), Address::new(0x1010));
        assert_eq!(addr.offset(-0x10), Address::new(0x0FF0));
    }

    #[test]
    fn test_address_display() {
        let addr = Address::new(0xDEADBEEF);
        assert_eq!(format!("{}", addr), "0x00000000DEADBEEF");
        assert_eq!(format!("{:x}", addr), "0x00000000deadbeef");
        assert_eq!(format!("{:X}", addr), "0x00000000DEADBEEF");
    }
}
