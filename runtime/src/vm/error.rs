use thiserror::Error;
use crate::memory::MemoryError;

/// Error type for VM operations
#[derive(Error, Debug)]
pub enum VMError {
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("Invalid program counter: {0}")]
    InvalidProgramCounter(usize),
    
    #[error("Invalid constant index: {0}")]
    InvalidConstantIndex(u32),
    
    #[error("Invalid local variable index: {0}")]
    InvalidLocalIndex(u32),
    
    #[error("Invalid operand")]
    InvalidOperand,
    
    #[error("Type error: {0}")]
    TypeError(String),
    
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    
    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
}

// Add conversion from MemoryError to VMError
impl From<MemoryError> for VMError {
    fn from(err: MemoryError) -> Self {
        VMError::MemoryError(format!("{}", err))
    }
}

/// Result type for VM operations
pub type VMResult<T> = Result<T, VMError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryError;

    #[test]
    fn test_vm_error_stack_underflow_display() {
        let error = VMError::StackUnderflow;
        assert_eq!(error.to_string(), "Stack underflow");
    }

    #[test]
    fn test_vm_error_invalid_program_counter_display() {
        let error = VMError::InvalidProgramCounter(42);
        assert_eq!(error.to_string(), "Invalid program counter: 42");
    }

    #[test]
    fn test_vm_error_invalid_constant_index_display() {
        let error = VMError::InvalidConstantIndex(10);
        assert_eq!(error.to_string(), "Invalid constant index: 10");
    }

    #[test]
    fn test_vm_error_invalid_local_index_display() {
        let error = VMError::InvalidLocalIndex(5);
        assert_eq!(error.to_string(), "Invalid local variable index: 5");
    }

    #[test]
    fn test_vm_error_invalid_operand_display() {
        let error = VMError::InvalidOperand;
        assert_eq!(error.to_string(), "Invalid operand");
    }

    #[test]
    fn test_vm_error_type_error_display() {
        let error = VMError::TypeError("Expected integer, found string".to_string());
        assert_eq!(error.to_string(), "Type error: Expected integer, found string");
    }

    #[test]
    fn test_vm_error_index_out_of_bounds_display() {
        let error = VMError::IndexOutOfBounds(100);
        assert_eq!(error.to_string(), "Index out of bounds: 100");
    }

    #[test]
    fn test_vm_error_arithmetic_error_display() {
        let error = VMError::ArithmeticError("Integer overflow".to_string());
        assert_eq!(error.to_string(), "Arithmetic error: Integer overflow");
    }

    #[test]
    fn test_vm_error_division_by_zero_display() {
        let error = VMError::DivisionByZero;
        assert_eq!(error.to_string(), "Division by zero");
    }

    #[test]
    fn test_vm_error_invalid_operation_display() {
        let error = VMError::InvalidOperation("Cannot negate a string".to_string());
        assert_eq!(error.to_string(), "Invalid operation: Cannot negate a string");
    }

    #[test]
    fn test_vm_error_runtime_error_display() {
        let error = VMError::RuntimeError("Null pointer exception".to_string());
        assert_eq!(error.to_string(), "Runtime error: Null pointer exception");
    }

    #[test]
    fn test_vm_error_memory_error_display() {
        let error = VMError::MemoryError("Out of memory".to_string());
        assert_eq!(error.to_string(), "Memory error: Out of memory");
    }

    #[test]
    fn test_vm_error_from_memory_error_conversion() {
        let memory_error = MemoryError::HeapError("Heap allocation failed".to_string());
        let vm_error: VMError = memory_error.into();
        
        match vm_error {
            VMError::MemoryError(msg) => {
                assert!(msg.contains("Heap allocation failed"));
            },
            _ => panic!("Expected MemoryError variant"),
        }
    }

    #[test]
    fn test_vm_error_debug_formatting() {
        let error = VMError::TypeError("Test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("TypeError"));
        assert!(debug_str.contains("Test error"));
    }

    #[test]
    fn test_vm_result_ok() {
        let result: VMResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_vm_result_error() {
        let result: VMResult<i32> = Err(VMError::StackUnderflow);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            VMError::StackUnderflow => {},
            _ => panic!("Expected StackUnderflow error"),
        }
    }

    #[test]
    fn test_vm_error_equality() {
        let error1 = VMError::DivisionByZero;
        let error2 = VMError::DivisionByZero;
        
        // VMError implements Debug but not PartialEq, so we compare their debug representations
        assert_eq!(format!("{:?}", error1), format!("{:?}", error2));
    }

    #[test]
    fn test_vm_error_clone() {
        let original = VMError::TypeError("Original error".to_string());
        
        // VMError doesn't implement Clone, but we can test that it can be pattern matched
        match original {
            VMError::TypeError(ref msg) => assert_eq!(msg, "Original error"),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_vm_error_large_values() {
        let large_pc = usize::MAX;
        let large_index = u32::MAX;
        let large_bound = usize::MAX;
        
        let pc_error = VMError::InvalidProgramCounter(large_pc);
        let index_error = VMError::InvalidConstantIndex(large_index);
        let bound_error = VMError::IndexOutOfBounds(large_bound);
        
        assert!(pc_error.to_string().contains(&large_pc.to_string()));
        assert!(index_error.to_string().contains(&large_index.to_string()));
        assert!(bound_error.to_string().contains(&large_bound.to_string()));
    }

    #[test]
    fn test_vm_error_empty_strings() {
        let type_error = VMError::TypeError(String::new());
        let arith_error = VMError::ArithmeticError(String::new());
        let op_error = VMError::InvalidOperation(String::new());
        let runtime_error = VMError::RuntimeError(String::new());
        let memory_error = VMError::MemoryError(String::new());
        
        assert_eq!(type_error.to_string(), "Type error: ");
        assert_eq!(arith_error.to_string(), "Arithmetic error: ");
        assert_eq!(op_error.to_string(), "Invalid operation: ");
        assert_eq!(runtime_error.to_string(), "Runtime error: ");
        assert_eq!(memory_error.to_string(), "Memory error: ");
    }

    #[test]
    fn test_vm_error_special_characters() {
        let special_msg = "Error with 🚀 emoji and unicode: ñ, Chinese: 中文, newlines:\ntest";
        let error = VMError::RuntimeError(special_msg.to_string());
        assert!(error.to_string().contains(special_msg));
    }
}