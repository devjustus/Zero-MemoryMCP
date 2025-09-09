//! Memory value enum for handling different data types

use std::fmt;
use serde::{Deserialize, Serialize};

/// Represents different types of values that can be stored in memory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MemoryValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    String(String),
}

impl MemoryValue {
    /// Returns the size in bytes of the value
    pub fn size(&self) -> usize {
        match self {
            MemoryValue::I8(_) | MemoryValue::U8(_) => 1,
            MemoryValue::I16(_) | MemoryValue::U16(_) => 2,
            MemoryValue::I32(_) | MemoryValue::U32(_) | MemoryValue::F32(_) => 4,
            MemoryValue::I64(_) | MemoryValue::U64(_) | MemoryValue::F64(_) => 8,
            MemoryValue::Bytes(b) => b.len(),
            MemoryValue::String(s) => s.len(),
        }
    }

    /// Converts the value to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            MemoryValue::I8(v) => v.to_le_bytes().to_vec(),
            MemoryValue::I16(v) => v.to_le_bytes().to_vec(),
            MemoryValue::I32(v) => v.to_le_bytes().to_vec(),
            MemoryValue::I64(v) => v.to_le_bytes().to_vec(),
            MemoryValue::U8(v) => v.to_le_bytes().to_vec(),
            MemoryValue::U16(v) => v.to_le_bytes().to_vec(),
            MemoryValue::U32(v) => v.to_le_bytes().to_vec(),
            MemoryValue::U64(v) => v.to_le_bytes().to_vec(),
            MemoryValue::F32(v) => v.to_le_bytes().to_vec(),
            MemoryValue::F64(v) => v.to_le_bytes().to_vec(),
            MemoryValue::Bytes(b) => b.clone(),
            MemoryValue::String(s) => s.as_bytes().to_vec(),
        }
    }

    /// Creates a value from bytes based on the specified type
    pub fn from_bytes(bytes: &[u8], value_type: ValueType) -> Option<Self> {
        match value_type {
            ValueType::I8 => bytes.get(0).map(|&b| MemoryValue::I8(b as i8)),
            ValueType::I16 => {
                if bytes.len() >= 2 {
                    Some(MemoryValue::I16(i16::from_le_bytes([bytes[0], bytes[1]])))
                } else {
                    None
                }
            }
            ValueType::I32 => {
                if bytes.len() >= 4 {
                    Some(MemoryValue::I32(i32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ])))
                } else {
                    None
                }
            }
            ValueType::I64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::I64(i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ])))
                } else {
                    None
                }
            }
            ValueType::U8 => bytes.get(0).map(|&b| MemoryValue::U8(b)),
            ValueType::U16 => {
                if bytes.len() >= 2 {
                    Some(MemoryValue::U16(u16::from_le_bytes([bytes[0], bytes[1]])))
                } else {
                    None
                }
            }
            ValueType::U32 => {
                if bytes.len() >= 4 {
                    Some(MemoryValue::U32(u32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ])))
                } else {
                    None
                }
            }
            ValueType::U64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::U64(u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ])))
                } else {
                    None
                }
            }
            ValueType::F32 => {
                if bytes.len() >= 4 {
                    Some(MemoryValue::F32(f32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3]
                    ])))
                } else {
                    None
                }
            }
            ValueType::F64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::F64(f64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7]
                    ])))
                } else {
                    None
                }
            }
            ValueType::Bytes => Some(MemoryValue::Bytes(bytes.to_vec())),
            ValueType::String => {
                String::from_utf8(bytes.to_vec())
                    .ok()
                    .map(MemoryValue::String)
            }
        }
    }

    /// Gets the value type enum for this value
    pub fn value_type(&self) -> ValueType {
        match self {
            MemoryValue::I8(_) => ValueType::I8,
            MemoryValue::I16(_) => ValueType::I16,
            MemoryValue::I32(_) => ValueType::I32,
            MemoryValue::I64(_) => ValueType::I64,
            MemoryValue::U8(_) => ValueType::U8,
            MemoryValue::U16(_) => ValueType::U16,
            MemoryValue::U32(_) => ValueType::U32,
            MemoryValue::U64(_) => ValueType::U64,
            MemoryValue::F32(_) => ValueType::F32,
            MemoryValue::F64(_) => ValueType::F64,
            MemoryValue::Bytes(_) => ValueType::Bytes,
            MemoryValue::String(_) => ValueType::String,
        }
    }
}

/// Enum representing the type of a memory value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bytes,
    String,
}

impl ValueType {
    /// Returns the size in bytes for this value type
    pub fn size(&self) -> Option<usize> {
        match self {
            ValueType::I8 | ValueType::U8 => Some(1),
            ValueType::I16 | ValueType::U16 => Some(2),
            ValueType::I32 | ValueType::U32 | ValueType::F32 => Some(4),
            ValueType::I64 | ValueType::U64 | ValueType::F64 => Some(8),
            ValueType::Bytes | ValueType::String => None, // Variable size
        }
    }
}

impl fmt::Display for MemoryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryValue::I8(v) => write!(f, "{}", v),
            MemoryValue::I16(v) => write!(f, "{}", v),
            MemoryValue::I32(v) => write!(f, "{}", v),
            MemoryValue::I64(v) => write!(f, "{}", v),
            MemoryValue::U8(v) => write!(f, "{}", v),
            MemoryValue::U16(v) => write!(f, "{}", v),
            MemoryValue::U32(v) => write!(f, "{}", v),
            MemoryValue::U64(v) => write!(f, "{}", v),
            MemoryValue::F32(v) => write!(f, "{}", v),
            MemoryValue::F64(v) => write!(f, "{}", v),
            MemoryValue::Bytes(b) => write!(f, "{:?}", b),
            MemoryValue::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_size() {
        assert_eq!(MemoryValue::I32(42).size(), 4);
        assert_eq!(MemoryValue::U64(100).size(), 8);
        assert_eq!(MemoryValue::F32(3.14).size(), 4);
        assert_eq!(MemoryValue::Bytes(vec![1, 2, 3]).size(), 3);
    }

    #[test]
    fn test_value_to_bytes() {
        assert_eq!(MemoryValue::U32(0x12345678).to_bytes(), vec![0x78, 0x56, 0x34, 0x12]);
        assert_eq!(MemoryValue::I8(-1).to_bytes(), vec![0xFF]);
        assert_eq!(MemoryValue::String("Hi".to_string()).to_bytes(), vec![b'H', b'i']);
    }

    #[test]
    fn test_value_from_bytes() {
        let bytes = vec![0x78, 0x56, 0x34, 0x12];
        let value = MemoryValue::from_bytes(&bytes, ValueType::U32).unwrap();
        assert_eq!(value, MemoryValue::U32(0x12345678));
    }
}