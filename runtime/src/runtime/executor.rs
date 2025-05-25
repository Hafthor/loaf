use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::memory::MemoryManager;
use crate::vm::{VM, Value};
use crate::runtime::{RuntimeConfig, RuntimeResult};
use crate::bytecode::Parser;

/// The Runtime is the main entry point for using the bytecode VM
#[derive(Clone)]
pub struct Runtime {
    vm: VM,
    memory_manager: Arc<MemoryManager>,
    config: RuntimeConfig,
}

impl Runtime {
    /// Create a new runtime with default configuration
    pub fn new() -> RuntimeResult<Self> {
        Self::with_config(RuntimeConfig::default())
    }
    
    /// Create a new runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> RuntimeResult<Self> {
        // Initialize memory manager
        let memory_manager = Arc::new(MemoryManager::new());
        
        // Initialize VM with memory manager
        let vm = VM::new(memory_manager.clone());
        
        Ok(Self {
            vm,
            memory_manager,
            config,
        })
    }
    
    /// Execute a bytecode file and return the result
    pub fn execute_file<P: AsRef<Path>>(&self, path: P) -> RuntimeResult<Value> {
        // Open and parse the bytecode file
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let module = Parser::parse(&mut reader)?;
        
        if self.config.debug_mode {
            println!("Loaded module: {}", module.name);
            println!("Constants: {}", module.constants.len());
            println!("Instructions: {}", module.instructions.len());
        }
        
        // Store the name for later reference
        let module_name = module.name.clone();
        
        // Load module into VM
        let mut vm = self.vm.clone();
        
        // Set stack tracing if enabled in config
        vm.set_stack_trace(self.config.stack_trace);
        
        // Load and execute the module
        vm.load_module(module);
        let result = vm.execute_module(&module_name)?;
        
        Ok(result)
    }
    
    /// Create a new heap and return its ID
    pub fn create_heap(&self) -> RuntimeResult<u32> {
        let heap_id = self.memory_manager.create_heap()?;
        Ok(heap_id)
    }
    
    /// Switch to a different heap for allocations
    pub fn switch_heap(&self, heap_id: u32) -> RuntimeResult<()> {
        self.memory_manager.switch_heap(heap_id)?;
        Ok(())
    }
    
    /// Get the current heap ID
    pub fn current_heap_id(&self) -> u32 {
        self.memory_manager.current_heap_id()
    }
    
    /// Trigger garbage collection on a specific heap
    pub fn collect_heap(&self, heap_id: u32) -> RuntimeResult<()> {
        self.memory_manager.collect_heap(heap_id)?;
        Ok(())
    }
    
    /// Trigger garbage collection on all heaps
    pub fn collect_all(&self) -> RuntimeResult<()> {
        self.memory_manager.collect_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use crate::vm::Value;
    use crate::utils::write_bytecode;
    use crate::bytecode::{BytecodeModule, Constant};

    /// Helper function to create a test runtime with default config
    fn create_test_runtime() -> Runtime {
        Runtime::new().expect("Failed to create test runtime")
    }

    /// Helper function to create a test runtime with custom config
    fn create_test_runtime_with_config(config: RuntimeConfig) -> Runtime {
        Runtime::with_config(config).expect("Failed to create test runtime with config")
    }

    /// Helper function to create a temporary bytecode file with valid content
    fn create_valid_bytecode_file() -> NamedTempFile {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        // Create a simple test module that won't fail
        let module = create_simple_test_module();
        
        // Write the module to the temporary file
        write_bytecode(&module, temp_file.path()).expect("Failed to write bytecode");
        
        temp_file
    }

    /// Helper function to create a simple test module that won't throw exceptions
    fn create_simple_test_module() -> BytecodeModule {
        use crate::bytecode::{OpCode, Instruction, BytecodeModule};
        
        let mut module = BytecodeModule::new("simple_test");
        
        // Add a simple constant
        module.constants.push(Constant::Integer(42));
        
        // Simple instructions: Push 42, then Halt
        module.instructions.push(Instruction::new(OpCode::Push).with_operand(0)); // Push constant 0 (42)
        module.instructions.push(Instruction::new(OpCode::Halt)); // End execution
        
        module
    }

    /// Helper function to create a temporary bytecode file with invalid content
    fn create_invalid_bytecode_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // Invalid bytecode
        temp_file.write_all(&invalid_data).expect("Failed to write invalid data");
        temp_file.flush().expect("Failed to flush temp file");
        temp_file
    }

    #[test]
    fn test_runtime_new() {
        let runtime = Runtime::new();
        assert!(runtime.is_ok(), "Runtime::new() should succeed");
        
        let runtime = runtime.unwrap();
        assert_eq!(runtime.current_heap_id(), 1, "Default heap ID should be 1");
    }

    #[test]
    fn test_runtime_with_default_config() {
        let config = RuntimeConfig::default();
        let runtime = Runtime::with_config(config);
        assert!(runtime.is_ok(), "Runtime::with_config() should succeed with default config");
    }

    #[test]
    fn test_runtime_with_custom_config() {
        let mut config = RuntimeConfig::default();
        config.debug_mode = true;
        config.stack_trace = true;
        
        let runtime = Runtime::with_config(config);
        assert!(runtime.is_ok(), "Runtime::with_config() should succeed with custom config");
        
        let runtime = runtime.unwrap();
        assert!(runtime.config.debug_mode, "Debug mode should be enabled");
        assert!(runtime.config.stack_trace, "Stack trace should be enabled");
    }

    #[test]
    fn test_execute_file_valid_bytecode() {
        let runtime = create_test_runtime();
        let temp_file = create_valid_bytecode_file();
        
        let result = runtime.execute_file(temp_file.path());
        assert!(result.is_ok(), "execute_file() should succeed with valid bytecode");
        
        // The result should be the integer value 42 from our test bytecode
        let value = result.unwrap();
        if let Value::Integer(i) = value {
            assert_eq!(i, 42, "Result should be the integer 42");
        } else {
            panic!("Expected integer value, got {:?}", value);
        }
    }

    #[test]
    fn test_execute_file_nonexistent_file() {
        let runtime = create_test_runtime();
        let nonexistent_path = "/this/path/does/not/exist.loaf";
        
        let result = runtime.execute_file(nonexistent_path);
        assert!(result.is_err(), "execute_file() should fail with nonexistent file");
    }

    #[test]
    fn test_execute_file_invalid_bytecode() {
        let runtime = create_test_runtime();
        let temp_file = create_invalid_bytecode_file();
        
        let result = runtime.execute_file(temp_file.path());
        assert!(result.is_err(), "execute_file() should fail with invalid bytecode");
    }

    #[test]
    fn test_execute_file_with_debug_mode() {
        let mut config = RuntimeConfig::default();
        config.debug_mode = true;
        let runtime = create_test_runtime_with_config(config);
        let temp_file = create_valid_bytecode_file();
        
        let result = runtime.execute_file(temp_file.path());
        assert!(result.is_ok(), "execute_file() should succeed with debug mode enabled");
    }

    #[test]
    fn test_execute_file_with_stack_trace() {
        let mut config = RuntimeConfig::default();
        config.stack_trace = true;
        let runtime = create_test_runtime_with_config(config);
        let temp_file = create_valid_bytecode_file();
        
        let result = runtime.execute_file(temp_file.path());
        assert!(result.is_ok(), "execute_file() should succeed with stack trace enabled");
    }

    #[test]
    fn test_create_heap() {
        let runtime = create_test_runtime();
        
        let heap_id = runtime.create_heap();
        assert!(heap_id.is_ok(), "create_heap() should succeed");
        
        let heap_id = heap_id.unwrap();
        assert!(heap_id > 0, "New heap ID should be greater than 0");
    }

    #[test]
    fn test_create_multiple_heaps() {
        let runtime = create_test_runtime();
        
        let heap1 = runtime.create_heap().unwrap();
        let heap2 = runtime.create_heap().unwrap();
        let heap3 = runtime.create_heap().unwrap();
        
        assert_ne!(heap1, heap2, "Heap IDs should be unique");
        assert_ne!(heap2, heap3, "Heap IDs should be unique");
        assert_ne!(heap1, heap3, "Heap IDs should be unique");
        
        // IDs should be sequential
        assert_eq!(heap2, heap1 + 1, "Heap IDs should be sequential");
        assert_eq!(heap3, heap2 + 1, "Heap IDs should be sequential");
    }

    #[test]
    fn test_current_heap_id_default() {
        let runtime = create_test_runtime();
        let current_id = runtime.current_heap_id();
        assert_eq!(current_id, 1, "Default heap ID should be 1");
    }

    #[test]
    fn test_switch_heap_valid() {
        let runtime = create_test_runtime();
        
        // Create a new heap
        let new_heap_id = runtime.create_heap().unwrap();
        
        // Switch to the new heap
        let result = runtime.switch_heap(new_heap_id);
        assert!(result.is_ok(), "switch_heap() should succeed with valid heap ID");
        
        // Verify the current heap changed
        assert_eq!(runtime.current_heap_id(), new_heap_id, "Current heap should be the switched heap");
    }

    #[test]
    fn test_switch_heap_invalid() {
        let runtime = create_test_runtime();
        
        // Try to switch to a non-existent heap
        let result = runtime.switch_heap(999);
        assert!(result.is_err(), "switch_heap() should fail with invalid heap ID");
        
        // Current heap should remain unchanged
        assert_eq!(runtime.current_heap_id(), 1, "Current heap should remain unchanged after failed switch");
    }

    #[test]
    fn test_switch_heap_back_to_default() {
        let runtime = create_test_runtime();
        
        // Create and switch to a new heap
        let new_heap_id = runtime.create_heap().unwrap();
        runtime.switch_heap(new_heap_id).unwrap();
        
        // Switch back to default heap
        let result = runtime.switch_heap(1);
        assert!(result.is_ok(), "switch_heap() should succeed switching back to default heap");
        assert_eq!(runtime.current_heap_id(), 1, "Current heap should be default heap");
    }

    #[test]
    fn test_collect_heap_valid() {
        let runtime = create_test_runtime();
        
        // Create a new heap
        let heap_id = runtime.create_heap().unwrap();
        
        // Collect the specific heap
        let result = runtime.collect_heap(heap_id);
        assert!(result.is_ok(), "collect_heap() should succeed with valid heap ID");
    }

    #[test]
    fn test_collect_heap_default() {
        let runtime = create_test_runtime();
        
        // Collect the default heap
        let result = runtime.collect_heap(1);
        assert!(result.is_ok(), "collect_heap() should succeed with default heap");
    }

    #[test]
    fn test_collect_heap_invalid() {
        let runtime = create_test_runtime();
        
        // Try to collect a non-existent heap
        let result = runtime.collect_heap(999);
        assert!(result.is_err(), "collect_heap() should fail with invalid heap ID");
    }

    #[test]
    fn test_collect_all_heaps() {
        let runtime = create_test_runtime();
        
        // Create multiple heaps
        runtime.create_heap().unwrap();
        runtime.create_heap().unwrap();
        runtime.create_heap().unwrap();
        
        // Collect all heaps
        let result = runtime.collect_all();
        assert!(result.is_ok(), "collect_all() should succeed");
    }

    #[test]
    fn test_collect_all_heaps_empty() {
        let runtime = create_test_runtime();
        
        // Collect all heaps when only default heap exists
        let result = runtime.collect_all();
        assert!(result.is_ok(), "collect_all() should succeed even with only default heap");
    }

    #[test]
    fn test_runtime_clone() {
        let runtime = create_test_runtime();
        let runtime_clone = runtime.clone();
        
        // Both runtimes should have the same current heap
        assert_eq!(runtime.current_heap_id(), runtime_clone.current_heap_id());
        
        // Creating a heap in one should be visible in the other (shared memory manager)
        let new_heap_id = runtime.create_heap().unwrap();
        let switch_result = runtime_clone.switch_heap(new_heap_id);
        assert!(switch_result.is_ok(), "Cloned runtime should be able to switch to heap created by original");
    }

    #[test]
    fn test_heap_operations_sequence() {
        let runtime = create_test_runtime();
        
        // Test a sequence of heap operations
        assert_eq!(runtime.current_heap_id(), 1);
        
        let heap1 = runtime.create_heap().unwrap();
        let heap2 = runtime.create_heap().unwrap();
        
        runtime.switch_heap(heap1).unwrap();
        assert_eq!(runtime.current_heap_id(), heap1);
        
        runtime.switch_heap(heap2).unwrap();
        assert_eq!(runtime.current_heap_id(), heap2);
        
        runtime.collect_heap(heap1).unwrap();
        runtime.collect_heap(heap2).unwrap();
        runtime.collect_all().unwrap();
        
        runtime.switch_heap(1).unwrap();
        assert_eq!(runtime.current_heap_id(), 1);
    }

    #[test]
    fn test_multiple_file_executions() {
        let runtime = create_test_runtime();
        
        // Execute the same file multiple times
        let temp_file = create_valid_bytecode_file();
        
        for i in 0..3 {
            let result = runtime.execute_file(temp_file.path());
            assert!(result.is_ok(), "execute_file() should succeed on iteration {}", i);
            
            if let Value::Integer(val) = result.unwrap() {
                assert_eq!(val, 42, "Result should be consistent across executions");
            }
        }
    }

    #[test]
    fn test_file_execution_with_heap_operations() {
        let runtime = create_test_runtime();
        let temp_file = create_valid_bytecode_file();
        
        // Execute file, then do heap operations, then execute again
        let result1 = runtime.execute_file(temp_file.path());
        assert!(result1.is_ok());
        
        let heap_id = runtime.create_heap().unwrap();
        runtime.switch_heap(heap_id).unwrap();
        runtime.collect_heap(heap_id).unwrap();
        
        let result2 = runtime.execute_file(temp_file.path());
        assert!(result2.is_ok());
        
        // Results should be the same
        if let (Value::Integer(val1), Value::Integer(val2)) = (result1.unwrap(), result2.unwrap()) {
            assert_eq!(val1, val2, "Results should be consistent");
        }
    }

    #[test]
    fn test_config_preservation() {
        let mut config = RuntimeConfig::default();
        config.debug_mode = true;
        config.stack_trace = true;
        
        let runtime = Runtime::with_config(config).unwrap();
        
        // Config should be preserved
        assert!(runtime.config.debug_mode);
        assert!(runtime.config.stack_trace);
        
        // Config should be preserved after clone
        let runtime_clone = runtime.clone();
        assert!(runtime_clone.config.debug_mode);
        assert!(runtime_clone.config.stack_trace);
    }

    #[test]
    fn test_concurrent_runtime_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let runtime = Arc::new(create_test_runtime());
        let mut handles = vec![];
        
        // Spawn multiple threads doing different operations
        for _ in 0..5 {
            let runtime_clone = Arc::clone(&runtime);
            let handle = thread::spawn(move || {
                // Each thread creates a heap and does some operations
                let heap_id = runtime_clone.create_heap().unwrap();
                runtime_clone.switch_heap(heap_id).unwrap();
                runtime_clone.collect_heap(heap_id).unwrap();
                heap_id
            });
            handles.push(handle);
        }
        
        // Wait for all threads and collect results
        let mut heap_ids = vec![];
        for handle in handles {
            let heap_id = handle.join().unwrap();
            heap_ids.push(heap_id);
        }
        
        // All heap IDs should be unique
        heap_ids.sort();
        for i in 1..heap_ids.len() {
            assert_ne!(heap_ids[i-1], heap_ids[i], "Heap IDs should be unique");
        }
        
        // Should be able to collect all heaps
        runtime.collect_all().unwrap();
    }
}