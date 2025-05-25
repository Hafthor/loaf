use crate::codegen::{BytecodeProgram, EndpointInfo, Instruction, Value};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// HTTP request representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub query_params: HashMap<String, String>,
}

/// HTTP response representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Isolated heap for endpoint execution
#[derive(Debug, Clone)]
pub struct IsolatedHeap {
    pub id: String,
    pub variables: HashMap<String, Value>,
    pub promises: HashMap<String, Promise>,
    pub stack: Vec<Value>,
}

/// Promise state in the runtime
#[derive(Debug, Clone)]
pub struct Promise {
    pub id: String,
    pub resolved: bool,
    pub value: Option<Value>,
    pub dependents: Vec<String>,
}

/// Virtual machine for executing bytecode
pub struct VirtualMachine {
    pub program: BytecodeProgram,
    pub global_heap: Arc<RwLock<IsolatedHeap>>,
    pub endpoint_handlers: HashMap<String, EndpointInfo>,
}

impl IsolatedHeap {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            variables: HashMap::new(),
            promises: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| anyhow!("Stack underflow"))
    }

    pub fn peek(&self) -> Result<&Value> {
        self.stack.last().ok_or_else(|| anyhow!("Stack is empty"))
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn create_promise(&mut self, id: String) -> Result<()> {
        let promise = Promise {
            id: id.clone(),
            resolved: false,
            value: None,
            dependents: Vec::new(),
        };
        self.promises.insert(id, promise);
        Ok(())
    }

    pub fn resolve_promise(&mut self, id: &str, value: Value) -> Result<()> {
        if let Some(promise) = self.promises.get_mut(id) {
            promise.resolved = true;
            promise.value = Some(value);
            Ok(())
        } else {
            Err(anyhow!("Promise not found: {}", id))
        }
    }

    pub fn is_promise_resolved(&self, id: &str) -> bool {
        self.promises.get(id).map_or(false, |p| p.resolved)
    }

    pub fn get_promise_value(&self, id: &str) -> Option<&Value> {
        self.promises.get(id).and_then(|p| p.value.as_ref())
    }
}

impl VirtualMachine {
    pub fn new(program: BytecodeProgram) -> Self {
        let endpoint_handlers = program.endpoints.clone();
        
        Self {
            program,
            global_heap: Arc::new(RwLock::new(IsolatedHeap::new())),
            endpoint_handlers,
        }
    }

