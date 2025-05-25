mod references;
mod manager;

pub use references::ObjectReference;
pub use manager::MemoryManager;

use thiserror::Error;

/// Represents possible memory management errors
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Failed to allocate memory: {0}")]
    AllocationFailed(String),
    
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    
    #[error("Operation on current heap failed: {0}")]
    HeapError(String),
}

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Trait defining operations that can be performed on memory objects
pub trait MemoryObject: Send + Sync {
    /// Get the size of this object in bytes
    fn size(&self) -> usize;
    
    /// Return a unique type identifier for this object
    fn type_id(&self) -> u32;
}