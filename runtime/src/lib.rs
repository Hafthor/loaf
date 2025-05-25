// Loaf - A bytecode runtime with multiple memory-managed heaps

pub mod bytecode;
pub mod heap;
pub mod memory;
pub mod vm;
pub mod runtime;
pub mod utils;

pub use bytecode::Instruction;
pub use heap::{Heap, HeapManager};
pub use memory::{MemoryManager, ObjectReference};
pub use vm::{VM, ExecutionContext};
pub use runtime::Runtime;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");