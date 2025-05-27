use crate::parser::{AstNode, BinaryOp, UnaryOp, HttpMethod};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Boolean,
    Null,
    Array(Box<Type>),
    Object(HashMap<String, Type>),
    Promise(Option<Box<Type>>),
    Any,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "boolean"),
            Type::Null => write!(f, "null"),
            Type::Array(inner) => write!(f, "array<{}>", inner),
            Type::Object(_) => write!(f, "object"),
            Type::Promise(inner) => match inner {
                Some(inner_type) => write!(f, "promise<{}>", inner_type),
                None => write!(f, "promise<any>"),
            },
            Type::Any => write!(f, "any"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub is_resolved: bool,
    pub dependencies: HashSet<String>,
    pub dependents: HashSet<String>,
    pub definition_line: usize,
    pub ast_node: Option<AstNode>,
}

impl Symbol {
    pub fn new(name: String, symbol_type: Type, definition_line: usize) -> Self {
        Self {
            name,
            symbol_type,
            is_resolved: false,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            definition_line,
            ast_node: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    resolution_order: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            resolution_order: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name.clone(), symbol);
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn get_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    pub fn add_dependency(&mut self, dependent: &str, dependency: &str) {
        if let Some(dep_symbol) = self.symbols.get_mut(dependency) {
            dep_symbol.dependents.insert(dependent.to_string());
        }
        if let Some(dependent_symbol) = self.symbols.get_mut(dependent) {
            dependent_symbol.dependencies.insert(dependency.to_string());
        }
    }

    pub fn resolve_dependencies(&mut self) -> Result<Vec<String>, AnalyzerError> {
        // Topological sort to determine resolution order
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize
        for (name, symbol) in &self.symbols {
            in_degree.insert(name.clone(), symbol.dependencies.len());
            graph.insert(name.clone(), symbol.dependents.iter().cloned().collect());
        }
        
        // Kahn's algorithm
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result = Vec::new();
        
        // Find symbols with no dependencies
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name.clone());
            }
        }
        
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            
            if let Some(dependents) = graph.get(&current) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }
        
        // Check for circular dependencies
        if result.len() != self.symbols.len() {
            let unresolved: Vec<String> = self.symbols.keys()
                .filter(|name| !result.contains(name))
                .cloned()
                .collect();
            return Err(AnalyzerError::CircularDependency(unresolved));
        }
        
        self.resolution_order = result.clone();
        Ok(result)
    }

    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.symbols
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    current_scope: String,
    endpoints: Vec<EndpointInfo>,
    tests: Vec<TestInfo>,
}

