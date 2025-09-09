//! Memory value enum for handling different data types

use serde::{Deserialize, Serialize};
use std::fmt;

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
            ValueType::I8 => bytes.first().map(|&b| MemoryValue::I8(b as i8)),
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
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            ValueType::I64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::I64(i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])))
                } else {
                    None
                }
            }
            ValueType::U8 => bytes.first().map(|&b| MemoryValue::U8(b)),
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
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            ValueType::U64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::U64(u64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])))
                } else {
                    None
                }
            }
            ValueType::F32 => {
                if bytes.len() >= 4 {
                    Some(MemoryValue::F32(f32::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ])))
                } else {
                    None
                }
            }
            ValueType::F64 => {
                if bytes.len() >= 8 {
                    Some(MemoryValue::F64(f64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])))
                } else {
                    None
                }
            }
            ValueType::Bytes => Some(MemoryValue::Bytes(bytes.to_vec())),
            ValueType::String => String::from_utf8(bytes.to_vec())
                .ok()
                .map(MemoryValue::String),
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
        assert_eq!(MemoryValue::F32(3.25).size(), 4);
        assert_eq!(MemoryValue::Bytes(vec![1, 2, 3]).size(), 3);
    }

    #[test]
    fn test_value_to_bytes() {
        assert_eq!(
            MemoryValue::U32(0x12345678).to_bytes(),
            vec![0x78, 0x56, 0x34, 0x12]
        );
        assert_eq!(MemoryValue::I8(-1).to_bytes(), vec![0xFF]);
        assert_eq!(
            MemoryValue::String("Hi".to_string()).to_bytes(),
            vec![b'H', b'i']
        );
    }

    #[test]
    fn test_value_from_bytes() {
        let bytes = vec![0x78, 0x56, 0x34, 0x12];
        let value = MemoryValue::from_bytes(&bytes, ValueType::U32).unwrap();
        assert_eq!(value, MemoryValue::U32(0x12345678));
    }

    #[test]
    fn test_all_value_types_from_bytes() {
        assert_eq!(
            MemoryValue::from_bytes(&[42], ValueType::I8).unwrap(),
            MemoryValue::I8(42)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[255], ValueType::U8).unwrap(),
            MemoryValue::U8(255)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[0x34, 0x12], ValueType::I16).unwrap(),
            MemoryValue::I16(0x1234)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[0x34, 0x12], ValueType::U16).unwrap(),
            MemoryValue::U16(0x1234)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[0x78, 0x56, 0x34, 0x12], ValueType::I32).unwrap(),
            MemoryValue::I32(0x12345678)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[0x78, 0x56, 0x34, 0x12], ValueType::U32).unwrap(),
            MemoryValue::U32(0x12345678)
        );
        assert_eq!(
            MemoryValue::from_bytes(
                &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F],
                ValueType::F64
            )
            .unwrap(),
            MemoryValue::F64(1.0)
        );
        assert_eq!(
            MemoryValue::from_bytes(&[0x00, 0x00, 0x80, 0x3F], ValueType::F32).unwrap(),
            MemoryValue::F32(1.0)
        );
        assert_eq!(
            MemoryValue::from_bytes(
                &[0xEF, 0xBE, 0xAD, 0xDE, 0x00, 0x00, 0x00, 0x00],
                ValueType::I64
            )
            .unwrap(),
            MemoryValue::I64(0xDEADBEEF)
        );
        assert_eq!(
            MemoryValue::from_bytes(
                &[0xEF, 0xBE, 0xAD, 0xDE, 0x00, 0x00, 0x00, 0x00],
                ValueType::U64
            )
            .unwrap(),
            MemoryValue::U64(0xDEADBEEF)
        );
        assert_eq!(
            MemoryValue::from_bytes(b"Hello", ValueType::String).unwrap(),
            MemoryValue::String("Hello".to_string())
        );
        assert_eq!(
            MemoryValue::from_bytes(&[1, 2, 3], ValueType::Bytes).unwrap(),
            MemoryValue::Bytes(vec![1, 2, 3])
        );
    }

    #[test]
    fn test_from_bytes_insufficient_data() {
        assert!(MemoryValue::from_bytes(&[], ValueType::I8).is_none());
        assert!(MemoryValue::from_bytes(&[1], ValueType::I16).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2], ValueType::I32).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2, 3, 4], ValueType::I64).is_none());
        assert!(MemoryValue::from_bytes(&[1], ValueType::U16).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2], ValueType::U32).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2, 3, 4], ValueType::U64).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2], ValueType::F32).is_none());
        assert!(MemoryValue::from_bytes(&[1, 2, 3, 4], ValueType::F64).is_none());
    }

    #[test]
    fn test_from_bytes_invalid_utf8() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        assert!(MemoryValue::from_bytes(&invalid_utf8, ValueType::String).is_none());
    }

    #[test]
    fn test_value_type_enum() {
        let value = MemoryValue::I32(42);
        assert_eq!(value.value_type(), ValueType::I32);

        let value = MemoryValue::U64(100);
        assert_eq!(value.value_type(), ValueType::U64);

        let value = MemoryValue::F32(std::f32::consts::PI);
        assert_eq!(value.value_type(), ValueType::F32);

        let value = MemoryValue::String("test".to_string());
        assert_eq!(value.value_type(), ValueType::String);

        let value = MemoryValue::Bytes(vec![1, 2, 3]);
        assert_eq!(value.value_type(), ValueType::Bytes);
    }

    #[test]
    fn test_value_type_size() {
        assert_eq!(ValueType::I8.size(), Some(1));
        assert_eq!(ValueType::U8.size(), Some(1));
        assert_eq!(ValueType::I16.size(), Some(2));
        assert_eq!(ValueType::U16.size(), Some(2));
        assert_eq!(ValueType::I32.size(), Some(4));
        assert_eq!(ValueType::U32.size(), Some(4));
        assert_eq!(ValueType::F32.size(), Some(4));
        assert_eq!(ValueType::I64.size(), Some(8));
        assert_eq!(ValueType::U64.size(), Some(8));
        assert_eq!(ValueType::F64.size(), Some(8));
        assert_eq!(ValueType::Bytes.size(), None);
        assert_eq!(ValueType::String.size(), None);
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(format!("{}", MemoryValue::I8(-42)), "-42");
        assert_eq!(format!("{}", MemoryValue::I16(1000)), "1000");
        assert_eq!(format!("{}", MemoryValue::I32(-100000)), "-100000");
        assert_eq!(format!("{}", MemoryValue::I64(9999999999)), "9999999999");
        assert_eq!(format!("{}", MemoryValue::U8(255)), "255");
        assert_eq!(format!("{}", MemoryValue::U16(65535)), "65535");
        assert_eq!(format!("{}", MemoryValue::U32(4294967295)), "4294967295");
        assert_eq!(
            format!("{}", MemoryValue::U64(18446744073709551615)),
            "18446744073709551615"
        );
        // Using PI and E constants - actual values will vary slightly from hardcoded
        let pi_str = format!("{}", std::f32::consts::PI);
        assert_eq!(
            format!("{}", MemoryValue::F32(std::f32::consts::PI)),
            pi_str
        );
        let e_str = format!("{}", std::f64::consts::E);
        assert_eq!(format!("{}", MemoryValue::F64(std::f64::consts::E)), e_str);
        assert_eq!(
            format!("{}", MemoryValue::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(
            format!("{}", MemoryValue::Bytes(vec![1, 2, 3])),
            "[1, 2, 3]"
        );
    }

    #[test]
    fn test_serialization_deserialization() {
        let values = vec![
            MemoryValue::I8(-128),
            MemoryValue::I16(-32768),
            MemoryValue::I32(-2147483648),
            MemoryValue::I64(-9223372036854775808),
            MemoryValue::U8(255),
            MemoryValue::U16(65535),
            MemoryValue::U32(4294967295),
            MemoryValue::U64(18446744073709551615),
            MemoryValue::F32(std::f32::consts::PI),
            MemoryValue::F64(std::f64::consts::E),
            MemoryValue::String("test string".to_string()),
            MemoryValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        ];

        for value in values {
            let json = serde_json::to_string(&value).unwrap();
            let deserialized: MemoryValue = serde_json::from_str(&json).unwrap();
            assert_eq!(value, deserialized);
        }
    }

    #[test]
    fn test_value_type_serialization() {
        let types = vec![
            ValueType::I8,
            ValueType::I16,
            ValueType::I32,
            ValueType::I64,
            ValueType::U8,
            ValueType::U16,
            ValueType::U32,
            ValueType::U64,
            ValueType::F32,
            ValueType::F64,
            ValueType::Bytes,
            ValueType::String,
        ];

        for value_type in types {
            let json = serde_json::to_string(&value_type).unwrap();
            let deserialized: ValueType = serde_json::from_str(&json).unwrap();
            assert_eq!(value_type, deserialized);
        }
    }

    #[test]
    fn test_clone_and_debug() {
        let value = MemoryValue::U32(42);
        let cloned = value.clone();
        assert_eq!(value, cloned);

        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("U32"));
        assert!(debug_str.contains("42"));

        let value_type = ValueType::F64;
        let cloned = value_type.clone();
        assert_eq!(value_type, cloned);

        let debug_str = format!("{:?}", value_type);
        assert!(debug_str.contains("F64"));
    }

    #[test]
    fn test_edge_cases() {
        let zero_bytes: Vec<u8> = vec![];
        assert_eq!(
            MemoryValue::from_bytes(&zero_bytes, ValueType::Bytes).unwrap(),
            MemoryValue::Bytes(vec![])
        );

        let empty_string = vec![];
        assert_eq!(
            MemoryValue::from_bytes(&empty_string, ValueType::String).unwrap(),
            MemoryValue::String("".to_string())
        );

        let max_i8 = MemoryValue::I8(127);
        assert_eq!(max_i8.to_bytes(), vec![127]);

        let min_i8 = MemoryValue::I8(-128);
        assert_eq!(min_i8.to_bytes(), vec![128]);
    }
}
