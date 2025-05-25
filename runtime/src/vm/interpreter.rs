use std::sync::Arc;
use std::collections::HashMap;
use crate::memory::MemoryManager;
use crate::vm::{ExecutionContext, Value, VMError, VMResult};
use crate::bytecode::{BytecodeModule, OpCode};
use crate::vm::execution_context::{ExceptionHandler, HandlerType};

/// The Virtual Machine that executes bytecode instructions
#[derive(Clone)]
pub struct VM {
    memory_manager: Arc<MemoryManager>,
    modules: HashMap<String, Arc<BytecodeModule>>,
    stack_trace_enabled: bool,
}

impl VM {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        Self {
            memory_manager,
            modules: HashMap::new(),
            stack_trace_enabled: false,
        }
    }
    
    /// Load a bytecode module
    pub fn load_module(&mut self, module: BytecodeModule) {
        let name = module.name.clone();
        let module_arc = Arc::new(module);
        self.modules.insert(name, module_arc);
    }
    
    /// Enable or disable stack tracing
    pub fn set_stack_trace(&mut self, enabled: bool) {
        self.stack_trace_enabled = enabled;
    }
    
    /// Execute a loaded module by name
    pub fn execute_module(&self, module_name: &str) -> VMResult<Value> {
        let module = self.modules.get(module_name)
            .ok_or_else(|| VMError::RuntimeError(format!("Module '{}' not found", module_name)))?;
        
        let mut context = ExecutionContext::new(module.clone(), self.memory_manager.clone());
        
        // Apply stack trace setting
        context.set_stack_trace(self.stack_trace_enabled);
        
        // Print initial stack state if tracing is enabled
        if self.stack_trace_enabled {
            println!("\nStarting execution with stack tracing enabled");
            context.print_stack();
        }
        
        self.execute(&mut context)
    }

    /// Execute instructions in an execution context
    pub fn execute(&self, context: &mut ExecutionContext) -> VMResult<Value> {
        while context.has_more_instructions() {
            let pc = context.pc();
            
            // Check if there's an exception that needs to be handled
            if let Some(_) = context.get_exception() {
                // We need to handle the exception case separately to avoid borrowing issues
                // First, find the appropriate handler if any
                let handler_info = {
                    let handler = context.find_handler(pc);
                    // Clone the necessary information to avoid borrowing issues
                    handler.map(|h| (h.handler_type, h.handler_pc, h.stack_depth))
                };
                
                if let Some((handler_type, handler_pc, stack_depth)) = handler_info {
                    // Found a handler - jump to it and pass execution to the handler
                    // Make sure we have the right stack depth before jumping
                    while context.stack_depth() > stack_depth {
                        let _ = context.pop(); // Discard extra stack items
                    }
                    
                    // For catch handlers, push the exception onto the stack
                    if handler_type == HandlerType::Catch {
                        if let Some(exception) = context.take_exception() {
                            context.push(exception)?;
                        }
                    }
                    
                    // Jump to the handler
                    context.set_pc(handler_pc);
                    continue;
                } else {
                    // No handler found, propagate the exception up the call stack
                    // If we get here, the exception is unhandled at the top level
                    if let Some(exception) = context.take_exception() {
                        return Err(VMError::RuntimeError(format!("Unhandled exception: {}", exception)));
                    }
                }
            }
            
            // Get instruction before the match to avoid multiple borrows
            let instruction = &context.module().instructions[pc];
            
            if self.stack_trace_enabled {
                println!("\nExecuting instruction at PC {}: {:?}", pc, instruction);
                context.print_stack();
            }
            
            match instruction.opcode {
                OpCode::Nop => {
                    // Do nothing
                },

                OpCode::Halt => {
                    // Return the top stack value or null if stack is empty
                    return Ok(context.pop().unwrap_or(Value::Null));
                },

                OpCode::Print => {
                    let value = context.pop()?;
                    println!("{}", value);
                },

                OpCode::Push => {
                    let const_idx = instruction.operands.get(0).copied().unwrap_or(0);
                    let value = context.get_constant(const_idx)?;
                    context.push(value)?;
                },
                
                OpCode::Pop => {
                    context.pop()?;
                },
                
                OpCode::Dup => {
                    let value = context.peek()?.clone();
                    context.push(value)?;
                },
                
                OpCode::Swap => {
                    let v1 = context.pop()?;
                    let v2 = context.pop()?;
                    context.push(v1)?;
                    context.push(v2)?;
                },
                
                OpCode::Add => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 + i2))?;
                        },
                        (Value::Float(f1), Value::Float(f2)) => {
                            context.push(Value::Float(f1 + f2))?;
                        },
                        (Value::Integer(i), Value::Float(f)) => {
                            context.push(Value::Float(i as f64 + f))?;
                        },
                        (Value::Float(f), Value::Integer(i)) => {
                            context.push(Value::Float(f + i as f64))?;
                        },
                        (Value::String(s1), Value::String(s2)) => {
                            context.push(Value::String(s1 + &s2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot add {:?} and {:?}", v1_clone, v2_clone))),
                    }
                },
                
                OpCode::Sub => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 - i2))?;
                        },
                        (Value::Float(f1), Value::Float(f2)) => {
                            context.push(Value::Float(f1 - f2))?;
                        },
                        (Value::Integer(i), Value::Float(f)) => {
                            context.push(Value::Float(i as f64 - f))?;
                        },
                        (Value::Float(f), Value::Integer(i)) => {
                            context.push(Value::Float(f - i as f64))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot subtract {:?} from {:?}", v2_clone, v1_clone))),
                    }
                },
                
                OpCode::Mul => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 * i2))?;
                        },
                        (Value::Float(f1), Value::Float(f2)) => {
                            context.push(Value::Float(f1 * f2))?;
                        },
                        (Value::Integer(i), Value::Float(f)) => {
                            context.push(Value::Float(i as f64 * f))?;
                        },
                        (Value::Float(f), Value::Integer(i)) => {
                            context.push(Value::Float(f * i as f64))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot multiply {:?} and {:?}", v1_clone, v2_clone))),
                    }
                },
                
                OpCode::Div => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            if i2 == 0 {
                                return Err(VMError::DivisionByZero);
                            }
                            let q = i1 / i2;
                            let r = i1 % i2;
                            context.push(Value::Integer(r))?;
                            context.push(Value::Integer(q))?;
                        },
                        (Value::Float(f1), Value::Float(f2)) => {
                            if f2 == 0.0 {
                                return Err(VMError::DivisionByZero);
                            }
                            let q = f1 / f2;
                            let r = f1 % f2;
                            context.push(Value::Float(r))?;
                            context.push(Value::Float(q))?;
                        },
                        (Value::Integer(i), Value::Float(f)) => {
                            if f == 0.0 {
                                return Err(VMError::DivisionByZero);
                            }
                            let q = i as f64 / f;
                            let r = i as f64 % f;
                            context.push(Value::Float(r))?;
                            context.push(Value::Float(q))?;
                        },
                        (Value::Float(f), Value::Integer(i)) => {
                            if i == 0 {
                                return Err(VMError::DivisionByZero);
                            }
                            let q = f / i as f64;
                            let r = f % i as f64;
                            context.push(Value::Float(r))?;
                            context.push(Value::Float(q))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot divide {:?} by {:?}", v1_clone, v2_clone))),
                    }
                },
                
                OpCode::Neg => {
                    let value = context.pop()?;
                    let value_clone = value.clone(); // Clone for error reporting
                    
                    match value {
                        Value::Integer(i) => context.push(Value::Integer(-i))?,
                        Value::Float(f) => context.push(Value::Float(-f))?,
                        _ => return Err(VMError::TypeError(format!("Cannot negate {:?}", value_clone))),
                    }
                },
                
                OpCode::BitAnd => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 & i2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform bitwise AND on non-integer values"))),
                    }
                },
                
                OpCode::BitOr => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 | i2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform bitwise OR on non-integer values"))),
                    }
                },
                
                OpCode::BitXor => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            context.push(Value::Integer(i1 ^ i2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform bitwise XOR on non-integer values"))),
                    }
                },
                
                OpCode::BitNot => {
                    let value = context.pop()?;
                    
                    match value {
                        Value::Integer(i) => context.push(Value::Integer(!i))?,
                        _ => return Err(VMError::TypeError(format!("Cannot perform bitwise NOT on non-integer value"))),
                    }
                },
                
                OpCode::ShiftLeft => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            if i2 < 0 {
                                return Err(VMError::InvalidOperation(format!("Cannot shift left by negative amount")));
                            }
                            context.push(Value::Integer(i1 << i2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform shift left on non-integer values"))),
                    }
                },
                
                OpCode::ShiftRight => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            if i2 < 0 {
                                return Err(VMError::InvalidOperation(format!("Cannot shift right by negative amount")));
                            }
                            context.push(Value::Integer(i1 >> i2))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform shift right on non-integer values"))),
                    }
                },
                
                OpCode::RotateLeft => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            if i2 < 0 {
                                return Err(VMError::InvalidOperation(format!("Cannot rotate left by negative amount")));
                            }
                            context.push(Value::Integer((i1 << i2) | (i1 >> (32 - i2))))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform rotate left on non-integer values"))),
                    }
                },
                
                OpCode::RotateRight => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    match (v1, v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => {
                            if i2 < 0 {
                                return Err(VMError::InvalidOperation(format!("Cannot rotate right by negative amount")));
                            }
                            context.push(Value::Integer((i1 >> i2) | (i1 << (32 - i2))))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot perform rotate right on non-integer values"))),
                    }
                },
                
                OpCode::And => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Logical AND operation
                    let result = v1.is_truthy() && v2.is_truthy();
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Or => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Logical OR operation
                    let result = v1.is_truthy() || v2.is_truthy();
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Not => {
                    let value = context.pop()?;
                    
                    // Logical NOT operation
                    let result = !value.is_truthy();
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Eq => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Simple equality check - could be enhanced for more complex comparisons
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 == f2,
                        (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
                        (Value::String(s1), Value::String(s2)) => s1 == s2,
                        (Value::Null, Value::Null) => true,
                        _ => false,
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Neq => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Not equal check
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 != i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 != f2,
                        (Value::Boolean(b1), Value::Boolean(b2)) => b1 != b2,
                        (Value::String(s1), Value::String(s2)) => s1 != s2,
                        (Value::Null, Value::Null) => false,
                        _ => true,
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Lt => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    // Less than comparison
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 < i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 < f2,
                        (Value::Integer(i), Value::Float(f)) => (*i as f64) < *f,
                        (Value::Float(f), Value::Integer(i)) => *f < (*i as f64),
                        (Value::String(s1), Value::String(s2)) => s1 < s2,
                        _ => return Err(VMError::TypeError(format!("Cannot compare {:?} and {:?} with <", v1_clone, v2_clone))),
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Lte => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    // Less than or equal comparison
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 <= i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 <= f2,
                        (Value::Integer(i), Value::Float(f)) => (*i as f64) <= *f,
                        (Value::Float(f), Value::Integer(i)) => *f <= (*i as f64),
                        (Value::String(s1), Value::String(s2)) => s1 <= s2,
                        _ => return Err(VMError::TypeError(format!("Cannot compare {:?} and {:?} with <=", v1_clone, v2_clone))),
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Gt => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    // Greater than comparison
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 > i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 > f2,
                        (Value::Integer(i), Value::Float(f)) => (*i as f64) > *f,
                        (Value::Float(f), Value::Integer(i)) => *f > (*i as f64),
                        (Value::String(s1), Value::String(s2)) => s1 > s2,
                        _ => return Err(VMError::TypeError(format!("Cannot compare {:?} and {:?} with >", v1_clone, v2_clone))),
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Gte => {
                    let v2 = context.pop()?;
                    let v1 = context.pop()?;
                    
                    // Clone values for error reporting
                    let v1_clone = v1.clone();
                    let v2_clone = v2.clone();
                    
                    // Greater than or equal comparison
                    let result = match (&v1, &v2) {
                        (Value::Integer(i1), Value::Integer(i2)) => i1 >= i2,
                        (Value::Float(f1), Value::Float(f2)) => f1 >= f2,
                        (Value::Integer(i), Value::Float(f)) => (*i as f64) >= *f,
                        (Value::Float(f), Value::Integer(i)) => *f >= (*i as f64),
                        (Value::String(s1), Value::String(s2)) => s1 >= s2,
                        _ => return Err(VMError::TypeError(format!("Cannot compare {:?} and {:?} with >=", v1_clone, v2_clone))),
                    };
                    
                    context.push(Value::Boolean(result))?;
                },
                
                OpCode::Jump => {
                    let target = instruction.operands.get(0).copied().unwrap_or(0) as usize;
                    if target >= context.module().instructions.len() {
                        return Err(VMError::InvalidProgramCounter(target));
                    }
                    context.set_pc(target);
                    continue; // Skip pc increment
                },
                
                OpCode::JumpIf => {
                    // Get a copy of the operand first
                    let target_op = instruction.operands.get(0).copied().unwrap_or(0);
                    let condition = context.pop()?;
                    
                    if condition.is_truthy() {
                        let target = target_op as usize;
                        if target >= context.module().instructions.len() {
                            return Err(VMError::InvalidProgramCounter(target));
                        }
                        context.set_pc(target);
                        continue; // Skip pc increment
                    }
                },
                
                OpCode::JumpIfNot => {
                    // Get a copy of the operand first
                    let target_op = instruction.operands.get(0).copied().unwrap_or(0);
                    let condition = context.pop()?;
                    
                    if !condition.is_truthy() {
                        let target = target_op as usize;
                        if target >= context.module().instructions.len() {
                            return Err(VMError::InvalidProgramCounter(target));
                        }
                        context.set_pc(target);
                        continue; // Skip pc increment
                    }
                },
                
                OpCode::Call => {
                    let target = instruction.operands.get(0).copied().unwrap_or(0) as usize;
                    context.increment_pc(); // Next instruction
                    let return_address = context.pc();
                    context.push(Value::ProgramCounter(return_address))?;
                    if target >= context.module().instructions.len() {
                        return Err(VMError::InvalidProgramCounter(target));
                    }
                    context.set_pc(target);
                    continue; // Skip pc increment
                },
                
                OpCode::Return => {
                    let target = context.pop()?;
                    let return_address = match target {
                        Value::ProgramCounter(pc) => pc,
                        _ => return Err(VMError::TypeError(format!("Expected program counter, found {:?}", target))),
                    };
                    if return_address >= context.module().instructions.len() {
                        return Err(VMError::InvalidProgramCounter(return_address));
                    }
                    context.set_pc(return_address);
                    continue; // Skip pc increment
                },

                // Exception handling opcodes
                OpCode::TryBlock => {
                    // Get catch and finally handler locations from operands
                    let catch_pc = instruction.operands.get(0).copied().map(|pc| pc as usize);
                    let finally_pc = instruction.operands.get(1).copied().map(|pc| pc as usize);
                    let end_pc = instruction.operands.get(2).copied().unwrap_or(0) as usize;
                    
                    // First register the finally handler if it exists
                    if let Some(finally_pc_val) = finally_pc {
                        context.push_exception_handler(ExceptionHandler {
                            handler_type: HandlerType::Finally,
                            handler_pc: finally_pc_val,
                            end_pc,
                            stack_depth: context.stack_depth(),
                        });
                    }
                    
                    // Then register the catch handler if it exists
                    if let Some(catch_pc_val) = catch_pc {
                        context.push_exception_handler(ExceptionHandler {
                            handler_type: HandlerType::Catch,
                            handler_pc: catch_pc_val,
                            end_pc: finally_pc.unwrap_or(end_pc),
                            stack_depth: context.stack_depth(),
                        });
                    }
                },
                
                OpCode::CatchBlock => {
                    // Just a marker - any actual handler setup was done by TryBlock
                    // When we reach here, the exception has already been stored in the local variable
                },
                
                OpCode::FinallyBlock => {
                    // Just a marker - any actual handler setup was done by TryBlock
                },
                
                OpCode::EndTry => {
                    // Pop the current handler - normal flow has reached the end of a protected region
                    context.pop_exception_handler();
                },
                
                OpCode::Throw => {
                    // Create an exception from the top of the stack
                    let exception_data = context.pop()?;
                    
                    // Create an exception value
                    let mut exception_value = match exception_data {
                        Value::String(message) => {
                            // Simple string exception
                            Value::create_exception("Error", &message)
                        },
                        Value::Array(array) if array.len() >= 2 => {
                            // Array with [type, message]
                            let typ = match &array[0] {
                                Value::String(s) => s.clone(),
                                _ => "Error".to_string(),
                            };
                            
                            let msg = match &array[1] {
                                Value::String(s) => s.clone(),
                                v => format!("{}", v),
                            };
                            
                            Value::create_exception(&typ, &msg)
                        },
                        v => {
                            // Anything else, convert to string
                            Value::create_exception("Error", &format!("{}", v))
                        }
                    };
                    
                    // Add stack trace information
                    exception_value.add_stack_frame(pc, Some(&context.module().name));
                    
                    // Set as the current exception
                    context.set_exception(exception_value);
                    
                    // We'll handle this exception in the next iteration
                },
                
                OpCode::Rethrow => {
                    // Take the current exception and rethrow it, adding a new stack frame
                    if let Some(mut exception) = context.take_exception() {
                        // Add current location to stack trace
                        exception.add_stack_frame(pc, Some(&context.module().name));
                        
                        // Set it as the current exception again
                        context.set_exception(exception);
                    } else {
                        // No active exception to rethrow
                        return Err(VMError::RuntimeError("Cannot rethrow without an active exception".to_string()));
                    }
                    
                    // We'll handle this exception in the next iteration
                },

                OpCode::StoreLocal => {
                    let idx = instruction.operands.get(0).copied().unwrap_or(0) as usize;
                    let value = context.pop()?;
                    context.store_local(idx, value)?;
                },
                
                OpCode::LoadLocal => {
                    let idx = instruction.operands.get(0).copied().unwrap_or(0) as usize;
                    let value = context.load_local(idx)?;
                    context.push(value)?;
                },

                OpCode::CreateHeap => {
                    let heap_id = context.memory_manager().create_heap()?;
                    context.push(Value::HeapId(heap_id))?;
                },
                
                OpCode::SwitchHeap => {
                    let heap_ref = context.pop()?;
                    let heap_id = match heap_ref {
                        Value::HeapId(id) => id,
                        Value::Integer(i) => i as u32,
                        _ => return Err(VMError::TypeError(format!("Expected heap ID, found {:?}", heap_ref))),
                    };
                    
                    context.memory_manager().switch_heap(heap_id)?;
                },
                
                OpCode::CollectHeap => {
                    let heap_ref = context.pop()?;
                    let heap_id = match heap_ref {
                        Value::HeapId(id) => id,
                        Value::Integer(i) => i as u32,
                        _ => return Err(VMError::TypeError(format!("Expected heap ID, found {:?}", heap_ref))),
                    };
                    
                    context.memory_manager().collect_heap(heap_id)?;
                },

                OpCode::NewArray => {
                    let size = instruction.operands.get(0).copied().unwrap_or(0) as usize;
                    let mut array = Vec::with_capacity(size);

                    // Pop values in reverse order (last item first)
                    for _ in 0..size {
                        array.insert(0, context.pop()?);
                    }

                    context.push(Value::Array(Arc::new(array)))?;
                },

                OpCode::GetElement => {
                    let index_value = context.pop()?;
                    let array_value = context.pop()?;

                    // Convert index to usize
                    let index = match index_value {
                        Value::Integer(i) => {
                            if i < 0 {
                                return Err(VMError::IndexOutOfBounds(i as usize));
                            }
                            i as usize
                        },
                        _ => return Err(VMError::TypeError(format!("Array index must be an integer"))),
                    };

                    // Get the array element
                    match array_value {
                        Value::Array(array) => {
                            if index >= array.len() {
                                return Err(VMError::IndexOutOfBounds(index));
                            }
                            context.push(array[index].clone())?;
                        },
                        Value::String(string) => {
                            let chars: Vec<char> = string.chars().collect();
                            if index >= chars.len() {
                                return Err(VMError::IndexOutOfBounds(index));
                            }
                            let char_str = chars[index].to_string();
                            context.push(Value::String(char_str))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot index into non-array value"))),
                    }
                },

                OpCode::SetElement => {
                    let value = context.pop()?;
                    let index_value = context.pop()?;
                    let array_value = context.pop()?;

                    // Convert index to usize
                    let index = match index_value {
                        Value::Integer(i) => {
                            if i < 0 {
                                return Err(VMError::IndexOutOfBounds(i as usize));
                            }
                            i as usize
                        },
                        _ => return Err(VMError::TypeError(format!("Array index must be an integer"))),
                    };

                    // Set the array element
                    match array_value {
                        Value::Array(array_arc) => {
                            if index >= array_arc.len() {
                                return Err(VMError::IndexOutOfBounds(index));
                            }

                            // We need to clone the array to modify it (Arc is immutable)
                            let mut array = array_arc.as_ref().clone();
                            array[index] = value.clone();

                            // Push the new array back on the stack
                            context.push(Value::Array(Arc::new(array)))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot set element of non-array value"))),
                    }
                },

                OpCode::ArrayLength => {
                    let value = context.pop()?;

                    match value {
                        Value::Array(array) => {
                            context.push(Value::Integer(array.len() as i64))?;
                        },
                        Value::String(string) => {
                            context.push(Value::Integer(string.len() as i64))?;
                        },
                        _ => return Err(VMError::TypeError(format!("Cannot get length of non-array/string value"))),
                    }
                },

                #[allow(unreachable_patterns)]
                _ => { // This is expected to be unreachable but kept for exhaustive checking
                    return Err(VMError::RuntimeError(
                        format!("Unsupported opcode: {:?}", instruction.opcode)
                    ));
                }
            }
            
            context.increment_pc();
        }
        
        // Return the top stack value or null if stack is empty
        if self.stack_trace_enabled {
            println!("\nExecution completed. Final stack:");
            context.print_stack();
        }
        
        Ok(context.pop().unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{BytecodeModule, Instruction, Constant};
    use crate::memory::MemoryManager;
    use std::sync::Arc;
    use std::collections::HashMap;

    fn create_test_vm() -> VM {
        let memory_manager = Arc::new(MemoryManager::new());
        VM::new(memory_manager)
    }

    fn create_test_module_with_instructions(instructions: Vec<Instruction>, constants: Vec<Constant>) -> BytecodeModule {
        let mut code_page = Vec::new();
        let mut address_map = HashMap::new();
        let mut address = 0u32;
        
        // Build code_page and address_map from instructions
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
        
        BytecodeModule {
            name: "test_module".to_string(),
            instructions,
            constants,
            code_page,
            address_map,
        }
    }

    fn assert_value_equals(actual: &Value, expected: &Value) {
        match (actual, expected) {
            (Value::Null, Value::Null) => {},
            (Value::Integer(a), Value::Integer(b)) => assert_eq!(a, b),
            (Value::Float(a), Value::Float(b)) => assert_eq!(a, b),
            (Value::Boolean(a), Value::Boolean(b)) => assert_eq!(a, b),
            (Value::String(a), Value::String(b)) => assert_eq!(a, b),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (av, bv) in a.iter().zip(b.iter()) {
                    assert_value_equals(av, bv);
                }
            },
            _ => panic!("Values do not match: {:?} != {:?}", actual, expected),
        }
    }

    #[test]
    fn test_vm_creation() {
        let vm = create_test_vm();
        assert_eq!(vm.modules.len(), 0);
        assert!(!vm.stack_trace_enabled);
    }

    #[test]
    fn test_load_module() {
        let mut vm = create_test_vm();
        let module = create_test_module_with_instructions(vec![], vec![]);
        
        vm.load_module(module);
        assert_eq!(vm.modules.len(), 1);
        assert!(vm.modules.contains_key("test_module"));
    }

    #[test]
    fn test_execute_module_not_found() {
        let vm = create_test_vm();
        let result = vm.execute_module("nonexistent");
        
        match result {
            Err(VMError::RuntimeError(msg)) => {
                assert!(msg.contains("Module 'nonexistent' not found"));
            }
            _ => panic!("Expected RuntimeError for missing module"),
        }
    }

    #[test]
    fn test_halt_instruction() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(42));
    }

    #[test]
    fn test_halt_empty_stack() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let module = create_test_module_with_instructions(instructions, vec![]);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Null);
    }

    #[test]
    fn test_push_and_pop() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Pop, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::String("test".to_string())];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Null);
    }

    #[test]
    fn test_dup_instruction() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Dup, operands: vec![] },
            Instruction { opcode: OpCode::Pop, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(123)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(123));
    }

    #[test]
    fn test_swap_instruction() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Swap, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(1), Constant::Integer(2)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(1));
    }

    #[test]
    fn test_add_integers() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Add, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(5), Constant::Integer(3)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(8));
    }

    #[test]
    fn test_add_floats() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Add, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Float(2.5), Constant::Float(1.5)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Float(4.0));
    }

    #[test]
    fn test_add_mixed_types() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Add, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(3), Constant::Float(2.5)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Float(5.5));
    }

    #[test]
    fn test_add_strings() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Add, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::String("Hello".to_string()), Constant::String(" World".to_string())];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_add_incompatible_types() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Add, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(5), Constant::Boolean(true)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module");
        
        match result {
            Err(VMError::TypeError(msg)) => {
                assert!(msg.contains("Cannot add"));
            }
            _ => panic!("Expected TypeError for incompatible types"),
        }
    }

    #[test]
    fn test_sub_integers() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Sub, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(10), Constant::Integer(3)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(7));
    }

    #[test]
    fn test_mul_integers() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Mul, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(4), Constant::Integer(3)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(12));
    }

    #[test]
    fn test_div_integers() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Div, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(10), Constant::Integer(3)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        // Division pushes remainder first, then quotient
        assert_value_equals(&result, &Value::Integer(3)); // quotient
    }

    #[test]
    fn test_div_by_zero() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Div, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(10), Constant::Integer(0)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module");
        
        match result {
            Err(VMError::DivisionByZero) => {},
            _ => panic!("Expected DivisionByZero error"),
        }
    }

    #[test]
    fn test_neg_integer() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Neg, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(-42));
    }

    #[test]
    fn test_bitwise_and() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::BitAnd, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(0b1010), Constant::Integer(0b1100)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(0b1000));
    }

    #[test]
    fn test_logical_and_true() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::And, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Boolean(true), Constant::Boolean(true)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Boolean(true));
    }

    #[test]
    fn test_logical_and_false() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::And, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Boolean(true), Constant::Boolean(false)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Boolean(false));
    }

    #[test]
    fn test_equality_same_types() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Eq, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42), Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Boolean(true));
    }

    #[test]
    fn test_equality_different_types() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Eq, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42), Constant::String("42".to_string())];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Boolean(false));
    }

    #[test]
    fn test_comparison_less_than() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Lt, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(5), Constant::Integer(10)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Boolean(true));
    }

    #[test]
    fn test_jump_instruction() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Jump, operands: vec![3] }, // Jump to instruction 3
            Instruction { opcode: OpCode::Push, operands: vec![0] }, // Skipped
            Instruction { opcode: OpCode::Push, operands: vec![0] }, // Skipped  
            Instruction { opcode: OpCode::Push, operands: vec![1] }, // Execute this
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(1), Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(42));
    }

    #[test]
    fn test_jump_if_true() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] }, // Push true
            Instruction { opcode: OpCode::JumpIf, operands: vec![4] }, // Jump to instruction 4 if true
            Instruction { opcode: OpCode::Push, operands: vec![1] }, // Skipped
            Instruction { opcode: OpCode::Halt, operands: vec![] }, // Skipped
            Instruction { opcode: OpCode::Push, operands: vec![2] }, // Execute this
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Boolean(true), Constant::Integer(1), Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(42));
    }

    #[test]
    fn test_jump_if_false() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] }, // Push false
            Instruction { opcode: OpCode::JumpIf, operands: vec![4] }, // Don't jump
            Instruction { opcode: OpCode::Push, operands: vec![1] }, // Execute this
            Instruction { opcode: OpCode::Halt, operands: vec![] },
            Instruction { opcode: OpCode::Push, operands: vec![2] }, // Not executed
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Boolean(false), Constant::Integer(1), Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(1));
    }

    #[test]
    fn test_pop_instruction_stack_underflow() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Pop, operands: vec![] }, // Pop from empty stack
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let module = create_test_module_with_instructions(instructions, vec![]);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module");
        
        match result {
            Err(VMError::StackUnderflow) => {},
            _ => panic!("Expected StackUnderflow error"),
        }
    }

    #[test]
    fn test_invalid_program_counter() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Jump, operands: vec![100] }, // Jump out of bounds
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let module = create_test_module_with_instructions(instructions, vec![]);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module");
        
        match result {
            Err(VMError::InvalidProgramCounter(100)) => {},
            _ => panic!("Expected InvalidProgramCounter error"),
        }
    }

    #[test]
    fn test_set_stack_trace() {
        let mut vm = create_test_vm();
        vm.set_stack_trace(true);
        assert!(vm.stack_trace_enabled);
        
        vm.set_stack_trace(false);
        assert!(!vm.stack_trace_enabled);
    }

    #[test]
    fn test_nop_instruction() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Nop, operands: vec![] }, // Should do nothing
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(42)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(42));
    }

    #[test]
    fn test_create_array() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::Push, operands: vec![2] },
            Instruction { opcode: OpCode::NewArray, operands: vec![3] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(1), Constant::Integer(2), Constant::Integer(3)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_value_equals(&arr[0], &Value::Integer(1));
                assert_value_equals(&arr[1], &Value::Integer(2));
                assert_value_equals(&arr[2], &Value::Integer(3));
            }
            _ => panic!("Expected array result"),
        }
    }

    #[test]
    fn test_array_length() {
        let mut vm = create_test_vm();
        let instructions = vec![
            Instruction { opcode: OpCode::Push, operands: vec![0] },
            Instruction { opcode: OpCode::Push, operands: vec![1] },
            Instruction { opcode: OpCode::NewArray, operands: vec![2] },
            Instruction { opcode: OpCode::ArrayLength, operands: vec![] },
            Instruction { opcode: OpCode::Halt, operands: vec![] },
        ];
        let constants = vec![Constant::Integer(1), Constant::Integer(2)];
        let module = create_test_module_with_instructions(instructions, constants);
        
        vm.load_module(module);
        let result = vm.execute_module("test_module").unwrap();
        
        assert_value_equals(&result, &Value::Integer(2));
    }
}