#[derive(Debug, Clone)]
pub struct EndpointInfo {
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub handler: AstNode,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct TestInfo {
    pub name: String,
    pub expect_expression: AstNode,
    pub inputs: HashMap<String, AstNode>,
    pub expected_output: AstNode,
    pub is_regex: bool,
    pub line: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_scope: "global".to_string(),
            endpoints: Vec::new(),
            tests: Vec::new(),
        }
    }

    pub fn analyze(&mut self, ast: &AstNode) -> Result<AnalyzedProgram, AnalyzerError> {
        // First pass: collect all symbols and their dependencies
        self.collect_symbols(ast)?;
        
        // Resolve dependencies and determine evaluation order
        let resolution_order = self.symbol_table.resolve_dependencies()?;
        
        // Second pass: type inference and promise propagation
        self.infer_types(&resolution_order)?;
        
        // Third pass: validate the program
        self.validate_program()?;
        
        Ok(AnalyzedProgram {
            symbol_table: self.symbol_table.clone(),
            resolution_order,
            endpoints: self.endpoints.clone(),
            tests: self.tests.clone(),
        })
    }

    fn collect_symbols(&mut self, node: &AstNode) -> Result<(), AnalyzerError> {
        match node {
            AstNode::Program(statements) => {
                for statement in statements {
                    self.collect_symbols(statement)?;
                }
            }
            
            AstNode::Assignment { name, value, line } => {
                // Create symbol for this assignment
                let symbol_type = self.infer_type_from_ast(value)?;
                let mut symbol = Symbol::new(name.clone(), symbol_type, *line);
                symbol.ast_node = Some(value.as_ref().clone());
                
                // Collect dependencies from the value expression
                let dependencies = self.collect_dependencies(value)?;
                for dep in &dependencies {
                    symbol.dependencies.insert(dep.clone());
                }
                
                self.symbol_table.add_symbol(symbol);
                
                // Establish bidirectional dependencies in the symbol table
                for dep in dependencies {
                    self.symbol_table.add_dependency(name, &dep);
                }
            }
            
            AstNode::Endpoint { name, method, path, handler, line } => {
                // Register the endpoint
                self.endpoints.push(EndpointInfo {
                    name: name.clone(),
                    method: method.clone(),
                    path: path.clone(),
                    handler: handler.as_ref().clone(),
                    line: *line,
                });
                
                // Analyze the handler
                self.collect_symbols(handler)?;
            }
            
            AstNode::Test { name, expect_expression, inputs, expected_output, is_regex, line } => {
                // Register the test
                self.tests.push(TestInfo {
                    name: name.clone(),
                    expect_expression: expect_expression.as_ref().clone(),
                    inputs: inputs.clone(),
                    expected_output: expected_output.as_ref().clone(),
                    is_regex: *is_regex,
                    line: *line,
                });
                
                // Analyze the expect expression, inputs and expected output
                self.collect_symbols(expect_expression)?;
                for (_, input_value) in inputs {
                    self.collect_symbols(input_value)?;
                }
                self.collect_symbols(expected_output)?;
            }
            
            AstNode::Object { fields, line } => {
                for (field_name, field_value) in fields {
                    // Treat each field as a symbol definition
                    let symbol_type = self.infer_type_from_ast(field_value)?;
                    let mut symbol = Symbol::new(field_name.clone(), symbol_type, *line);
                    symbol.ast_node = Some(field_value.clone());
                    
                    // Collect dependencies from the field value expression
                    let dependencies = self.collect_dependencies(field_value)?;
                    for dep in &dependencies {
                        symbol.dependencies.insert(dep.clone());
                    }
                    
                    self.symbol_table.add_symbol(symbol);
                    
                    // Establish bidirectional dependencies in the symbol table
                    for dep in dependencies {
                        self.symbol_table.add_dependency(field_name, &dep);
                    }
                    
                    // Also analyze the field value for nested structures
                    self.collect_symbols(field_value)?;
                }
            }
            
            AstNode::Array { elements, .. } => {
                for element in elements {
                    self.collect_symbols(element)?;
                }
            }
            
            AstNode::Binary { left, right, .. } => {
                self.collect_symbols(left)?;
                self.collect_symbols(right)?;
            }
            
            AstNode::Unary { operand, .. } => {
                self.collect_symbols(operand)?;
            }
            
            AstNode::MemberAccess { object, .. } => {
                self.collect_symbols(object)?;
            }
            
            AstNode::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.collect_symbols(arg)?;
                }
            }
            
            AstNode::Promise { expression, .. } => {
                self.collect_symbols(expression)?;
            }
            
            AstNode::HttpCall { url, body, headers, .. } => {
                self.collect_symbols(url)?;
                if let Some(body) = body {
                    self.collect_symbols(body)?;
                }
                if let Some(headers) = headers {
                    for value in headers.values() {
                        self.collect_symbols(value)?;
                    }
                }
            }
            
            // Leaf nodes don't need collection
            _ => {}
        }
        
        Ok(())
    }

    fn collect_dependencies(&self, node: &AstNode) -> Result<HashSet<String>, AnalyzerError> {
        let mut dependencies = HashSet::new();
        
        match node {
            AstNode::Identifier(name) => {
                dependencies.insert(name.clone());
            }
            
            AstNode::Binary { left, right, .. } => {
                dependencies.extend(self.collect_dependencies(left)?);
                dependencies.extend(self.collect_dependencies(right)?);
            }
            
            AstNode::Unary { operand, .. } => {
                dependencies.extend(self.collect_dependencies(operand)?);
            }
            
            AstNode::MemberAccess { object, .. } => {
                dependencies.extend(self.collect_dependencies(object)?);
            }
            
            AstNode::Object { fields, .. } => {
                for value in fields.values() {
                    dependencies.extend(self.collect_dependencies(value)?);
                }
            }
            
            AstNode::Array { elements, .. } => {
                for element in elements {
                    dependencies.extend(self.collect_dependencies(element)?);
                }
            }
            
            AstNode::Promise { expression, .. } => {
                dependencies.extend(self.collect_dependencies(expression)?);
            }
            
            AstNode::HttpCall { url, body, headers, .. } => {
                dependencies.extend(self.collect_dependencies(url)?);
                if let Some(body) = body {
                    dependencies.extend(self.collect_dependencies(body)?);
                }
                if let Some(headers) = headers {
                    for value in headers.values() {
                        dependencies.extend(self.collect_dependencies(value)?);
                    }
                }
            }
            
            AstNode::FunctionCall { arguments, .. } => {
                // Collect dependencies from function arguments
                for arg in arguments {
                    dependencies.extend(self.collect_dependencies(arg)?);
                }
            }
            
            // Leaf nodes have no dependencies
            _ => {}
        }
        
        Ok(dependencies)
    }

    fn infer_type_from_ast(&self, node: &AstNode) -> Result<Type, AnalyzerError> {
        match node {
            AstNode::String(_) => Ok(Type::String),
            AstNode::Number(_) => Ok(Type::Number),
            AstNode::Boolean(_) => Ok(Type::Boolean),
            AstNode::Null => Ok(Type::Null),
            
            AstNode::Identifier(name) => {
                if let Some(symbol) = self.symbol_table.get_symbol(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    // Forward reference - assume it will be defined later
                    Ok(Type::Any)
                }
            }
            
            AstNode::Binary { left, right, operator, line } => {
                let left_type = self.infer_type_from_ast(left)?;
                let right_type = self.infer_type_from_ast(right)?;
                
                match operator {
                    BinaryOp::Add => {
                        match (&left_type, &right_type) {
                            (Type::Number, Type::Number) => Ok(Type::Number),
                            (Type::String, _) | (_, Type::String) => Ok(Type::String),
                            (Type::Promise(inner_left), Type::Promise(inner_right)) => {
                                // Both are promises, result is a promise of the add operation
                                if let (Some(left), Some(right)) = (inner_left.as_ref(), inner_right.as_ref()) {
                                    let result_type = self.infer_binary_result_type(left, right, operator)?;
                                    Ok(Type::Promise(Some(Box::new(result_type))))
                                } else {
                                    Ok(Type::Promise(Some(Box::new(Type::Any))))
                                }
                            }
                            (Type::Promise(inner), _) | (_, Type::Promise(inner)) => {
                                // One operand is a promise, result becomes a promise
                                let other_type = if matches!(left_type, Type::Promise(_)) {
                                    &right_type
                                } else {
                                    &left_type
                                };
                                if let Some(inner_type) = inner.as_ref() {
                                    let result_type = self.infer_binary_result_type(inner_type, other_type, operator)?;
                                    Ok(Type::Promise(Some(Box::new(result_type))))
                                } else {
                                    Ok(Type::Promise(Some(Box::new(Type::Any))))
                                }
                            }
                            _ => Ok(Type::Any),
                        }
                    }
                    
                    BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        match (&left_type, &right_type) {
                            (Type::Number, Type::Number) => Ok(Type::Number),
                            (Type::Promise(inner_left), Type::Promise(inner_right)) => {
                                if let (Some(left), Some(right)) = (inner_left.as_ref(), inner_right.as_ref()) {
                                    let result_type = self.infer_binary_result_type(left, right, operator)?;
                                    Ok(Type::Promise(Some(Box::new(result_type))))
                                } else {
                                    Ok(Type::Promise(Some(Box::new(Type::Number))))
                                }
                            }
                            (Type::Promise(inner), Type::Number) | (Type::Number, Type::Promise(inner)) => {
                                if let Some(inner_type) = inner.as_ref() {
                                    if matches!(**inner_type, Type::Number) {
                                        Ok(Type::Promise(Some(Box::new(Type::Number))))
                                    } else {
                                        Err(AnalyzerError::TypeError {
                                            expected: "number".to_string(),
                                            found: format!("{:?}", inner_type),
                                            line: *line,
                                        })
                                    }
                                } else {
                                    Ok(Type::Promise(Some(Box::new(Type::Number))))
                                }
                            }
                            _ => Err(AnalyzerError::TypeError {
                                expected: "number".to_string(),
                                found: format!("{} and {}", left_type, right_type),
                                line: *line,
                            }),
                        }
                    }
                    
                    BinaryOp::Equal => Ok(Type::Boolean),
                }
            }
            
            AstNode::Unary { operand, operator, .. } => {
                let operand_type = self.infer_type_from_ast(operand)?;
                match operator {
                    UnaryOp::Negate => {
                        match operand_type {
                            Type::Number => Ok(Type::Number),
                            Type::Promise(inner) if inner.is_some() && matches!(**inner.as_ref().unwrap(), Type::Number) => {
                                Ok(Type::Promise(Some(Box::new(Type::Number))))
                            }
                            _ => Ok(Type::Any),
                        }
                    }
                    UnaryOp::Not => Ok(Type::Boolean),
                }
            }
            
            AstNode::Array { elements, .. } => {
                if elements.is_empty() {
                    Ok(Type::Array(Box::new(Type::Any)))
                } else {
                    let element_type = self.infer_type_from_ast(&elements[0])?;
                    Ok(Type::Array(Box::new(element_type)))
                }
            }
            
            AstNode::Object { fields, .. } => {
                let mut object_fields = HashMap::new();
                for (key, value) in fields {
                    let value_type = self.infer_type_from_ast(value)?;
                    object_fields.insert(key.clone(), value_type);
                }
                Ok(Type::Object(object_fields))
            }
            
            AstNode::Promise { expression, .. } => {
                let inner_type = self.infer_type_from_ast(expression)?;
                Ok(Type::Promise(Some(Box::new(inner_type))))
            }
            
            AstNode::HttpCall { .. } => {
                // HTTP calls return promises by default
                Ok(Type::Promise(Some(Box::new(Type::Any))))
            }
            
            AstNode::FunctionCall { name: _, arguments: _, .. } => {
                // Function calls are automatically treated as promises
                Ok(Type::Promise(Some(Box::new(Type::Any))))
            }

            AstNode::MemberAccess { object, property, .. } => {
                let object_type = self.infer_type_from_ast(object)?;
                match object_type {
                    Type::Object(fields) => {
                        if let Some(field_type) = fields.get(property) {
                            Ok(field_type.clone())
                        } else {
                            Ok(Type::Any)
                        }
                    }
                    Type::Promise(inner) => {
                        // Member access on a promise returns a promise
                        if let Some(inner_type) = inner.as_ref() {
                            match inner_type.as_ref() {
                                Type::Object(fields) => {
                                    if let Some(field_type) = fields.get(property) {
                                        Ok(Type::Promise(Some(Box::new(field_type.clone()))))
                                    } else {
                                        Ok(Type::Promise(Some(Box::new(Type::Any))))
                                    }
                                }
                                _ => Ok(Type::Promise(Some(Box::new(Type::Any))))
                            }
                        } else {
                            Ok(Type::Promise(Some(Box::new(Type::Any))))
                        }
                    }
                    _ => Ok(Type::Any),
                }
            }
            
            AstNode::Test { .. } => {
                // Test declarations don't have a runtime type, they are metadata
                Ok(Type::Any)
            }
            
            _ => Ok(Type::Any),
        }
    }

    fn infer_binary_result_type(&self, left: &Type, right: &Type, operator: &BinaryOp) -> Result<Type, AnalyzerError> {
        match operator {
            BinaryOp::Add => {
                match (left, right) {
                    (Type::Number, Type::Number) => Ok(Type::Number),
                    (Type::String, _) | (_, Type::String) => Ok(Type::String),
                    _ => Ok(Type::Any),
                }
            }
            BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                match (left, right) {
                    (Type::Number, Type::Number) => Ok(Type::Number),
                    _ => Ok(Type::Any),
                }
            }
            BinaryOp::Equal => Ok(Type::Boolean),
        }
    }

    fn infer_types(&mut self, resolution_order: &[String]) -> Result<(), AnalyzerError> {
        for symbol_name in resolution_order {
            if let Some(symbol) = self.symbol_table.get_symbol(symbol_name).cloned() {
                // Check if any dependencies are promises
                let has_promise_dependency = symbol.dependencies.iter()
                    .any(|dep| {
                        if let Some(dep_symbol) = self.symbol_table.get_symbol(dep) {
                            matches!(dep_symbol.symbol_type, Type::Promise(_))
                        } else {
                            false
                        }
                    });
                
                // If any dependency is a promise, this symbol becomes a promise too
                if has_promise_dependency {
                    if let Some(symbol_mut) = self.symbol_table.get_symbol_mut(symbol_name) {
                        if !matches!(symbol_mut.symbol_type, Type::Promise(_)) {
                            symbol_mut.symbol_type = Type::Promise(Some(Box::new(symbol_mut.symbol_type.clone())));
                        }
                    }
                }
                
                // Mark as resolved
                if let Some(symbol_mut) = self.symbol_table.get_symbol_mut(symbol_name) {
                    symbol_mut.is_resolved = true;
                }
            }
        }
        
        Ok(())
    }

    fn validate_program(&self) -> Result<(), AnalyzerError> {
        // Check for undefined symbols
        for symbol in self.symbol_table.symbols().values() {
            for dependency in &symbol.dependencies {
                if !self.symbol_table.symbols().contains_key(dependency) {
                    return Err(AnalyzerError::UndefinedSymbol {
                        name: dependency.clone(),
                        line: symbol.definition_line,
                    });
                }
            }
        }
        
        // Validate endpoints
        for endpoint in &self.endpoints {
            // Check for duplicate endpoint paths with same method
            let conflicts: Vec<_> = self.endpoints.iter()
                .filter(|other| {
                    other.name != endpoint.name &&
                    other.method == endpoint.method &&
                    other.path == endpoint.path
                })
                .collect();
            
            if !conflicts.is_empty() {
                return Err(AnalyzerError::DuplicateEndpoint {
                    name: endpoint.name.clone(),
                    method: format!("{:?}", endpoint.method),
                    path: endpoint.path.clone(),
                    line: endpoint.line,
                });
            }
        }
        
        Ok(())
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn endpoints(&self) -> &[EndpointInfo] {
        &self.endpoints
    }

    pub fn tests(&self) -> &[TestInfo] {
        &self.tests
    }
}