    /// Execute bytecode starting from a given instruction pointer
    pub async fn execute(&self, start_ip: usize, heap: &mut IsolatedHeap) -> Result<Option<Value>> {
        let mut ip = start_ip;
        
        while ip < self.program.instructions.len() {
            let instruction = &self.program.instructions[ip];
            
            match instruction {
                Instruction::LoadConstant(value) => {
                    heap.push(value.clone());
                }

                Instruction::LoadVariable(name) => {
                    if let Some(value) = heap.get_variable(name) {
                        heap.push(value.clone());
                    } else {
                        return Err(anyhow!("Variable not found: {}", name));
                    }
                }

                Instruction::StoreVariable(name) => {
                    let value = heap.pop()?;
                    heap.set_variable(name.clone(), value);
                }

                Instruction::Add => {
                    let b = heap.pop()?;
                    let a = heap.pop()?;
                    let result = self.add_values(&a, &b)?;
                    heap.push(result);
                }

                Instruction::Subtract => {
                    let b = heap.pop()?;
                    let a = heap.pop()?;
                    let result = self.subtract_values(&a, &b)?;
                    heap.push(result);
                }

                Instruction::Multiply => {
                    let b = heap.pop()?;
                    let a = heap.pop()?;
                    let result = self.multiply_values(&a, &b)?;
                    heap.push(result);
                }

                Instruction::Divide => {
                    let b = heap.pop()?;
                    let a = heap.pop()?;
                    let result = self.divide_values(&a, &b)?;
                    heap.push(result);
                }

                Instruction::CreateObject => {
                    heap.push(Value::Object(HashMap::new()));
                }

                Instruction::SetProperty(key) => {
                    let value = heap.pop()?;
                    let mut object = heap.pop()?;
                    
                    if let Value::Object(ref mut map) = object {
                        map.insert(key.clone(), value);
                        heap.push(object);
                    } else {
                        return Err(anyhow!("Cannot set property on non-object"));
                    }
                }

                Instruction::GetProperty(key) => {
                    let object = heap.pop()?;
                    
                    if let Value::Object(ref map) = object {
                        let value = map.get(key).cloned().unwrap_or(Value::Null);
                        heap.push(value);
                    } else {
                        return Err(anyhow!("Cannot get property from non-object"));
                    }
                }

                Instruction::CreateArray => {
                    heap.push(Value::Array(Vec::new()));
                }

                Instruction::AppendArray => {
                    let value = heap.pop()?;
                    let mut array = heap.pop()?;
                    
                    if let Value::Array(ref mut vec) = array {
                        vec.push(value);
                        heap.push(array);
                    } else {
                        return Err(anyhow!("Cannot append to non-array"));
                    }
                }

                Instruction::CreatePromise(id) => {
                    heap.create_promise(id.clone())?;
                    heap.push(Value::Promise(id.clone()));
                }

                Instruction::ResolvePromise(id) => {
                    let value = heap.pop()?;
                    heap.resolve_promise(id, value)?;
                }

                Instruction::AwaitPromise(id) => {
                    if heap.is_promise_resolved(id) {
                        if let Some(value) = heap.get_promise_value(id) {
                            heap.push(value.clone());
                        } else {
                            heap.push(Value::Null);
                        }
                    } else {
                        // In a real implementation, this would suspend execution
                        // For now, we'll return null for unresolved promises
                        heap.push(Value::Null);
                    }
                }

                Instruction::HttpCall { method, url, body } => {
                    // Simulate HTTP call - in a real implementation this would make actual HTTP requests
                    let response = self.simulate_http_call(method, url, body.as_deref()).await?;
                    heap.push(Value::String(response));
                }

                Instruction::Return => {
                    return Ok(heap.stack.last().cloned());
                }

                Instruction::Jump(target) => {
                    ip = *target;
                    continue;
                }

                Instruction::JumpIfFalse(target) => {
                    let condition = heap.pop()?;
                    if !self.is_truthy(&condition) {
                        ip = *target;
                        continue;
                    }
                }

                Instruction::Duplicate => {
                    let value = heap.peek()?.clone();
                    heap.push(value);
                }

                Instruction::Pop => {
                    heap.pop()?;
                }

                Instruction::Swap => {
                    if heap.stack.len() < 2 {
                        return Err(anyhow!("Not enough values on stack to swap"));
                    }
                    let len = heap.stack.len();
                    heap.stack.swap(len - 1, len - 2);
                }

                Instruction::RegisterEndpoint { .. } => {
                    // Endpoint registration is handled during startup
                    // Skip this instruction during execution
                }

                _ => {
                    return Err(anyhow!("Unsupported instruction: {:?}", instruction));
                }
            }
            
            ip += 1;
        }

        Ok(heap.stack.last().cloned())
    }

    /// Handle HTTP request by finding matching endpoint and executing it
    pub async fn handle_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Find matching endpoint
        let endpoint_info = self.find_matching_endpoint(&request.method, &request.path)?;
        
        // Create isolated heap for this request
        let mut isolated_heap = IsolatedHeap::new();
        
        // Set up request context variables
        isolated_heap.set_variable("request".to_string(), Value::Object({
            let mut req_obj = HashMap::new();
            req_obj.insert("method".to_string(), Value::String(request.method.clone()));
            req_obj.insert("path".to_string(), Value::String(request.path.clone()));
            if let Some(body) = &request.body {
                req_obj.insert("body".to_string(), Value::String(body.clone()));
            }
            req_obj
        }));

        // Execute endpoint handler
        let result = self.execute(endpoint_info.handler_start, &mut isolated_heap).await?;
        
        // Convert result to HTTP response
        let response_body = match result {
            Some(Value::String(s)) => Some(s),
            Some(Value::Object(obj)) => Some(serde_json::to_string(&obj)?),
            Some(Value::Array(arr)) => Some(serde_json::to_string(&arr)?),
            Some(other) => Some(format!("{:?}", other)),
            None => None,
        };

