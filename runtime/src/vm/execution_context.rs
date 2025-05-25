use std::sync::Arc;
use crate::memory::MemoryManager;
use crate::bytecode::{BytecodeModule, Constant};
use crate::vm::{Value, VMError, VMResult};

/// Represents the type of an exception handler
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandlerType {
    /// A try-catch handler
    Catch,
    /// A finally handler
    Finally,
}

/// Represents an exception handler
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// The type of handler
    pub handler_type: HandlerType,
    /// Program counter where the handler starts
    pub handler_pc: usize,
    /// Program counter where the protected region ends
    pub end_pc: usize,
    /// The stack depth at the beginning of the try block
    pub stack_depth: usize,
}

/// The execution context for VM instructions
pub struct ExecutionContext {
    module: Arc<BytecodeModule>,
    memory_manager: Arc<MemoryManager>,
    pc: usize,
    stack: Vec<Value>,
    locals: Vec<Value>,
    stack_trace_enabled: bool,
    /// Stack of active exception handlers
    exception_handlers: Vec<ExceptionHandler>,
    /// Current active exception, if any
    current_exception: Option<Value>,
}

impl ExecutionContext {
    pub fn new(module: Arc<BytecodeModule>, memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            module,
            memory_manager,
            pc: 0,
            stack: Vec::with_capacity(256),
            locals: Vec::with_capacity(64),
            stack_trace_enabled: false,
            exception_handlers: Vec::new(),
            current_exception: None,
        }
    }
    
    /// Get the loaded bytecode module
    pub fn module(&self) -> &BytecodeModule {
        &self.module
    }
    
    /// Get a reference to the memory manager
    pub fn memory_manager(&self) -> &MemoryManager {
        &self.memory_manager
    }
    
    /// Get the current program counter
    pub fn pc(&self) -> usize {
        self.pc
    }
    
    /// Set the program counter
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }
    
    /// Increment the program counter
    pub fn increment_pc(&mut self) {
        // In instruction-based execution, just increment by 1
        self.pc += 1;
    }
    
    /// Check if there are more instructions to execute
    pub fn has_more_instructions(&self) -> bool {
        self.pc < self.module.instructions.len()
    }
    
    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) -> VMResult<()> {
        if self.stack_trace_enabled {
            println!("PUSH: {}", value);
        }
        self.stack.push(value);
        Ok(())
    }
    
    /// Pop a value from the stack
    pub fn pop(&mut self) -> VMResult<Value> {
        match self.stack.pop() {
            Some(value) => {
                if self.stack_trace_enabled {
                    println!("POP: {}", value);
                }
                Ok(value)
            },
            None => {
                // Print stack trace for debugging
                println!("Stack underflow occurred at PC: {}", self.pc);
                
                let current_instruction = if self.pc < self.module.instructions.len() {
                    format!("{:?}", self.module.instructions[self.pc])
                } else {
                    "Out of bounds".to_string()
                };
                
                println!("Current instruction: {}", current_instruction);
                Err(VMError::StackUnderflow)
            }
        }
    }
    
    /// Peek at the top value on the stack without removing it
    pub fn peek(&self) -> VMResult<&Value> {
        self.stack.last().ok_or(VMError::StackUnderflow)
    }
    
    /// Store a value in a local variable
    pub fn store_local(&mut self, index: usize, value: Value) -> VMResult<()> {
        if self.stack_trace_enabled {
            println!("STORE LOCAL {}: {}", index, value);
        }
        
        // Ensure the locals vector has enough capacity
        while self.locals.len() <= index {
            self.locals.push(Value::Null);
        }
        
        self.locals[index] = value;
        Ok(())
    }
    
    /// Load a value from a local variable
    pub fn load_local(&self, index: usize) -> VMResult<Value> {
        if index >= self.locals.len() {
            return Err(VMError::InvalidLocalIndex(index as u32));
        }
        
        if self.stack_trace_enabled {
            println!("LOAD LOCAL {}: {}", index, self.locals[index]);
        }
        
        Ok(self.locals[index].clone())
    }
    
    /// Get the constant at the specified index
    pub fn get_constant(&self, index: u32) -> VMResult<Value> {
        let idx = index as usize;
        if idx >= self.module.constants.len() {
            return Err(VMError::InvalidConstantIndex(index));
        }
        
        let constant = &self.module.constants[idx];
        Ok(match constant {
            Constant::Null => Value::Null,
            Constant::Integer(i) => Value::Integer(*i),
            Constant::Float(f) => Value::Float(*f),
            Constant::String(s) => Value::String(s.clone()),
            Constant::Boolean(b) => Value::Boolean(*b),
        })
    }
    
    /// Enable or disable stack trace debugging
    pub fn set_stack_trace(&mut self, enabled: bool) {
        self.stack_trace_enabled = enabled;
    }
    
    /// Get the current stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }
    
    /// Print the current stack for debugging
    pub fn print_stack(&self) {
        println!("STACK (depth={}): ", self.stack.len());
        for (i, value) in self.stack.iter().rev().enumerate() {
            println!("{}: {}", self.stack.len() - i - 1, value);
        }
    }

    /// Push an exception handler onto the handler stack
    pub fn push_exception_handler(&mut self, handler: ExceptionHandler) {
        if self.stack_trace_enabled {
            println!("PUSH HANDLER: {:?} at PC={}", handler.handler_type, handler.handler_pc);
        }
        self.exception_handlers.push(handler);
    }

    /// Pop an exception handler from the handler stack
    pub fn pop_exception_handler(&mut self) -> Option<ExceptionHandler> {
        let handler = self.exception_handlers.pop();
        if self.stack_trace_enabled {
            if let Some(ref h) = handler {
                println!("POP HANDLER: {:?} at PC={}", h.handler_type, h.handler_pc);
            }
        }
        handler
    }

    /// Get the topmost exception handler
    pub fn current_exception_handler(&self) -> Option<&ExceptionHandler> {
        self.exception_handlers.last()
    }

    /// Set the current exception
    pub fn set_exception(&mut self, exception: Value) {
        if self.stack_trace_enabled {
            println!("SETTING EXCEPTION: {}", exception);
        }
        self.current_exception = Some(exception);
    }

    /// Get the current exception
    pub fn get_exception(&self) -> Option<&Value> {
        self.current_exception.as_ref()
    }

    /// Take the current exception, clearing it from the context
    pub fn take_exception(&mut self) -> Option<Value> {
        let exc = self.current_exception.take();
        if self.stack_trace_enabled && exc.is_some() {
            println!("TAKING EXCEPTION: {}", exc.as_ref().unwrap());
        }
        exc
    }

    /// Find the appropriate exception handler for the current PC
    pub fn find_handler(&self, pc: usize) -> Option<&ExceptionHandler> {
        // Search from the newest handler to the oldest
        for handler in self.exception_handlers.iter().rev() {
            // A handler applies if the PC is within the protected region
            if pc < handler.end_pc {
                return Some(handler);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeModule, Instruction, Constant, OpCode};
    use crate::memory::MemoryManager;
    use std::sync::Arc;
    use std::collections::HashMap;

    fn create_test_context() -> ExecutionContext {
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42)];
        
        // Build code_page to match instructions
        let mut code_page = Vec::new();
        let mut address_map = HashMap::new();
        let mut address = 0u32;
        
        for (idx, instruction) in instructions.iter().enumerate() {
            address_map.insert(address, idx);
            
            // Add opcode byte
            code_page.push(instruction.opcode.to_byte());
            address += 1;
            
            // Add operand bytes (each operand is 4 bytes in big-endian format)
            for operand in &instruction.operands {
                code_page.push((*operand >> 24) as u8);
                code_page.push((*operand >> 16) as u8);
                code_page.push((*operand >> 8) as u8);
                code_page.push(*operand as u8);
                address += 4;
            }
        }
        
        let module = Arc::new(BytecodeModule {
            name: "test_module".to_string(),
            instructions,
            constants,
            code_page,
            address_map,
        });
        let memory_manager = Arc::new(MemoryManager::new());
        ExecutionContext::new(module, memory_manager)
    }

    #[test]
    fn test_context_creation() {
        let context = create_test_context();
        assert_eq!(context.pc(), 0);
        assert_eq!(context.stack_depth(), 0);
        assert!(context.has_more_instructions());
    }

    #[test]
    fn test_pc_operations() {
        let mut context = create_test_context();
        assert_eq!(context.pc(), 0);
        
        context.set_pc(5);
        assert_eq!(context.pc(), 5);
        
        context.increment_pc();
        // PC should increment based on opcode at position 5 (if it exists)
        // Since our test module only has 2 instructions, this will be handled gracefully
    }

    #[test]
    fn test_stack_operations() {
        let mut context = create_test_context();
        
        // Test push
        context.push(Value::Integer(10)).unwrap();
        assert_eq!(context.stack_depth(), 1);
        
        context.push(Value::String("test".to_string())).unwrap();
        assert_eq!(context.stack_depth(), 2);
        
        // Test peek
        let peeked = context.peek().unwrap();
        match peeked {
            Value::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
        assert_eq!(context.stack_depth(), 2); // Peek should not change stack size
        
        // Test pop
        let popped = context.pop().unwrap();
        match popped {
            Value::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
        assert_eq!(context.stack_depth(), 1);
        
        let popped2 = context.pop().unwrap();
        match popped2 {
            Value::Integer(i) => assert_eq!(i, 10),
            _ => panic!("Expected integer value"),
        }
        assert_eq!(context.stack_depth(), 0);
    }

    #[test]
    fn test_stack_underflow() {
        let mut context = create_test_context();
        
        // Try to pop from empty stack
        let result = context.pop();
        match result {
            Err(VMError::StackUnderflow) => {},
            _ => panic!("Expected StackUnderflow error"),
        }
        
        // Try to peek at empty stack
        let result = context.peek();
        match result {
            Err(VMError::StackUnderflow) => {},
            _ => panic!("Expected StackUnderflow error"),
        }
    }

    #[test]
    fn test_local_variables() {
        let mut context = create_test_context();
        
        // Store values in local variables
        context.store_local(0, Value::Integer(100)).unwrap();
        context.store_local(1, Value::String("hello".to_string())).unwrap();
        context.store_local(5, Value::Boolean(true)).unwrap(); // Test sparse storage
        
        // Load values from local variables
        let val0 = context.load_local(0).unwrap();
        match val0 {
            Value::Integer(i) => assert_eq!(i, 100),
            _ => panic!("Expected integer value"),
        }
        
        let val1 = context.load_local(1).unwrap();
        match val1 {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string value"),
        }
        
        let val5 = context.load_local(5).unwrap();
        match val5 {
            Value::Boolean(b) => assert_eq!(b, true),
            _ => panic!("Expected boolean value"),
        }
        
        // Test that uninitialized locals return Null
        let val2 = context.load_local(2).unwrap();
        match val2 {
            Value::Null => {},
            _ => panic!("Expected null value for uninitialized local"),
        }
    }

    #[test]
    fn test_invalid_local_index() {
        let context = create_test_context();
        
        // Try to load from index that's too high
        let result = context.load_local(1000);
        match result {
            Err(VMError::InvalidLocalIndex(1000)) => {},
            _ => panic!("Expected InvalidLocalIndex error"),
        }
    }

    #[test]
    fn test_constants() {
        let context = create_test_context();
        
        // Get constant at index 0
        let const_val = context.get_constant(0).unwrap();
        match const_val {
            Value::Integer(i) => assert_eq!(i, 42),
            _ => panic!("Expected integer constant"),
        }
        
        // Try to get constant at invalid index
        let result = context.get_constant(10);
        match result {
            Err(VMError::InvalidConstantIndex(10)) => {},
            _ => panic!("Expected InvalidConstantIndex error"),
        }
    }

    #[test]
    fn test_has_more_instructions() {
        let mut context = create_test_context();
        
        assert!(context.has_more_instructions()); // PC = 0, has 2 instructions
        
        context.set_pc(1);
        assert!(context.has_more_instructions()); // PC = 1, has 2 instructions
        
        context.set_pc(2);
        assert!(!context.has_more_instructions()); // PC = 2, only has 2 instructions (0,1)
        
        context.set_pc(10);
        assert!(!context.has_more_instructions()); // PC way beyond instructions
    }

    #[test]
    fn test_stack_trace_setting() {
        let mut context = create_test_context();
        
        context.set_stack_trace(true);
        // Stack trace behavior is mostly for debugging output, hard to test directly
        // but we can at least verify the setting doesn't crash
        
        context.push(Value::Integer(1)).unwrap();
        context.pop().unwrap();
        context.store_local(0, Value::String("test".to_string())).unwrap();
        context.load_local(0).unwrap();
        
        context.set_stack_trace(false);
    }

    #[test]
    fn test_exception_handlers() {
        let mut context = create_test_context();
        
        // Test empty handler stack
        assert!(context.current_exception_handler().is_none());
        assert!(context.find_handler(0).is_none());
        
        // Push a catch handler
        let catch_handler = ExceptionHandler {
            handler_type: HandlerType::Catch,
            handler_pc: 10,
            end_pc: 20,
            stack_depth: 5,
        };
        context.push_exception_handler(catch_handler.clone());
        
        // Verify handler is on stack
        let current = context.current_exception_handler().unwrap();
        assert_eq!(current.handler_type, HandlerType::Catch);
        assert_eq!(current.handler_pc, 10);
        assert_eq!(current.end_pc, 20);
        assert_eq!(current.stack_depth, 5);
        
        // Test finding handler for PC within range
        let found = context.find_handler(15).unwrap();
        assert_eq!(found.handler_type, HandlerType::Catch);
        assert_eq!(found.handler_pc, 10);
        
        // Test PC outside range
        assert!(context.find_handler(25).is_none());
        
        // Push a finally handler
        let finally_handler = ExceptionHandler {
            handler_type: HandlerType::Finally,
            handler_pc: 30,
            end_pc: 40,
            stack_depth: 3,
        };
        context.push_exception_handler(finally_handler.clone());
        
        // Should find the most recent handler first
        let found = context.find_handler(35).unwrap();
        assert_eq!(found.handler_type, HandlerType::Finally);
        
        // Pop handlers
        let popped = context.pop_exception_handler().unwrap();
        assert_eq!(popped.handler_type, HandlerType::Finally);
        
        let popped2 = context.pop_exception_handler().unwrap();
        assert_eq!(popped2.handler_type, HandlerType::Catch);
        
        // Stack should be empty now
        assert!(context.pop_exception_handler().is_none());
        assert!(context.current_exception_handler().is_none());
    }

    #[test]
    fn test_exception_management() {
        let mut context = create_test_context();
        
        // Test no exception initially
        assert!(context.get_exception().is_none());
        assert!(context.take_exception().is_none());
        
        // Set an exception
        let exception = Value::create_exception("TestError", "Test message");
        context.set_exception(exception.clone());
        
        // Get exception (should still be there)
        let current_exc = context.get_exception().unwrap();
        match current_exc {
            Value::Exception(exc_data) => {
                assert_eq!(exc_data.exception_type, "TestError");
                assert_eq!(exc_data.message, "Test message");
            }
            _ => panic!("Expected exception value"),
        }
        
        // Take exception (should remove it)
        let taken_exc = context.take_exception().unwrap();
        match taken_exc {
            Value::Exception(exc_data) => {
                assert_eq!(exc_data.exception_type, "TestError");
                assert_eq!(exc_data.message, "Test message");
            }
            _ => panic!("Expected exception value"),
        }
        
        // Should be no exception now
        assert!(context.get_exception().is_none());
        assert!(context.take_exception().is_none());
    }

    #[test]
    fn test_handler_type_equality() {
        assert_eq!(HandlerType::Catch, HandlerType::Catch);
        assert_eq!(HandlerType::Finally, HandlerType::Finally);
        assert_ne!(HandlerType::Catch, HandlerType::Finally);
    }

    #[test]
    fn test_multiple_handlers_nested() {
        let mut context = create_test_context();
        
        // Create nested try blocks
        let outer_handler = ExceptionHandler {
            handler_type: HandlerType::Catch,
            handler_pc: 100,
            end_pc: 200,
            stack_depth: 2,
        };
        context.push_exception_handler(outer_handler);
        
        let inner_handler = ExceptionHandler {
            handler_type: HandlerType::Finally,
            handler_pc: 150,
            end_pc: 180,
            stack_depth: 5,
        };
        context.push_exception_handler(inner_handler);
        
        // Should find inner handler first for PC in inner range
        let found = context.find_handler(160).unwrap();
        assert_eq!(found.handler_type, HandlerType::Finally);
        assert_eq!(found.handler_pc, 150);
        
        // For PC in outer but not inner range, should find outer
        let found = context.find_handler(190).unwrap();
        assert_eq!(found.handler_type, HandlerType::Catch);
        assert_eq!(found.handler_pc, 100);
        
        // For PC outside both ranges, should find nothing
        assert!(context.find_handler(250).is_none());
    }

    #[test]
    fn test_print_stack_debug() {
        let mut context = create_test_context();
        
        // Add some values to stack
        context.push(Value::Integer(1)).unwrap();
        context.push(Value::String("test".to_string())).unwrap();
        context.push(Value::Boolean(true)).unwrap();
        
        // This test mainly verifies that print_stack doesn't crash
        // The actual output goes to stdout and is hard to capture in tests
        context.print_stack();
        
        // Enable stack trace and test again
        context.set_stack_trace(true);
        context.print_stack();
    }
}