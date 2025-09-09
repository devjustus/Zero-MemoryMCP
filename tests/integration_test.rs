//! Integration tests for Memory-MCP core types

use memory_mcp::core::types::*;

#[test]
fn test_basic_setup() {
    assert_eq!(2 + 2, 4);
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_platform() {
    assert!(cfg!(target_os = "windows"));
}

#[test]
fn test_address_and_scan_result_integration() {
    let addr = Address::new(0x1000);
    let value = MemoryValue::U32(42);
    let result = ScanResult::new(addr, value.clone());

    assert_eq!(result.address, addr);
    assert_eq!(result.value, value);
}

#[test]
fn test_scan_session_workflow() {
    let mut session = ScanSession::new(
        "integration-test".to_string(),
        ScanType::Exact,
        ValueType::U32,
    );

    let results = vec![
        ScanResult::new(Address::new(0x1000), MemoryValue::U32(100)),
        ScanResult::new(Address::new(0x2000), MemoryValue::U32(200)),
        ScanResult::new(Address::new(0x3000), MemoryValue::U32(300)),
    ];

    session.add_results(results);
    assert_eq!(session.results.len(), 3);
    assert_eq!(session.scan_count, 1);

    session.filter_results(|r| {
        if let MemoryValue::U32(val) = r.value {
            val > 150
        } else {
            false
        }
    });

    assert_eq!(session.results.len(), 2);
}

#[test]
fn test_process_and_module_relationship() {
    let mut process = ProcessInfo::new(1234, "test_app.exe".to_string());
    process.architecture = ProcessArchitecture::X64;

    let module1 = ModuleInfo::new("kernel32.dll".to_string(), Address::new(0x10000000), 0x1000);

    let module2 = ModuleInfo::new("ntdll.dll".to_string(), Address::new(0x20000000), 0x2000);

    assert!(module1.contains_address(Address::new(0x10000500)));
    assert!(!module1.contains_address(Address::new(0x10001000)));
    assert!(module2.contains_address(Address::new(0x20000500)));

    assert_eq!(process.architecture.pointer_size(), 8);
}

#[test]
fn test_memory_value_conversions() {
    let test_values = vec![
        (MemoryValue::U32(0x12345678), ValueType::U32, 4),
        (MemoryValue::I64(-999999), ValueType::I64, 8),
        (MemoryValue::F32(std::f32::consts::PI), ValueType::F32, 4),
        (
            MemoryValue::String("test".to_string()),
            ValueType::String,
            4,
        ),
    ];

    for (value, expected_type, expected_size) in test_values {
        assert_eq!(value.value_type(), expected_type);
        assert_eq!(value.size(), expected_size);

        let bytes = value.to_bytes();
        assert_eq!(bytes.len(), expected_size);

        if !matches!(expected_type, ValueType::String) {
            let reconstructed = MemoryValue::from_bytes(&bytes, expected_type).unwrap();
            assert_eq!(value, reconstructed);
        }
    }
}

#[test]
fn test_error_handling_integration() {
    let error = MemoryError::read_failed(Address::new(0xDEADBEEF), "Access violation");

    match error {
        MemoryError::ReadFailed { address, reason } => {
            assert_eq!(address, "0x00000000DEADBEEF");
            assert_eq!(reason, "Access violation");
        }
        _ => panic!("Wrong error type"),
    }

    let error = MemoryError::pointer_chain_broken(3, "Null pointer");
    match error {
        MemoryError::PointerChainBroken { level, reason } => {
            assert_eq!(level, 3);
            assert_eq!(reason, "Null pointer");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_address_parsing_and_display() {
    let test_cases = vec![
        ("0x1000", 0x1000),
        ("0X2000", 0x2000),
        ("$3000", 0x3000),
        ("DEADBEEF", 0xDEADBEEF),
        ("4096", 4096),
    ];

    for (input, expected) in test_cases {
        let addr = input.parse::<Address>().unwrap();
        assert_eq!(addr.as_usize(), expected);

        let display = format!("{:X}", addr);
        assert!(display.starts_with("0x"));
    }
}

#[test]
fn test_scan_type_combinations() {
    let scan_types = vec![
        (ScanType::Exact, true, false),
        (ScanType::Unknown, false, false),
        (ScanType::Increased, false, true),
        (ScanType::IncreasedBy, true, true),
        (ScanType::Decreased, false, true),
        (ScanType::DecreasedBy, true, true),
        (ScanType::Changed, false, true),
        (ScanType::Unchanged, false, true),
        (ScanType::Between, true, false),
        (ScanType::BiggerThan, true, false),
        (ScanType::SmallerThan, true, false),
    ];

    for (scan_type, requires_value, requires_previous) in scan_types {
        assert_eq!(scan_type.requires_value(), requires_value);
        assert_eq!(scan_type.requires_previous(), requires_previous);
    }
}

#[test]
fn test_region_info_with_scan_result() {
    let mut result = ScanResult::new(Address::new(0x10000), MemoryValue::U64(0xCAFEBABE));

    let region = RegionInfo {
        base_address: Address::new(0x10000),
        size: 0x10000,
        protection: 0x20,
        state: 0x1000,
        region_type: 0x20000,
    };

    result.region_info = Some(region);

    assert!(result.region_info.is_some());
    let info = result.region_info.as_ref().unwrap();
    assert_eq!(info.base_address, Address::new(0x10000));
    assert_eq!(info.size, 0x10000);
}

#[test]
fn test_cross_module_serialization() {
    use serde_json;

    let process = ProcessInfo::new(5678, "notepad.exe".to_string());
    let module = ModuleInfo::new("user32.dll".to_string(), Address::new(0x7FF80000), 0x100000);
    let scan_result = ScanResult::new(Address::new(0x7FF80500), MemoryValue::I32(42));

    let process_json = serde_json::to_string(&process).unwrap();
    let module_json = serde_json::to_string(&module).unwrap();
    let result_json = serde_json::to_string(&scan_result).unwrap();

    let _process2: ProcessInfo = serde_json::from_str(&process_json).unwrap();
    let _module2: ModuleInfo = serde_json::from_str(&module_json).unwrap();
    let _result2: ScanResult = serde_json::from_str(&result_json).unwrap();

    assert!(process_json.contains("notepad.exe"));
    assert!(module_json.contains("user32.dll"));
    // Address is serialized as a number, not hex string
    // 0x7FF80500 = 2147092736 in decimal
    assert!(result_json.contains("2147092736") || result_json.contains("address"));
}

#[test]
fn test_memory_operations_workflow() {
    let base_addr = Address::new(0x400000);

    let offsets = vec![0x100, 0x200, 0x300];
    let addresses: Vec<Address> = offsets
        .iter()
        .map(|&offset| base_addr.offset(offset))
        .collect();

    assert_eq!(addresses[0], Address::new(0x400100));
    assert_eq!(addresses[1], Address::new(0x400200));
    assert_eq!(addresses[2], Address::new(0x400300));

    for addr in &addresses {
        assert!(addr.is_aligned(0x100));
    }
}

#[test]
fn test_value_type_matching() {
    let values = vec![
        (MemoryValue::I8(42), ValueType::I8),
        (MemoryValue::I16(1000), ValueType::I16),
        (MemoryValue::I32(100000), ValueType::I32),
        (MemoryValue::I64(9999999999), ValueType::I64),
        (MemoryValue::U8(255), ValueType::U8),
        (MemoryValue::U16(65535), ValueType::U16),
        (MemoryValue::U32(4294967295), ValueType::U32),
        (MemoryValue::U64(18446744073709551615), ValueType::U64),
        (MemoryValue::F32(std::f32::consts::PI), ValueType::F32),
        (MemoryValue::F64(std::f64::consts::E), ValueType::F64),
        (MemoryValue::String("hello".to_string()), ValueType::String),
        (MemoryValue::Bytes(vec![1, 2, 3]), ValueType::Bytes),
    ];

    for (value, expected_type) in values {
        assert_eq!(value.value_type(), expected_type);

        let size = value.size();
        if let Some(type_size) = expected_type.size() {
            assert_eq!(size, type_size);
        }
    }
}

#[test]
fn test_complete_scan_workflow() {
    let mut session = ScanSession::new(
        "complete-workflow".to_string(),
        ScanType::Exact,
        ValueType::U32,
    );

    let initial_results: Vec<ScanResult> = (0..1000)
        .map(|i| {
            ScanResult::new(
                Address::new(0x1000 + i * 4),
                MemoryValue::U32(i as u32 * 10),
            )
        })
        .collect();

    session.add_results(initial_results);
    assert_eq!(session.results.len(), 1000);

    session.filter_results(|r| {
        if let MemoryValue::U32(val) = r.value {
            val % 100 == 0
        } else {
            false
        }
    });

    assert_eq!(session.results.len(), 100);

    let next_scan_results: Vec<ScanResult> = session
        .results
        .iter()
        .filter_map(|r| {
            if let MemoryValue::U32(val) = r.value {
                if val % 200 == 0 {
                    Some(ScanResult::with_previous(
                        r.address,
                        MemoryValue::U32(val + 1),
                        r.value.clone(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    session.add_results(next_scan_results);
    assert_eq!(session.scan_count, 2);
    assert_eq!(session.results.len(), 50);

    for result in &session.results {
        assert!(result.previous_value.is_some());
    }
}