        Ok(HttpResponse {
            status: 200,
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers
            },
            body: response_body,
        })
    }

    fn find_matching_endpoint(&self, method: &str, path: &str) -> Result<&EndpointInfo> {
        for endpoint in self.endpoint_handlers.values() {
            if endpoint.method == method && self.path_matches(&endpoint.path, path) {
                return Ok(endpoint);
            }
        }
        Err(anyhow!("No matching endpoint found for {} {}", method, path))
    }

    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        // Simple exact match for now
        // In a full implementation, this would support path parameters
        pattern == path
    }

    async fn simulate_http_call(&self, method: &str, url: &str, _body: Option<&str>) -> Result<String> {
        // Simulate HTTP call - in a real implementation this would use reqwest or similar
        Ok(format!("{{\"simulated_response\": \"{}:{}\"}}", method, url))
    }

    fn add_values(&self, a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (Value::String(x), Value::String(y)) => Ok(Value::String(format!("{}{}", x, y))),
            _ => Err(anyhow!("Cannot add values of different types")),
        }
    }

    fn subtract_values(&self, a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
            _ => Err(anyhow!("Cannot subtract non-numeric values")),
        }
    }

    fn multiply_values(&self, a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),
            _ => Err(anyhow!("Cannot multiply non-numeric values")),
        }
    }

    fn divide_values(&self, a: &Value, b: &Value) -> Result<Value> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => {
                if *y == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(Value::Number(x / y))
                }
            }
            _ => Err(anyhow!("Cannot divide non-numeric values")),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Promise(_) => true, // Promises are always truthy
        }
    }
}

/// HTTP server for running loaf programs
pub struct LoafServer {
    vm: VirtualMachine,
    port: u16,
}

impl LoafServer {
    pub fn new(program: BytecodeProgram, port: u16) -> Self {
        Self {
            vm: VirtualMachine::new(program),
            port,
        }
    }

    pub async fn start(&self) -> Result<()> {
        println!("Starting loaf HTTP server on port {}", self.port);
        
        // In a real implementation, this would start an actual HTTP server
        // For now, we'll just simulate it
        println!("Server started successfully!");
        println!("Registered endpoints:");
        for (id, endpoint) in &self.vm.endpoint_handlers {
            println!("  {} {} -> handler {}", endpoint.method, endpoint.path, id);
        }
        
        Ok(())
    }

    pub async fn handle_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.vm.handle_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolated_heap() {
        let mut heap = IsolatedHeap::new();
        
        // Test stack operations
        heap.push(Value::Number(42.0));
        assert_eq!(heap.stack.len(), 1);
        
        let value = heap.pop().unwrap();
        assert!(matches!(value, Value::Number(42.0)));
        assert_eq!(heap.stack.len(), 0);
        
        // Test variable operations
        heap.set_variable("test".to_string(), Value::String("hello".to_string()));
        let var_value = heap.get_variable("test").unwrap();
        assert!(matches!(var_value, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_promise_operations() {
        let mut heap = IsolatedHeap::new();
        
        let promise_id = "test_promise".to_string();
        heap.create_promise(promise_id.clone()).unwrap();
        
        assert!(!heap.is_promise_resolved(&promise_id));
        
        heap.resolve_promise(&promise_id, Value::Number(123.0)).unwrap();
        assert!(heap.is_promise_resolved(&promise_id));
        
        let value = heap.get_promise_value(&promise_id).unwrap();
        assert!(matches!(value, Value::Number(123.0)));
    }

    #[tokio::test]
    async fn test_http_request_handling() {
        // Create a simple bytecode program with an endpoint
        let program = BytecodeProgram {
            instructions: vec![
                Instruction::LoadConstant(Value::String("Hello World".to_string())),
                Instruction::Return,
            ],
            constants: vec![],
            endpoints: {
                let mut endpoints = HashMap::new();
                endpoints.insert("test_handler".to_string(), EndpointInfo {
                    method: "GET".to_string(),
                    path: "/test".to_string(),
                    handler_start: 0,
                    handler_end: 2,
                });
                endpoints
            },
            entry_point: 0,
        };

        let vm = VirtualMachine::new(program);
        
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
        };

        let response = vm.handle_request(request).await.unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.is_some());
    }
}