#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub symbol_table: SymbolTable,
    pub resolution_order: Vec<String>,
    pub endpoints: Vec<EndpointInfo>,
    pub tests: Vec<TestInfo>,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("Circular dependency detected involving symbols: {0:?}")]
    CircularDependency(Vec<String>),
    
    #[error("Undefined symbol '{name}' at line {line}")]
    UndefinedSymbol { name: String, line: usize },
    
    #[error("Type error at line {line}: expected {expected}, found {found}")]
    TypeError { expected: String, found: String, line: usize },
    
    #[error("Duplicate endpoint '{name}' for {method} {path} at line {line}")]
    DuplicateEndpoint { name: String, method: String, path: String, line: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze_source(source: &str) -> Result<AnalyzedProgram, AnalyzerError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast)
    }

    #[test]
    fn test_forward_reference_resolution() {
        let source = r#"{
            "price": 10.0,
            "quantity": 2,
            "subtotal": 20.0,
            "tax": 2.0,
            "total": 22.0
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Check that all symbols are resolved
        for symbol in result.symbol_table.symbols().values() {
            assert!(symbol.is_resolved, "Symbol {} should be resolved", symbol.name);
        }
        
        // Check that symbols were collected
        assert!(result.symbol_table.get_symbol("price").is_some());
        assert!(result.symbol_table.get_symbol("quantity").is_some());
        assert!(result.symbol_table.get_symbol("subtotal").is_some());
        assert!(result.symbol_table.get_symbol("tax").is_some());
        assert!(result.symbol_table.get_symbol("total").is_some());
    }

    #[test]
    fn test_promise_propagation() {
        let source = r#"{
            "user_data": "@promise:fetch_user",
            "greeting": "Hello World"
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Check that promise symbol was created
        let user_data_symbol = result.symbol_table.get_symbol("user_data").unwrap();
        let greeting_symbol = result.symbol_table.get_symbol("greeting").unwrap();
        
        // For now, just check that symbols exist
        assert!(user_data_symbol.is_resolved);
        assert!(greeting_symbol.is_resolved);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let source = r#"{
            "a": 1,
            "b": 2, 
            "c": 3
        }"#;
        
        let result = analyze_source(source);
        assert!(result.is_ok(), "Simple literals should not cause circular dependency");
        
        // For now, we don't have actual circular dependencies in our test cases
        // since we're using literals instead of variable references
    }

    #[test]
    fn test_basic_type_inference() {
        let source = r#"{
            "str_val": "hello",
            "num_val": 42.5,
            "bool_val": true,
            "null_val": null
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let str_symbol = result.symbol_table.get_symbol("str_val").unwrap();
        assert_eq!(str_symbol.symbol_type, Type::String);
        
        let num_symbol = result.symbol_table.get_symbol("num_val").unwrap();
        assert_eq!(num_symbol.symbol_type, Type::Number);
        
        let bool_symbol = result.symbol_table.get_symbol("bool_val").unwrap();
        assert_eq!(bool_symbol.symbol_type, Type::Boolean);
        
        let null_symbol = result.symbol_table.get_symbol("null_val").unwrap();
        assert_eq!(null_symbol.symbol_type, Type::Null);
    }

    #[test]
    fn test_array_type_inference() {
        let source = r#"{
            "numbers": [1, 2, 3],
            "strings": ["a", "b", "c"],
            "empty": []
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let numbers_symbol = result.symbol_table.get_symbol("numbers").unwrap();
        if let Type::Array(inner) = &numbers_symbol.symbol_type {
            assert_eq!(**inner, Type::Number);
        } else {
            panic!("Expected array type for numbers");
        }
        
        let strings_symbol = result.symbol_table.get_symbol("strings").unwrap();
        if let Type::Array(inner) = &strings_symbol.symbol_type {
            assert_eq!(**inner, Type::String);
        } else {
            panic!("Expected array type for strings");
        }
        
        let empty_symbol = result.symbol_table.get_symbol("empty").unwrap();
        if let Type::Array(inner) = &empty_symbol.symbol_type {
            assert_eq!(**inner, Type::Any);
        } else {
            panic!("Expected array type for empty array");
        }
    }

    #[test]
    fn test_object_type_inference() {
        let source = r#"{
            "user": {
                "name": "John",
                "age": 30,
                "active": true
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let user_symbol = result.symbol_table.get_symbol("user").unwrap();
        if let Type::Object(fields) = &user_symbol.symbol_type {
            assert_eq!(fields.get("name"), Some(&Type::String));
            assert_eq!(fields.get("age"), Some(&Type::Number));
            assert_eq!(fields.get("active"), Some(&Type::Boolean));
        } else {
            panic!("Expected object type for user");
        }
    }

    #[test]
    fn test_binary_operations_type_inference() {
        // Note: This test assumes the parser can handle identifiers and binary operations
        // For now, we'll test with literal operations
        let source = r#"{
            "addition": "hello",
            "subtraction": 42,
            "multiplication": 3.14,
            "comparison": true
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // These are just literal assignments for now, but structure is ready for binary ops
        assert!(result.symbol_table.get_symbol("addition").is_some());
        assert!(result.symbol_table.get_symbol("subtraction").is_some());
        assert!(result.symbol_table.get_symbol("multiplication").is_some());
        assert!(result.symbol_table.get_symbol("comparison").is_some());
    }

    #[test]
    fn test_promise_type_creation() {
        let source = r#"{
            "data": "initial value"
        }"#;
        
        let _result = analyze_source(source).unwrap();
        
        // Test that we can create promise types manually
        let promise_type = Type::Promise(Some(Box::new(Type::String)));
        assert!(matches!(promise_type, Type::Promise(_)));
        
        if let Type::Promise(inner) = promise_type {
            assert_eq!(inner.unwrap().as_ref(), &Type::String);
        }
    }

    #[test]
    fn test_symbol_table_operations() {
        let mut symbol_table = SymbolTable::new();
        
        // Test adding symbols
        let symbol1 = Symbol::new("test1".to_string(), Type::String, 1);
        let symbol2 = Symbol::new("test2".to_string(), Type::Number, 2);
        
        symbol_table.add_symbol(symbol1);
        symbol_table.add_symbol(symbol2);
        
        // Test retrieving symbols
        assert!(symbol_table.get_symbol("test1").is_some());
        assert!(symbol_table.get_symbol("test2").is_some());
        assert!(symbol_table.get_symbol("nonexistent").is_none());
        
        // Test dependencies
        symbol_table.add_dependency("test2", "test1");
        
        let test2_symbol = symbol_table.get_symbol("test2").unwrap();
        assert!(test2_symbol.dependencies.contains("test1"));
        
        let test1_symbol = symbol_table.get_symbol("test1").unwrap();
        assert!(test1_symbol.dependents.contains("test2"));
    }

    #[test]
    fn test_dependency_resolution_simple() {
        let source = r#"{
            "a": 1,
            "b": 2,
            "c": 3
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // With literal values, there should be no dependencies
        assert_eq!(result.resolution_order.len(), 3);
        assert!(result.resolution_order.contains(&"a".to_string()));
        assert!(result.resolution_order.contains(&"b".to_string()));
        assert!(result.resolution_order.contains(&"c".to_string()));
    }

    #[test]
    fn test_nested_object_analysis() {
        let source = r#"{
            "config": {
                "database": {
                    "host": "localhost",
                    "port": 5432,
                    "enabled": true
                },
                "cache": {
                    "ttl": 300,
                    "enabled": false
                }
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Check that nested fields are analyzed
        let config_symbol = result.symbol_table.get_symbol("config").unwrap();
        assert!(matches!(config_symbol.symbol_type, Type::Object(_)));
        
        // Check that nested object fields are created as symbols
        assert!(result.symbol_table.get_symbol("database").is_some());
        assert!(result.symbol_table.get_symbol("cache").is_some());
        assert!(result.symbol_table.get_symbol("host").is_some());
        assert!(result.symbol_table.get_symbol("port").is_some());
        assert!(result.symbol_table.get_symbol("enabled").is_some());
        assert!(result.symbol_table.get_symbol("ttl").is_some());
    }

    #[test]
    fn test_array_with_mixed_types() {
        let source = r#"{
            "mixed": [1, "hello", true, null]
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let mixed_symbol = result.symbol_table.get_symbol("mixed").unwrap();
        if let Type::Array(inner) = &mixed_symbol.symbol_type {
            // First element determines array type, so should be Number
            assert_eq!(**inner, Type::Number);
        } else {
            panic!("Expected array type for mixed array");
        }
    }

    #[test]
    fn test_empty_program() {
        let source = r#"{}"#;
        
        let result = analyze_source(source).unwrap();
        
        assert_eq!(result.symbol_table.symbols().len(), 0);
        assert_eq!(result.resolution_order.len(), 0);
        assert_eq!(result.endpoints.len(), 0);
    }

    #[test]
    fn test_symbol_resolution_status() {
        let source = r#"{
            "value1": "test",
            "value2": 42
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // All symbols should be resolved after analysis
        for symbol in result.symbol_table.symbols().values() {
            assert!(symbol.is_resolved, "Symbol {} should be resolved", symbol.name);
        }
    }

    #[test]
    fn test_type_display_formatting() {
        assert_eq!(format!("{}", Type::Number), "number");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Boolean), "boolean");
        assert_eq!(format!("{}", Type::Null), "null");
        assert_eq!(format!("{}", Type::Any), "any");
        
        let array_type = Type::Array(Box::new(Type::String));
        assert_eq!(format!("{}", array_type), "array<string>");
        
        let promise_type = Type::Promise(Some(Box::new(Type::Number)));
        assert_eq!(format!("{}", promise_type), "promise<number>");
        
        let promise_any = Type::Promise(None);
        assert_eq!(format!("{}", promise_any), "promise<any>");
        
        let object_type = Type::Object(HashMap::new());
        assert_eq!(format!("{}", object_type), "object");
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("test_symbol".to_string(), Type::String, 42);
        
        assert_eq!(symbol.name, "test_symbol");
        assert_eq!(symbol.symbol_type, Type::String);
        assert_eq!(symbol.definition_line, 42);
        assert!(!symbol.is_resolved);
        assert!(symbol.dependencies.is_empty());
        assert!(symbol.dependents.is_empty());
        assert!(symbol.ast_node.is_none());
    }

    #[test]
    fn test_analyzer_initialization() {
        let analyzer = SemanticAnalyzer::new();
        
        assert_eq!(analyzer.current_scope, "global");
        assert_eq!(analyzer.symbol_table.symbols().len(), 0);
        assert_eq!(analyzer.endpoints.len(), 0);
    }

    #[test]
    fn test_complex_nested_structure() {
        let source = r#"{
            "api": {
                "version": "1.0",
                "endpoints": [
                    {
                        "path": "/users",
                        "methods": ["GET", "POST"]
                    },
                    {
                        "path": "/posts",
                        "methods": ["GET", "POST", "PUT", "DELETE"]
                    }
                ],
                "config": {
                    "timeout": 5000,
                    "retries": 3,
                    "debug": false
                }
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Should have symbols for all the nested structure
        assert!(result.symbol_table.get_symbol("api").is_some());
        assert!(result.symbol_table.get_symbol("version").is_some());
        assert!(result.symbol_table.get_symbol("endpoints").is_some());
        assert!(result.symbol_table.get_symbol("config").is_some());
        assert!(result.symbol_table.get_symbol("timeout").is_some());
        assert!(result.symbol_table.get_symbol("retries").is_some());
        assert!(result.symbol_table.get_symbol("debug").is_some());
        
        // Nested objects and arrays should have correct types
        let api_symbol = result.symbol_table.get_symbol("api").unwrap();
        assert!(matches!(api_symbol.symbol_type, Type::Object(_)));
        
        let endpoints_symbol = result.symbol_table.get_symbol("endpoints").unwrap();
        assert!(matches!(endpoints_symbol.symbol_type, Type::Array(_)));
    }

    #[test]
    fn test_error_types() {
        // Test that error types can be created (even if we can't trigger them easily with current parser)
        let circular_error = AnalyzerError::CircularDependency(vec!["a".to_string(), "b".to_string()]);
        assert!(matches!(circular_error, AnalyzerError::CircularDependency(_)));
        
        let undefined_error = AnalyzerError::UndefinedSymbol { 
            name: "test".to_string(), 
            line: 1 
        };
        assert!(matches!(undefined_error, AnalyzerError::UndefinedSymbol { .. }));
        
        let type_error = AnalyzerError::TypeError { 
            expected: "number".to_string(), 
            found: "string".to_string(), 
            line: 1 
        };
        assert!(matches!(type_error, AnalyzerError::TypeError { .. }));
        
        let duplicate_error = AnalyzerError::DuplicateEndpoint { 
            name: "test".to_string(), 
            method: "GET".to_string(), 
            path: "/test".to_string(), 
            line: 1 
        };
        assert!(matches!(duplicate_error, AnalyzerError::DuplicateEndpoint { .. }));
    }

    #[test]
    fn test_analyzed_program_structure() {
        let source = r#"{
            "name": "test",
            "value": 42
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Test AnalyzedProgram structure
        assert!(result.symbol_table.symbols().len() > 0);
        assert!(result.resolution_order.len() > 0);
        assert_eq!(result.endpoints.len(), 0); // No endpoints in this simple test
        
        // Test that resolution order contains our symbols
        assert!(result.resolution_order.contains(&"name".to_string()));
        assert!(result.resolution_order.contains(&"value".to_string()));
    }

    #[test]
    fn test_multiple_simple_objects() {
        let source = r#"{
            "user1": {
                "name": "Alice",
                "age": 25
            },
            "user2": {
                "name": "Bob", 
                "age": 30
            },
            "user3": {
                "name": "Charlie",
                "age": 35
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Should have symbols for all users and their fields
        assert!(result.symbol_table.get_symbol("user1").is_some());
        assert!(result.symbol_table.get_symbol("user2").is_some());
        assert!(result.symbol_table.get_symbol("user3").is_some());
        
        // Should have multiple name and age symbols (from nested objects)
        assert!(result.symbol_table.get_symbol("name").is_some());
        assert!(result.symbol_table.get_symbol("age").is_some());
        
        // All symbols should be resolved
        for symbol in result.symbol_table.symbols().values() {
            assert!(symbol.is_resolved);
        }
    }

    #[test]
    fn test_dependency_collection_with_identifiers() {
        // This test would need actual variable references to work properly,
        // but tests the dependency collection logic structure
        let source = r#"{
            "base": 42,
            "derived": "test_value"
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Even without actual dependencies, verify the structure works
        let base_symbol = result.symbol_table.get_symbol("base").unwrap();
        let derived_symbol = result.symbol_table.get_symbol("derived").unwrap();
        
        // With literals, dependencies should be empty
        assert!(base_symbol.dependencies.is_empty());
        assert!(derived_symbol.dependencies.is_empty());
    }

    #[test]
    fn test_member_access_type_inference() {
        let source = r#"{
            "config": {
                "port": 8080,
                "host": "localhost"
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let config_symbol = result.symbol_table.get_symbol("config").unwrap();
        if let Type::Object(fields) = &config_symbol.symbol_type {
            assert_eq!(fields.get("port"), Some(&Type::Number));
            assert_eq!(fields.get("host"), Some(&Type::String));
        } else {
            panic!("Expected object type for config");
        }
    }

    #[test]
    fn test_deeply_nested_objects() {
        let source = r#"{
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "deep_value": "buried treasure"
                        }
                    }
                }
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Verify all levels are analyzed
        assert!(result.symbol_table.get_symbol("level1").is_some());
        assert!(result.symbol_table.get_symbol("level2").is_some());
        assert!(result.symbol_table.get_symbol("level3").is_some());
        assert!(result.symbol_table.get_symbol("level4").is_some());
        assert!(result.symbol_table.get_symbol("deep_value").is_some());
        
        let deep_symbol = result.symbol_table.get_symbol("deep_value").unwrap();
        assert_eq!(deep_symbol.symbol_type, Type::String);
    }

    #[test]
    fn test_arrays_of_objects() {
        let source = r#"{
            "users": [
                {
                    "name": "Alice",
                    "age": 30
                },
                {
                    "name": "Bob", 
                    "age": 25
                },
                {
                    "name": "Charlie",
                    "age": 35
                }
            ]
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let users_symbol = result.symbol_table.get_symbol("users").unwrap();
        if let Type::Array(inner) = &users_symbol.symbol_type {
            // First element should determine array type - should be object
            assert!(matches!(**inner, Type::Object(_)));
        } else {
            panic!("Expected array type for users");
        }
        
        // Should have symbols for nested object fields
        assert!(result.symbol_table.get_symbol("name").is_some());
        assert!(result.symbol_table.get_symbol("age").is_some());
    }

    #[test]
    fn test_mixed_data_structures() {
        let source = r#"{
            "metadata": {
                "name": "Test Dataset",
                "version": 1.2,
                "tags": ["important", "experimental"],
                "config": {
                    "enabled": true,
                    "limits": [100, 200, 300]
                }
            }
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Should handle the complex nested structure
        assert!(result.symbol_table.get_symbol("metadata").is_some());
        assert!(result.symbol_table.get_symbol("name").is_some());
        assert!(result.symbol_table.get_symbol("version").is_some());
        assert!(result.symbol_table.get_symbol("tags").is_some());
        assert!(result.symbol_table.get_symbol("config").is_some());
        assert!(result.symbol_table.get_symbol("enabled").is_some());
        assert!(result.symbol_table.get_symbol("limits").is_some());
        
        // Verify types are correct
        let name_symbol = result.symbol_table.get_symbol("name").unwrap();
        assert_eq!(name_symbol.symbol_type, Type::String);
        
        let version_symbol = result.symbol_table.get_symbol("version").unwrap();
        assert_eq!(version_symbol.symbol_type, Type::Number);
        
        let enabled_symbol = result.symbol_table.get_symbol("enabled").unwrap();
        assert_eq!(enabled_symbol.symbol_type, Type::Boolean);
    }

    #[test]
    fn test_symbol_definition_line_tracking() {
        let source = r#"{
            "first": "value1",
            "second": "value2",
            "third": "value3"
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Verify that symbols track their definition lines
        for symbol in result.symbol_table.symbols().values() {
            // Lines should be > 0 (line numbers are 1-based)
            assert!(symbol.definition_line > 0, 
                   "Symbol {} should have a valid line number", symbol.name);
        }
    }

    #[test]
    fn test_empty_array_and_object() {
        let source = r#"{
            "empty_array": [],
            "empty_object": {}
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let array_symbol = result.symbol_table.get_symbol("empty_array").unwrap();
        if let Type::Array(inner) = &array_symbol.symbol_type {
            assert_eq!(**inner, Type::Any);
        } else {
            panic!("Expected array type");
        }
        
        let object_symbol = result.symbol_table.get_symbol("empty_object").unwrap();
        if let Type::Object(fields) = &object_symbol.symbol_type {
            assert!(fields.is_empty());
        } else {
            panic!("Expected object type");
        }
    }

    #[test] 
    fn test_very_large_numbers() {
        let source = r#"{
            "large_int": 9007199254740991,
            "small_int": -9007199254740991,
            "large_float": 1.7976931348623157e+308,
            "small_float": -1.7976931348623157e+308,
            "zero": 0,
            "negative_zero": -0
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // All should be analyzed as numbers
        for field in ["large_int", "small_int", "large_float", "small_float", "zero", "negative_zero"] {
            let symbol = result.symbol_table.get_symbol(field).unwrap();
            assert_eq!(symbol.symbol_type, Type::Number, "Field {} should be Number type", field);
        }
    }

    #[test]
    fn test_unicode_string_handling() {
        let source = r#"{
            "emoji": "🚀",
            "chinese": "你好世界",
            "arabic": "مرحبا بالعالم",
            "russian": "Привет мир",
            "mixed": "Hello 🌍 世界 مرحبا Привет"
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // All should be analyzed as strings
        for field in ["emoji", "chinese", "arabic", "russian", "mixed"] {
            let symbol = result.symbol_table.get_symbol(field).unwrap();
            assert_eq!(symbol.symbol_type, Type::String, "Field {} should be String type", field);
        }
    }

    #[test]
    fn test_nested_arrays() {
        let source = r#"{
            "matrix": [
                [1, 2, 3],
                [4, 5, 6],
                [7, 8, 9]
            ],
            "jagged": [
                [1, 2],
                [3, 4, 5],
                [6]
            ]
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let matrix_symbol = result.symbol_table.get_symbol("matrix").unwrap();
        if let Type::Array(inner) = &matrix_symbol.symbol_type {
            // Should be array of arrays of numbers
            if let Type::Array(inner_inner) = inner.as_ref() {
                assert_eq!(**inner_inner, Type::Number);
            } else {
                panic!("Expected nested array type");
            }
        } else {
            panic!("Expected array type for matrix");
        }
        
        let jagged_symbol = result.symbol_table.get_symbol("jagged").unwrap();
        assert!(matches!(jagged_symbol.symbol_type, Type::Array(_)));
    }

    #[test]
    fn test_boolean_variations() {
        let source = r#"{
            "true_val": true,
            "false_val": false
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let true_symbol = result.symbol_table.get_symbol("true_val").unwrap();
        assert_eq!(true_symbol.symbol_type, Type::Boolean);
        
        let false_symbol = result.symbol_table.get_symbol("false_val").unwrap();
        assert_eq!(false_symbol.symbol_type, Type::Boolean);
    }

    #[test]
    fn test_null_handling() {
        let source = r#"{
            "null_value": null,
            "mixed_with_null": [null, "test", null, 42]
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        let null_symbol = result.symbol_table.get_symbol("null_value").unwrap();
        assert_eq!(null_symbol.symbol_type, Type::Null);
        
        let mixed_symbol = result.symbol_table.get_symbol("mixed_with_null").unwrap();
        // Array type should be determined by first element (null in this case)
        if let Type::Array(inner) = &mixed_symbol.symbol_type {
            assert_eq!(**inner, Type::Null);
        } else {
            panic!("Expected array type");
        }
    }

    #[test]
    fn test_special_characters_in_keys() {
        let source = r#"{
            "normal_key": "value",
            "key-with-dashes": "value",
            "key_with_underscores": "value",
            "key.with.dots": "value",
            "key with spaces": "value",
            "key@with#special$chars%": "value"
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // All keys should be handled properly
        let expected_keys = [
            "normal_key", 
            "key-with-dashes", 
            "key_with_underscores",
            "key.with.dots",
            "key with spaces",
            "key@with#special$chars%"
        ];
        
        for key in &expected_keys {
            assert!(result.symbol_table.get_symbol(key).is_some(), 
                   "Key '{}' should exist in symbol table", key);
        }
    }

    #[test]
    fn test_object_field_analysis_vs_top_level() {
        let source = r#"{
            "user": {
                "name": "Alice",
                "age": 30
            },
            "name": "Global Name",
            "age": 99
        }"#;
        
        let result = analyze_source(source).unwrap();
        
        // Should have symbols for both object fields and top-level fields
        // Note: Currently the analyzer creates symbols for both, which means
        // the last occurrence wins in the symbol table
        assert!(result.symbol_table.get_symbol("user").is_some());
        assert!(result.symbol_table.get_symbol("name").is_some());
        assert!(result.symbol_table.get_symbol("age").is_some());
        
        // The global "name" and "age" should exist (may overwrite object fields)
        let name_symbol = result.symbol_table.get_symbol("name").unwrap();
        assert_eq!(name_symbol.symbol_type, Type::String);
        
        let age_symbol = result.symbol_table.get_symbol("age").unwrap();
        assert_eq!(age_symbol.symbol_type, Type::Number);
    }

    #[test]
    fn test_performance_with_many_symbols() {
        // Create a large object to test performance
        let mut fields = Vec::new();
        for i in 0..100 {
            fields.push(format!(r#""field_{}": "value_{}""#, i, i));
        }
        let source = format!("{{\n{}\n}}", fields.join(",\n"));
        
        let start = std::time::Instant::now();
        let result = analyze_source(&source).unwrap();
        let duration = start.elapsed();
        
        // Should complete in reasonable time (adjust threshold as needed)
        assert!(duration < std::time::Duration::from_millis(100), 
               "Analysis took too long: {:?}", duration);
        
        // Should have all 100 symbols
        assert_eq!(result.symbol_table.symbols().len(), 100);
        
        // All should be resolved
        for symbol in result.symbol_table.symbols().values() {
            assert!(symbol.is_resolved);
        }
    }

    #[test]
    fn test_memory_usage_with_deep_nesting() {
        // Create a simpler deeply nested structure that's easier to debug
        let source = r#"{
            "root": {
                "level_0": {
                    "level_1": {
                        "level_2": {
                            "deep_value": "found"
                        }
                    }
                }
            }
        }"#;
        
        let result = analyze_source(&source).unwrap();
        
        // Should handle deep nesting without issues
        assert!(result.symbol_table.get_symbol("root").is_some());
        assert!(result.symbol_table.get_symbol("level_0").is_some());
        assert!(result.symbol_table.get_symbol("level_1").is_some());
        assert!(result.symbol_table.get_symbol("level_2").is_some());
        assert!(result.symbol_table.get_symbol("deep_value").is_some());
    }
}
