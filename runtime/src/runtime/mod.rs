mod executor;
mod config;

pub use executor::Runtime;
pub use config::RuntimeConfig;

use std::io;
use thiserror::Error;

/// Errors that can occur in the runtime
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("VM error: {0}")]
    VMError(#[from] crate::vm::VMError),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("Bytecode parsing error: {0}")]
    ParsingError(#[from] crate::bytecode::ParseError),
    
    #[error("Memory error: {0}")]
    MemoryError(#[from] crate::memory::MemoryError),
    
    #[error("Runtime configuration error: {0}")]
    ConfigError(String),
}

/// Result type for runtime operations
pub type RuntimeResult<T> = Result<T, RuntimeError>;