//! Windows-specific type definitions and wrappers

pub mod handle;
pub mod memory_info;
pub mod module_info;

// Re-export commonly used types
pub use handle::Handle;
pub use memory_info::MemoryBasicInfo;
pub use module_info::ModuleInfo;

#[cfg(test)]
mod tests {
    #[test]
    fn test_types_available() {
        // Types should be available - compile-time test
    }
}
