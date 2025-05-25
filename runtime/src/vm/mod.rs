mod interpreter;
mod execution_context;
mod value;
mod error;

pub use interpreter::VM;
pub use execution_context::ExecutionContext;
pub use value::Value;
pub use error::{VMError, VMResult};