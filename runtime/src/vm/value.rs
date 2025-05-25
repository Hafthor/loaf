use std::fmt;
use std::sync::Arc;
use crate::memory::ObjectReference;

/// Represents exception data for structured exception handling
#[derive(Clone, Debug)]
pub struct ExceptionData {
    /// The exception type (as a string)
    pub exception_type: String,
    /// The exception message
    pub message: String,
    /// Optional stack trace information
    pub stack_trace: Vec<StackFrame>,
}

/// Represents a stack frame for exception stack traces
#[derive(Clone, Debug)]
pub struct StackFrame {
    /// The program counter where the exception occurred
    pub pc: usize,
    /// Optional module name
    pub module: Option<String>,
}

/// Represents a value in the VM
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Object(ObjectReference),
    Array(Arc<Vec<Value>>),
    HeapId(u32),
    ProgramCounter(usize),
    Exception(ExceptionData),
}

impl Value {
    /// Create a new exception value
    pub fn create_exception(exception_type: &str, message: &str) -> Self {
        Value::Exception(ExceptionData {
            exception_type: exception_type.to_string(),
            message: message.to_string(),
            stack_trace: Vec::new(),
        })
    }

    /// Check if the value is an exception
    pub fn is_exception(&self) -> bool {
        matches!(self, Value::Exception(_))
    }

    /// Get the exception data if this is an exception
    pub fn as_exception(&self) -> Option<&ExceptionData> {
        match self {
            Value::Exception(exc) => Some(exc),
            _ => None,
        }
    }

    /// Add a stack frame to an exception's stack trace
    pub fn add_stack_frame(&mut self, pc: usize, module_name: Option<&str>) {
        if let Value::Exception(exc) = self {
            exc.stack_trace.push(StackFrame {
                pc,
                module: module_name.map(|s| s.to_string()),
            });
        }
    }

