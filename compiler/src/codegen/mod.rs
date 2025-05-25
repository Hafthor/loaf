use crate::analyzer::AnalyzedProgram;
use crate::parser::AstNode;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Bytecode instructions for the loaf runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Value operations
    LoadConstant(Value),
    LoadVariable(String),
    StoreVariable(String),
    
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Object/Array operations
    CreateObject,
    SetProperty(String),
    GetProperty(String),
    CreateArray,
    AppendArray,
    GetIndex(usize),
    
    // Promise operations
    CreatePromise(String), // promise_id
    ResolvePromise(String),
    AwaitPromise(String),
    
    // HTTP operations
    RegisterEndpoint {
        method: String,
        path: String,
        handler_id: String,
    },
    HttpCall {
        method: String,
        url: String,
        body: Option<String>,
    },
    
    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    Return,
    
    // Stack operations
    Duplicate,
    Pop,
    Swap,
}

/// Runtime values that can be stored in bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Promise(String), // promise_id
}

/// Compiled bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeProgram {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub endpoints: HashMap<String, EndpointInfo>,
    pub entry_point: usize,
}

/// Information about HTTP endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub method: String,
    pub path: String,
    pub handler_start: usize,
    pub handler_end: usize,
}

/// Code generator for converting analyzed AST to bytecode
pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    endpoints: HashMap<String, EndpointInfo>,
    label_counter: usize,
    variable_locations: HashMap<String, usize>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            endpoints: HashMap::new(),
            label_counter: 0,
            variable_locations: HashMap::new(),
        }
    }

    /// Generate bytecode from analyzed program
    pub fn generate(&mut self, program: &AnalyzedProgram) -> Result<BytecodeProgram> {
        let entry_point = self.instructions.len();

        // Process assignments in dependency order
        for symbol_name in &program.resolution_order {
            if let Some(symbol) = program.symbol_table.symbols().get(symbol_name) {
                if let Some(ast_node) = &symbol.ast_node {
                    let assignment = AstNode::Assignment {
                        name: symbol.name.clone(),
                        value: Box::new(ast_node.clone()),
                        line: symbol.definition_line,
                    };
                    self.generate_assignment(&assignment, program.symbol_table.symbols())?;
                }
            }
        }

        // Generate endpoint registrations  
        for endpoint in &program.endpoints {
            let handler_id = Uuid::new_v4().to_string();
            let handler_start = self.instructions.len();
            
            // Generate endpoint handler code
            self.generate_expression(&endpoint.handler, program.symbol_table.symbols())?;
            self.emit(Instruction::Return);
            let handler_end = self.instructions.len();

            // Register the endpoint
            self.emit(Instruction::RegisterEndpoint {
                method: format!("{:?}", endpoint.method),
                path: endpoint.path.clone(),
                handler_id: handler_id.clone(),
            });

            self.endpoints.insert(handler_id, EndpointInfo {
                method: format!("{:?}", endpoint.method),
                path: endpoint.path.clone(),
                handler_start,
                handler_end,
            });
        }

        Ok(BytecodeProgram {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
            endpoints: self.endpoints.clone(),
            entry_point,
        })
    }

    fn generate_assignment(&mut self, assignment: &AstNode, symbols: &HashMap<String, crate::analyzer::Symbol>) -> Result<()> {
        if let AstNode::Assignment { name, value, .. } = assignment {
            // Generate code for the value expression
            self.generate_expression(value, symbols)?;
            
            // Store the result in the variable
            self.emit(Instruction::StoreVariable(name.clone()));
        }
        Ok(())
    }

    fn generate_expression(&mut self, expr: &AstNode, symbols: &HashMap<String, crate::analyzer::Symbol>) -> Result<()> {
        match expr {
            AstNode::String(value) => {
                // Check for special annotations
                if value.starts_with("@promise:") {
                    let promise_id = value.strip_prefix("@promise:").unwrap_or(value);
                    self.emit(Instruction::CreatePromise(promise_id.to_string()));
                } else if value.starts_with("@endpoint:") {
                    // Parse endpoint annotation: @endpoint:METHOD:PATH
                    let parts: Vec<&str> = value.strip_prefix("@endpoint:").unwrap_or("").split(':').collect();
                    if parts.len() >= 2 {
                        let method = parts[0].to_string();
                        let path = parts[1].to_string();
                        let endpoint_key = format!("{}:{}", method, path);
                        
                        self.emit(Instruction::RegisterEndpoint {
                            method: method.clone(),
                            path: path.clone(),
                            handler_id: "default".to_string(),
                        });
                        
                        // Add to endpoints map
                        self.endpoints.insert(endpoint_key, crate::codegen::EndpointInfo {
                            method,
                            path,
                            handler_start: self.instructions.len(),
                            handler_end: self.instructions.len(),
                        });
                    } else {
                        self.emit(Instruction::LoadConstant(Value::String(value.clone())));
                    }
                } else if value.starts_with("@http:") {
                    // Parse HTTP call annotation: @http:METHOD:URL
                    let parts: Vec<&str> = value.strip_prefix("@http:").unwrap_or("").split(':').collect();
                    if parts.len() >= 2 {
                        let method = parts[0].to_string();
                        let url = parts[1..].join(":");  // Rejoin in case URL contains colons
                        self.emit(Instruction::HttpCall {
                            method,
                            url,
                            body: None,
                        });
                    } else {
                        self.emit(Instruction::LoadConstant(Value::String(value.clone())));
                    }
                } else {
                    self.emit(Instruction::LoadConstant(Value::String(value.clone())));
                }
            }

            AstNode::Number(value) => {
                self.emit(Instruction::LoadConstant(Value::Number(*value)));
            }

            AstNode::Boolean(value) => {
                self.emit(Instruction::LoadConstant(Value::Boolean(*value)));
            }

            AstNode::Null => {
                self.emit(Instruction::LoadConstant(Value::Null));
            }

            AstNode::Identifier(name) => {
                // Check if this variable is a promise
                if let Some(symbol) = symbols.get(name) {
                    match &symbol.symbol_type {
                        crate::analyzer::Type::Promise(_) => {
                            self.emit(Instruction::AwaitPromise(name.clone()));
                        }
                        _ => {
                            self.emit(Instruction::LoadVariable(name.clone()));
                        }
                    }
                } else {
                    self.emit(Instruction::LoadVariable(name.clone()));
                }
            }

            AstNode::Binary { left, operator, right, .. } => {
                self.generate_expression(left, symbols)?;
                self.generate_expression(right, symbols)?;
                
                match operator {
                    crate::parser::BinaryOp::Add => self.emit(Instruction::Add),
                    crate::parser::BinaryOp::Subtract => self.emit(Instruction::Subtract),
                    crate::parser::BinaryOp::Multiply => self.emit(Instruction::Multiply),
                    crate::parser::BinaryOp::Divide => self.emit(Instruction::Divide),
                    _ => return Err(anyhow!("Unsupported binary operator: {:?}", operator)),
                }
            }

            AstNode::Object { fields, .. } => {
                self.emit(Instruction::CreateObject);
                for (key, value) in fields {
                    self.generate_expression(value, symbols)?;
                    self.emit(Instruction::SetProperty(key.clone()));
                }
            }

            AstNode::Array { elements, .. } => {
                self.emit(Instruction::CreateArray);
                for element in elements {
                    self.generate_expression(element, symbols)?;
                    self.emit(Instruction::AppendArray);
                }
            }

            AstNode::Promise { .. } => {
                let promise_id = Uuid::new_v4().to_string();
                self.emit(Instruction::CreatePromise(promise_id.clone()));
                
                // The promise will be resolved when the target is computed
                // This is handled by the promise resolution system
            }

            AstNode::HttpCall { method, url, body, .. } => {
                let body_str = if let Some(_body_node) = body {
                    // For now, we'll serialize the body as JSON
                    // In a full implementation, this would be more sophisticated
                    Some("{}".to_string())
                } else {
                    None
                };

                // For now, convert URL to string - in full implementation would evaluate it
                let url_str = format!("{:?}", url); // Simple conversion for demo

                self.emit(Instruction::HttpCall {
                    method: format!("{:?}", method),
                    url: url_str,
                    body: body_str,
                });
            }

            AstNode::MemberAccess { object, property, .. } => {
                // Generate code for the object
                self.generate_expression(object, symbols)?;
                
                // Emit instruction to get the property
                self.emit(Instruction::GetProperty(property.clone()));
            }

            AstNode::FunctionCall { name, arguments, .. } => {
                // Generate code for arguments
                for arg in arguments {
                    self.generate_expression(arg, symbols)?;
                }
                
                // Generate a promise for the function call
                let promise_id = format!("{}:{}", name, Uuid::new_v4());
                self.emit(Instruction::CreatePromise(promise_id));
            }

            _ => {
                return Err(anyhow!("Unsupported expression type for code generation"));
            }
        }
        Ok(())
    }

    fn convert_literal_to_value(&self, literal: &serde_json::Value) -> Result<Value> {
        match literal {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Ok(Value::Number(f))
                } else {
                    Err(anyhow!("Invalid number format"))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            serde_json::Value::Array(arr) => {
                let mut values = Vec::new();
                for item in arr {
                    values.push(self.convert_literal_to_value(item)?);
                }
                Ok(Value::Array(values))
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (key, value) in obj {
                    map.insert(key.clone(), self.convert_literal_to_value(value)?);
                }
                Ok(Value::Object(map))
            }
        }
    }

    fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn next_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::SemanticAnalyzer;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_simple_assignment_codegen() {
        let source = r#"{ "x": 42 }"#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let analyzed = analyzer.analyze(&ast).unwrap();
        
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.generate(&analyzed).unwrap();
        
        assert!(!bytecode.instructions.is_empty());
        assert_eq!(bytecode.entry_point, 0);
    }

    #[test]
    fn test_promise_codegen() {
        let source = r#"{
            "a": "@promise:b",
            "b": 42
        }"#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let analyzed = analyzer.analyze(&ast).unwrap();
        
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.generate(&analyzed).unwrap();
        
        // Should contain promise-related instructions
        let has_promise_instruction = bytecode.instructions.iter().any(|inst| {
            matches!(inst, Instruction::CreatePromise(_) | Instruction::AwaitPromise(_))
        });
        assert!(has_promise_instruction);
    }

    #[test]
    fn test_endpoint_codegen() {
        let source = r#"{
            "handler": "@endpoint:GET:/api/test",
            "response": { "message": "Hello World" }
        }"#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let analyzed = analyzer.analyze(&ast).unwrap();
        
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.generate(&analyzed).unwrap();
        
        // Should contain endpoint registration
        let has_endpoint_instruction = bytecode.instructions.iter().any(|inst| {
            matches!(inst, Instruction::RegisterEndpoint { .. })
        });
        assert!(has_endpoint_instruction);
        assert!(!bytecode.endpoints.is_empty());
    }

    #[test]
    fn test_arithmetic_codegen() {
        let source = r#"{ "result": 10 + 5 * 2 }"#;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        let analyzed = analyzer.analyze(&ast).unwrap();
        
        let mut codegen = CodeGenerator::new();
        let bytecode = codegen.generate(&analyzed).unwrap();
        
        // Should contain arithmetic instructions
        let has_arithmetic = bytecode.instructions.iter().any(|inst| {
            matches!(inst, Instruction::Add | Instruction::Multiply)
        });
        assert!(has_arithmetic);
    }
}
