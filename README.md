[![codecov](https://codecov.io/github/devjustus/Zero-MemoryMCP/graph/badge.svg?token=V0P64XTN39)](https://codecov.io/github/devjustus/Zero-MemoryMCP) [![Release](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/release.yml/badge.svg)](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/release.yml) [![Nightly Build](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/nightly.yml/badge.svg)](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/nightly.yml) [![Security](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/devjustus/Zero-MemoryMCP/actions/workflows/security.yml)

# Memory-MCP 🧠

A high-performance Model Context Protocol (MCP) server for advanced memory manipulation and process analysis, designed as a modern alternative to traditional memory editing tools.

## 🎯 Overview

Memory-MCP is a Rust-based MCP server that provides comprehensive memory manipulation capabilities through a clean, type-safe API. Built with performance and safety in mind, it offers direct Windows API access for process memory operations without the overhead of traditional GUI-based tools.

## ✨ Features

### Core Capabilities
- **Process Management**
  - Process enumeration and attachment
  - Module/DLL listing and analysis
  - x86/x64 process support with WoW64 handling
  - Automatic privilege escalation (SeDebugPrivilege)

- **Memory Operations**
  - High-speed parallel memory scanning
  - Multi-type value searching (integers, floats, strings, byte arrays)
  - Progressive scan refinement
  - Memory reading/writing with type safety
  - Region-based memory mapping

- **Advanced Scanning**
  - Array of Bytes (AOB) pattern scanning
  - Multi-level pointer chain resolution
  - Unknown initial value scanning
  - Increased/decreased value tracking
  - SIMD-optimized scan algorithms

- **MCP Integration**
  - Native MCP protocol support
  - Stateful scan sessions
  - Async/await architecture
  - JSON-RPC communication

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (with MSVC toolchain for Windows)
- Windows 10/11 (x64)
- Administrator privileges (for SeDebugPrivilege)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/memory-mcp.git
cd memory-mcp

# Build the project
cargo build --release

# Run the MCP server
cargo run --release
```

### Basic Usage

```json
// Attach to a process
{
  "method": "attach_process",
  "params": {
    "process_name": "notepad.exe"
  }
}

// Scan for a value
{
  "method": "scan_memory",
  "params": {
    "value": 100,
    "scan_type": "exact",
    "value_type": "i32"
  }
}

// Read memory
{
  "method": "read_memory",
  "params": {
    "address": "0x7FF6A0B0C0D0",
    "size": 4,
    "type": "i32"
  }
}
```

## 📖 API Documentation

### Process Operations

#### `attach_process`
Attaches to a target process for memory operations.

**Parameters:**
- `process_name` (string, optional): Name of the process
- `pid` (integer, optional): Process ID

**Returns:** Process information including handle, PID, and architecture

---

#### `list_processes`
Enumerates all running processes.

**Parameters:** None

**Returns:** Array of process information objects

---

#### `get_modules`
Lists all loaded modules in the attached process.

**Parameters:**
- `pid` (integer): Process ID

**Returns:** Array of module information with base addresses

### Memory Scanning

#### `scan_memory`
Performs memory scanning for specified values.

**Parameters:**
- `value` (variant): Value to search for
- `scan_type` (enum): Type of scan
  - `exact`: Exact value match
  - `unknown`: Unknown initial value
  - `increased`: Increased since last scan
  - `decreased`: Decreased since last scan
  - `changed`: Changed since last scan
  - `unchanged`: Unchanged since last scan
- `value_type` (enum): Data type
  - Integer types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
  - Float types: `f32`, `f64`
  - Other: `string`, `bytes`

**Returns:** Array of memory addresses containing matches

---

#### `next_scan`
Refines previous scan results with new criteria.

**Parameters:**
- `session_id` (string): Scan session identifier
- `value` (variant): New value to filter by
- `scan_type` (enum): Refinement type

**Returns:** Filtered array of addresses

### Memory Access

#### `read_memory`
Reads data from a memory address.

**Parameters:**
- `address` (string): Memory address in hex format
- `size` (integer): Number of bytes to read
- `type` (enum): Data type for interpretation

**Returns:** Value at the specified address

---

#### `write_memory`
Writes data to a memory address.

**Parameters:**
- `address` (string): Memory address in hex format
- `value` (variant): Value to write
- `type` (enum): Data type

**Returns:** Success status

### Pattern Scanning

#### `aob_scan`
Scans for byte patterns in memory.

**Parameters:**
- `pattern` (string): Byte pattern (supports wildcards with ??)
  - Example: `"48 89 5C 24 ?? 48 89 74 24 ??"`
- `module` (string, optional): Limit scan to specific module

**Returns:** Array of addresses where pattern was found

### Pointer Operations

#### `resolve_pointer`
Resolves multi-level pointer chains.

**Parameters:**
- `base_address` (string): Base address or module+offset
- `offsets` (array): Array of pointer offsets

**Returns:** Final resolved address

## 🏗️ Architecture

```
memory-mcp/
├── src/
│   ├── main.rs              # MCP server entry point
│   ├── memory/
│   │   ├── mod.rs           # Memory module exports
│   │   ├── scanner.rs       # Scanning algorithms
│   │   ├── reader.rs        # Memory reading operations
│   │   ├── writer.rs        # Memory writing operations
│   │   └── pointer.rs       # Pointer resolution
│   ├── process/
│   │   ├── mod.rs           # Process module exports
│   │   ├── manager.rs       # Process management
│   │   ├── modules.rs       # Module enumeration
│   │   └── privileges.rs    # Privilege handling
│   ├── scanner/
│   │   ├── mod.rs           # Scanner module exports
│   │   ├── aob.rs           # AOB pattern scanning
│   │   ├── algorithms.rs    # Scan algorithms
│   │   └── parallel.rs      # Parallel scanning
│   ├── mcp/
│   │   ├── mod.rs           # MCP module exports
│   │   ├── server.rs        # MCP server implementation
│   │   └── handlers.rs      # Request handlers
│   └── utils/
│       ├── mod.rs           # Utility exports
│       └── types.rs         # Type definitions
├── Cargo.toml               # Dependencies
└── README.md               # This file
```

## 🔧 Configuration

Create a `config.toml` file in the project root:

```toml
[server]
host = "127.0.0.1"
port = 3000
max_connections = 10

[scanner]
max_threads = 8
chunk_size = 65536
cache_size = 1048576

[memory]
max_read_size = 10485760
enable_write_protection = true
backup_before_write = true

[logging]
level = "info"
file = "memory-mcp.log"
```

## 🚀 Performance Optimizations

- **Parallel Scanning**: Utilizes Rayon for CPU-bound operations
- **SIMD Instructions**: Leverages SSE/AVX for pattern matching
- **Memory Mapping**: Uses memory-mapped I/O for large regions
- **Smart Caching**: Caches frequently accessed memory regions
- **Async I/O**: Non-blocking operations with Tokio runtime

## 🛡️ Safety Features

- **Type Safety**: Rust's type system prevents memory corruption
- **Automatic Cleanup**: RAII pattern for handle management
- **Write Protection**: Optional safeguards for critical regions
- **Backup System**: Automatic backup before memory writes
- **Error Recovery**: Comprehensive error handling and recovery

## 🔍 Advanced Usage Examples

### Progressive Value Scanning
```rust
// First scan for unknown value
let session = scan_memory(ScanParams {
    scan_type: ScanType::Unknown,
    value_type: ValueType::I32,
    ..Default::default()
});

// Player takes damage, health decreased
let session = next_scan(session.id, ScanParams {
    scan_type: ScanType::Decreased,
    ..Default::default()
});

// Use health potion, health increased
let session = next_scan(session.id, ScanParams {
    scan_type: ScanType::Increased,
    ..Default::default()
});
```

### Pattern Scanning with Module Targeting
```rust
// Find specific function signature in game.exe
let addresses = aob_scan(AobParams {
    pattern: "48 89 5C 24 ?? 48 89 74 24 ?? 57 48 83 EC 20",
    module: Some("game.exe"),
    ..Default::default()
});
```

### Multi-Level Pointer Resolution
```rust
// Resolve player health through pointer chain
let health_addr = resolve_pointer(PointerParams {
    base: "game.exe+0x1A2B3C",
    offsets: vec![0x10, 0x48, 0x0, 0x14],
});
```

## 🐛 Troubleshooting

### Common Issues

**"Access Denied" Error**
- Run the server with administrator privileges
- Ensure SeDebugPrivilege is enabled

**"Process Not Found"**
- Verify process name is correct (case-sensitive)
- Check if process is running with different privileges

**Slow Scanning Performance**
- Adjust `max_threads` in configuration
- Increase `chunk_size` for large memory scans
- Use module-specific scanning when possible

**Memory Read Failures**
- Check if address is valid and accessible
- Verify process architecture matches server (x64)
- Ensure memory region has read permissions

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

### Development Setup
```bash
# Install development dependencies
cargo install cargo-watch cargo-expand

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Watch for changes
cargo watch -x run
```

### 🚨 IMPORTANT: Before Committing

**Always run the CI validation script before committing to avoid pipeline failures:**

```bash
# Windows:
.\ci-check.bat

# Linux/Mac:
./ci-check.sh
```

This script runs ALL the same checks as the CI pipeline:
- ✅ Code formatting (`cargo fmt`)
- ✅ Clippy linting with CI flags (`cargo clippy -- -D warnings`)
- ✅ All tests (`cargo test --all`)
- ✅ Documentation build (`cargo doc`)
- ✅ Security audit (`cargo audit`)

**Optional: Setup pre-commit hooks for automatic validation:**
```bash
pip install pre-commit
pre-commit install
```

### Quick Validation Commands
If you prefer running checks manually:
```bash
# The essential checks - run these ALWAYS before committing:
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --no-fail-fast
```

## 📊 Benchmarks

| Operation | Time | Memory |
|-----------|------|--------|
| Process Enumeration | ~5ms | 2MB |
| 4GB Memory Scan (i32) | ~800ms | 50MB |
| AOB Pattern Scan | ~150ms | 20MB |
| Pointer Resolution (5 levels) | ~2ms | 1MB |
| Memory Read (4KB) | ~0.5ms | 4KB |

*Benchmarks on: Intel i7-12700K, 32GB RAM, Windows 11*

## 🔒 Security Considerations

- This tool requires administrator privileges
- Only use on processes you own or have permission to modify
- Be aware of anti-cheat systems that may detect memory access
- Always backup important data before memory modifications

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- Windows API documentation and community
- Rust memory manipulation crates ecosystem
- MCP protocol specification contributors

## 📞 Support

- GitHub Issues: [Report bugs or request features](https://github.com/yourusername/memory-mcp/issues)
- Discord: [Join our community](https://discord.gg/memorymcp)
- Documentation: [Full API docs](https://docs.memory-mcp.dev)

---

**Memory-MCP** - Modern memory manipulation through Model Context Protocol 🚀