    /// Checks if the value is truthy (used in conditionals)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0 && !f.is_nan(),
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Object(_) => true,
            Value::Array(arr) => !arr.is_empty(),
            Value::HeapId(_) => true,
            Value::ProgramCounter(_) => true,
            Value::Exception(_) => true,
        }
    }
    
    /// Get the value as an integer or error
    pub fn as_integer(&self) -> Result<i64, String> {
        match self {
            Value::Integer(i) => Ok(*i),
            Value::Float(f) => Ok(*f as i64),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(format!("Cannot convert {} to integer", self)),
        }
    }
    
    /// Get the value as a float or error
    pub fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Integer(i) => Ok(*i as f64),
            Value::Float(f) => Ok(*f),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(format!("Cannot convert {} to float", self)),
        }
    }
    
    /// Get the value as a boolean or error
    pub fn as_boolean(&self) -> Result<bool, String> {
        Ok(self.is_truthy())
    }
    
    /// Get the value as a string or error
    pub fn as_string(&self) -> Result<String, String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Cannot convert {} to string", self)),
        }
    }
    
    /// Get the value as an array or error
    pub fn as_array(&self) -> Result<Arc<Vec<Value>>, String> {
        match self {
            Value::Array(arr) => Ok(arr.clone()),
            _ => Err(format!("Cannot convert {} to array", self)),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Object(obj) => write!(f, "<object:ref={:?}>", obj),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Value::HeapId(id) => write!(f, "<heap:{}>", id),
            Value::ProgramCounter(pc) => write!(f, "<pc:{}>", pc),
            Value::Exception(exc) => write!(f, "Exception: {}({})", exc.exception_type, exc.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_null_display() {
        let value = Value::Null;
        assert_eq!(value.to_string(), "null");
    }

    #[test]
    fn test_value_integer_display() {
        let value = Value::Integer(42);
        assert_eq!(value.to_string(), "42");
        
        let negative_value = Value::Integer(-123);
        assert_eq!(negative_value.to_string(), "-123");
    }

    #[test]
    fn test_value_float_display() {
        let value = Value::Float(3.14);
        assert_eq!(value.to_string(), "3.14");
        
        let negative_value = Value::Float(-2.718);
        assert_eq!(negative_value.to_string(), "-2.718");
    }

    #[test]
    fn test_value_boolean_display() {
        let true_value = Value::Boolean(true);
        let false_value = Value::Boolean(false);
        assert_eq!(true_value.to_string(), "true");
        assert_eq!(false_value.to_string(), "false");
    }

    #[test]
    fn test_value_string_display() {
        let value = Value::String("hello world".to_string());
        assert_eq!(value.to_string(), "\"hello world\"");
        
        let empty_value = Value::String(String::new());
        assert_eq!(empty_value.to_string(), "\"\"");
    }

    #[test]
    fn test_value_object_display() {
        let obj_ref = ObjectReference::new(1, 123);
        let value = Value::Object(obj_ref);
        let display = value.to_string();
        assert!(display.contains("<object:ref="));
        assert!(display.contains("123"));
    }

    #[test]
    fn test_value_array_display() {
        let arr = vec![Value::Integer(1), Value::String("test".to_string()), Value::Boolean(true)];
        let value = Value::Array(Arc::new(arr));
        let display = value.to_string();
        assert_eq!(display, "[1, \"test\", true]");
        
        let empty_arr = Value::Array(Arc::new(vec![]));
        assert_eq!(empty_arr.to_string(), "[]");
    }

    #[test]
    fn test_value_heap_id_display() {
        let value = Value::HeapId(42);
        assert_eq!(value.to_string(), "<heap:42>");
    }

    #[test]
    fn test_value_program_counter_display() {
        let value = Value::ProgramCounter(100);
        assert_eq!(value.to_string(), "<pc:100>");
    }

    #[test]
    fn test_value_exception_display() {
        let exc_data = ExceptionData {
            exception_type: "TypeError".to_string(),
            message: "Invalid operation".to_string(),
            stack_trace: vec![],
        };
        let value = Value::Exception(exc_data);
        assert_eq!(value.to_string(), "Exception: TypeError(Invalid operation)");
    }

    #[test]
    fn test_value_create_exception() {
        let exc = Value::create_exception("NullPointerException", "Object is null");
        
        assert!(exc.is_exception());
        
        let exc_data = exc.as_exception().unwrap();
        assert_eq!(exc_data.exception_type, "NullPointerException");
        assert_eq!(exc_data.message, "Object is null");
        assert_eq!(exc_data.stack_trace.len(), 0);
    }

    #[test]
    fn test_value_is_exception() {
        let exc = Value::create_exception("Error", "message");
        let int_val = Value::Integer(42);
        let str_val = Value::String("test".to_string());
        
        assert!(exc.is_exception());
        assert!(!int_val.is_exception());
        assert!(!str_val.is_exception());
    }

    #[test]
    fn test_value_as_exception() {
        let exc = Value::create_exception("TestError", "test message");
        let int_val = Value::Integer(42);
        
        assert!(exc.as_exception().is_some());
        assert!(int_val.as_exception().is_none());
        
        let exc_data = exc.as_exception().unwrap();
        assert_eq!(exc_data.exception_type, "TestError");
        assert_eq!(exc_data.message, "test message");
    }

    #[test]
    fn test_value_add_stack_frame() {
        let mut exc = Value::create_exception("Error", "message");
        let mut int_val = Value::Integer(42);
        
        exc.add_stack_frame(100, Some("main"));
        exc.add_stack_frame(200, None);
        
        // Adding stack frame to non-exception should do nothing
        int_val.add_stack_frame(50, Some("test"));
        
        let exc_data = exc.as_exception().unwrap();
        assert_eq!(exc_data.stack_trace.len(), 2);
        
        assert_eq!(exc_data.stack_trace[0].pc, 100);
        assert_eq!(exc_data.stack_trace[0].module, Some("main".to_string()));
        
        assert_eq!(exc_data.stack_trace[1].pc, 200);
        assert_eq!(exc_data.stack_trace[1].module, None);
        
        // int_val should not be modified
        assert!(!int_val.is_exception());
    }

    #[test]
    fn test_value_is_truthy() {
        // Falsy values
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Integer(0).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
        assert!(!Value::Float(f64::NAN).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(!Value::String(String::new()).is_truthy());
        assert!(!Value::Array(Arc::new(vec![])).is_truthy());
        
        // Truthy values
        assert!(Value::Integer(1).is_truthy());
        assert!(Value::Integer(-1).is_truthy());
        assert!(Value::Float(1.0).is_truthy());
        assert!(Value::Float(-1.0).is_truthy());
        assert!(Value::Boolean(true).is_truthy());
        assert!(Value::String("hello".to_string()).is_truthy());
        assert!(Value::Array(Arc::new(vec![Value::Null])).is_truthy());
        assert!(Value::Object(ObjectReference::new(1, 100)).is_truthy());
        assert!(Value::HeapId(1).is_truthy());
        assert!(Value::ProgramCounter(0).is_truthy());
        assert!(Value::create_exception("Error", "message").is_truthy());
    }

    #[test]
    fn test_value_as_integer() {
        assert_eq!(Value::Integer(42).as_integer().unwrap(), 42);
        assert_eq!(Value::Float(3.14).as_integer().unwrap(), 3);
        assert_eq!(Value::Boolean(true).as_integer().unwrap(), 1);
        assert_eq!(Value::Boolean(false).as_integer().unwrap(), 0);
        
        assert!(Value::String("test".to_string()).as_integer().is_err());
        assert!(Value::Null.as_integer().is_err());
    }

    #[test]
    fn test_value_as_float() {
        assert_eq!(Value::Integer(42).as_float().unwrap(), 42.0);
        assert_eq!(Value::Float(3.14).as_float().unwrap(), 3.14);
        assert_eq!(Value::Boolean(true).as_float().unwrap(), 1.0);
        assert_eq!(Value::Boolean(false).as_float().unwrap(), 0.0);
        
        assert!(Value::String("test".to_string()).as_float().is_err());
        assert!(Value::Null.as_float().is_err());
    }

    #[test]
    fn test_value_as_boolean() {
        assert_eq!(Value::Boolean(true).as_boolean().unwrap(), true);
        assert_eq!(Value::Boolean(false).as_boolean().unwrap(), false);
        assert_eq!(Value::Integer(1).as_boolean().unwrap(), true);
        assert_eq!(Value::Integer(0).as_boolean().unwrap(), false);
        assert_eq!(Value::String("test".to_string()).as_boolean().unwrap(), true);
        assert_eq!(Value::String(String::new()).as_boolean().unwrap(), false);
        assert_eq!(Value::Null.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_value_as_string() {
        assert_eq!(Value::String("hello".to_string()).as_string().unwrap(), "hello");
        
        assert!(Value::Integer(42).as_string().is_err());
        assert!(Value::Boolean(true).as_string().is_err());
        assert!(Value::Null.as_string().is_err());
    }

    #[test]
    fn test_value_as_array() {
        let arr = vec![Value::Integer(1), Value::Integer(2)];
        let array_val = Value::Array(Arc::new(arr.clone()));
        
        let result = array_val.as_array().unwrap();
        assert_eq!(result.len(), 2);
        
        assert!(Value::Integer(42).as_array().is_err());
        assert!(Value::String("test".to_string()).as_array().is_err());
    }

    #[test]
    fn test_value_clone() {
        let original = Value::String("test".to_string());
        let cloned = original.clone();
        
        match (original, cloned) {
            (Value::String(s1), Value::String(s2)) => assert_eq!(s1, s2),
            _ => panic!("Clone failed"),
        }
    }

    #[test]
    fn test_value_debug_formatting() {
        let value = Value::Integer(42);
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("Integer"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_exception_data_clone() {
        let exc_data = ExceptionData {
            exception_type: "TestError".to_string(),
            message: "test message".to_string(),
            stack_trace: vec![StackFrame { pc: 100, module: Some("main".to_string()) }],
        };
        
        let cloned = exc_data.clone();
        assert_eq!(exc_data.exception_type, cloned.exception_type);
        assert_eq!(exc_data.message, cloned.message);
        assert_eq!(exc_data.stack_trace.len(), cloned.stack_trace.len());
    }

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame {
            pc: 123,
            module: Some("test_module".to_string()),
        };
        
        assert_eq!(frame.pc, 123);
        assert_eq!(frame.module, Some("test_module".to_string()));
        
        let frame_no_module = StackFrame {
            pc: 456,
            module: None,
        };
        
        assert_eq!(frame_no_module.pc, 456);
        assert_eq!(frame_no_module.module, None);
    }

    #[test]
    fn test_value_large_numbers() {
        let large_int = Value::Integer(i64::MAX);
        let small_int = Value::Integer(i64::MIN);
        let large_float = Value::Float(f64::MAX);
        let small_float = Value::Float(f64::MIN);
        
        assert!(large_int.to_string().contains(&i64::MAX.to_string()));
        assert!(small_int.to_string().contains(&i64::MIN.to_string()));
        
        // f64::MAX and f64::MIN can be displayed in different formats (scientific or decimal)
        // depending on the system, so we check that the string representation matches the actual f64 value
        assert_eq!(large_float.to_string(), f64::MAX.to_string());
        assert_eq!(small_float.to_string(), f64::MIN.to_string());
    }

    #[test]
    fn test_value_special_float_values() {
        let infinity = Value::Float(f64::INFINITY);
        let neg_infinity = Value::Float(f64::NEG_INFINITY);
        let nan = Value::Float(f64::NAN);
        
        assert!(infinity.is_truthy());
        assert!(neg_infinity.is_truthy());
        assert!(!nan.is_truthy()); // NaN is falsy
        
        assert_eq!(infinity.as_float().unwrap(), f64::INFINITY);
        assert_eq!(neg_infinity.as_float().unwrap(), f64::NEG_INFINITY);
        assert!(nan.as_float().unwrap().is_nan());
    }

    #[test]
    fn test_value_nested_arrays() {
        let inner_array = Arc::new(vec![Value::Integer(1), Value::Integer(2)]);
        let outer_array = Arc::new(vec![
            Value::Array(inner_array),
            Value::String("test".to_string())
        ]);
        let nested_value = Value::Array(outer_array);
        
        let display = nested_value.to_string();
        assert!(display.contains("[1, 2]"));
        assert!(display.contains("\"test\""));
    }

    #[test]
    fn test_value_unicode_strings() {
        let unicode_str = "Hello 🌍 世界 español";
        let value = Value::String(unicode_str.to_string());
        assert_eq!(value.to_string(), format!("\"{}\"", unicode_str));
        assert_eq!(value.as_string().unwrap(), unicode_str);
    }

    #[test]
    fn test_value_conversion_error_messages() {
        let string_val = Value::String("test".to_string());
        
        let int_err = string_val.as_integer().unwrap_err();
        assert!(int_err.contains("Cannot convert"));
        assert!(int_err.contains("to integer"));
        
        let float_err = string_val.as_float().unwrap_err();
        assert!(float_err.contains("Cannot convert"));
        assert!(float_err.contains("to float"));
        
        let array_err = string_val.as_array().unwrap_err();
        assert!(array_err.contains("Cannot convert"));
        assert!(array_err.contains("to array"));
    }
}