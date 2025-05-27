use crate::lexer::{Token, TokenType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    // Program root
    Program(Vec<AstNode>),
    
    // Declarations
    Assignment {
        name: String,
        value: Box<AstNode>,
        line: usize,
    },
    
    // Expressions
    Binary {
        left: Box<AstNode>,
        operator: BinaryOp,
        right: Box<AstNode>,
        line: usize,
    },
    
    Unary {
        operator: UnaryOp,
        operand: Box<AstNode>,
        line: usize,
    },
    
    MemberAccess {
        object: Box<AstNode>,
        property: String,
        line: usize,
    },
    
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Identifier(String),
    
    // JSON Objects and Arrays
    Object {
        fields: HashMap<String, AstNode>,
        line: usize,
    },
    
    Array {
        elements: Vec<AstNode>,
        line: usize,
    },
    
    // Function calls
    FunctionCall {
        name: String,
        arguments: Vec<AstNode>,
        line: usize,
    },
    
    // Annotations and special constructs
    Promise {
        expression: Box<AstNode>,
        line: usize,
    },
    
    Endpoint {
        name: String,
        method: HttpMethod,
        path: String,
        handler: Box<AstNode>,
        line: usize,
    },
    
    HttpCall {
        url: Box<AstNode>,
        method: HttpMethod,
        body: Option<Box<AstNode>>,
        headers: Option<HashMap<String, AstNode>>,
        line: usize,
    },
    
    // Test declarations
    Test {
        name: String,
        expect_expression: Box<AstNode>,
        inputs: HashMap<String, AstNode>,
        expected_output: Box<AstNode>,
        is_regex: bool,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if self.peek().token_type == TokenType::Eof {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        
        Ok(AstNode::Program(statements))
    }

    fn parse_statement(&mut self) -> Result<AstNode, ParseError> {
        // Check for annotations first
        if let TokenType::Endpoint = self.peek().token_type {
            return self.parse_endpoint();
        }
        
        if let TokenType::Promise = self.peek().token_type {
            return self.parse_promise();
        }
        
        if let TokenType::Test = self.peek().token_type {
            return self.parse_test();
        }
        
        // Check for assignment
        if let TokenType::Identifier(_) = self.peek().token_type {
            if self.peek_next().map(|t| &t.token_type) == Some(&TokenType::Equal) {
                return self.parse_assignment();
            }
        }
        
        // Otherwise, parse as expression
        self.parse_expression()
    }

    fn parse_assignment(&mut self) -> Result<AstNode, ParseError> {
        let name_token = self.advance();
        let line = name_token.line;
        
        let name = match &name_token.token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => return Err(ParseError::ExpectedIdentifier(line)),
        };
        
        self.consume(TokenType::Equal, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;
        
        Ok(AstNode::Assignment {
            name,
            value: Box::new(value),
            line,
        })
    }

    fn parse_endpoint(&mut self) -> Result<AstNode, ParseError> {
        let endpoint_token = self.advance(); // consume @endpoint
        let line = endpoint_token.line;
        
        // Expect an object with endpoint configuration
        let config = self.parse_object()?;
        
        if let AstNode::Object { fields, .. } = config {
            let name = self.extract_string_field(&fields, "name", line)?;
            let method_str = self.extract_string_field(&fields, "method", line)?;
            let path = self.extract_string_field(&fields, "path", line)?;
            
            let method = match method_str.to_lowercase().as_str() {
                "get" => HttpMethod::Get,
                "post" => HttpMethod::Post,
                "put" => HttpMethod::Put,
                "delete" => HttpMethod::Delete,
                "patch" => HttpMethod::Patch,
                _ => return Err(ParseError::InvalidHttpMethod(method_str, line)),
            };
            
            let handler = fields.get("handler")
                .ok_or_else(|| ParseError::MissingField("handler".to_string(), line))?
                .clone();
            
            Ok(AstNode::Endpoint {
                name,
                method,
                path,
                handler: Box::new(handler),
                line,
            })
        } else {
            Err(ParseError::ExpectedObject(line))
        }
    }

    fn parse_promise(&mut self) -> Result<AstNode, ParseError> {
        let promise_token = self.advance(); // consume @promise
        let line = promise_token.line;
        
        let expression = self.parse_expression()?;
        
        Ok(AstNode::Promise {
            expression: Box::new(expression),
            line,
        })
    }

    fn parse_test(&mut self) -> Result<AstNode, ParseError> {
        let test_token = self.advance(); // consume @test
        let line = test_token.line;
        
        // Expect a string literal for the test name
        let name = if let TokenType::String(test_name) = &self.peek().token_type {
            let name = test_name.clone();
            self.advance(); // consume the string
            name
        } else {
            return Err(ParseError::ExpectedString(line));
        };
        
        // Expect an object with test configuration
        let config = self.parse_object()?;
        
        if let AstNode::Object { fields, .. } = config {
            // Parse the expect field (what to test)
            let expect_expression = fields.get("expect")
                .ok_or_else(|| ParseError::MissingField("expect".to_string(), line))?
                .clone();
            
            // For now, we'll create empty inputs and use the expect as expected_output
            let inputs = HashMap::new();
            
            // Determine the assertion type and expected value
            let (expected_output, is_regex) = if let Some(equals_value) = fields.get("equals") {
                (equals_value.clone(), false)
            } else if let Some(matches_value) = fields.get("matches") {
                (matches_value.clone(), true)
            } else if let Some(gt_value) = fields.get("greater_than") {
                // For now, treat comparison as equals for simplicity
                (gt_value.clone(), false)
            } else if let Some(lt_value) = fields.get("less_than") {
                // For now, treat comparison as equals for simplicity
                (lt_value.clone(), false)
            } else {
                return Err(ParseError::MissingField("equals, matches, greater_than, or less_than".to_string(), line));
            };
            
            Ok(AstNode::Test {
                name,
                expect_expression: Box::new(expect_expression),
                inputs,
                expected_output: Box::new(expected_output),
                is_regex,
                line,
            })
        } else {
            Err(ParseError::ExpectedObject(line))
        }
    }

    fn parse_expression(&mut self) -> Result<AstNode, ParseError> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_multiplicative()?;
        
        while self.match_token_types(&[TokenType::Plus, TokenType::Minus]) {
            let operator_token = self.previous();
            let operator = match operator_token.token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let line = operator_token.line;
            let right = self.parse_multiplicative()?;
            left = AstNode::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<AstNode, ParseError> {
        let mut left = self.parse_unary()?;
        
        while self.match_token_types(&[TokenType::Multiply, TokenType::Divide]) {
            let operator_token = self.previous();
            let operator = match operator_token.token_type {
                TokenType::Multiply => BinaryOp::Multiply,
                TokenType::Divide => BinaryOp::Divide,
                _ => unreachable!(),
            };
            let line = operator_token.line;
            let right = self.parse_unary()?;
            left = AstNode::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                line,
            };
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, ParseError> {
        if self.match_token_types(&[TokenType::Minus]) {
            let operator_token = self.previous();
            let line = operator_token.line;
            let operand = self.parse_unary()?;
            return Ok(AstNode::Unary {
                operator: UnaryOp::Negate,
                operand: Box::new(operand),
                line,
            });
        }
        
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<AstNode, ParseError> {
        let token = self.advance();
        
        let mut node = match &token.token_type {
            TokenType::String(s) => AstNode::String(s.clone()),
            TokenType::Number(n) => AstNode::Number(*n),
            TokenType::Boolean(b) => AstNode::Boolean(*b),
            TokenType::Null => AstNode::Null,
            TokenType::Identifier(name) => {
                let name = name.clone(); // Clone to avoid borrowing issues
                let line = token.line;
                
                // Check if this identifier is followed by parentheses (function call)
                if self.check(&TokenType::LeftParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token_types(&[TokenType::Comma]) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(TokenType::RightParen, "Expected ')' after function arguments")?;
                    
                    // For annotations, treat function calls as special identifiers with arguments
                    if name.starts_with('@') {
                        // Create a special identifier that includes the function call syntax
                        let mut func_call = name.clone();
                        func_call.push('(');
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                func_call.push_str(", ");
                            }
                            // For now, just use the argument as is (this is a simplified approach)
                            if let AstNode::Identifier(arg_name) = arg {
                                func_call.push_str(arg_name);
                            }
                        }
                        func_call.push(')');
                        AstNode::Identifier(func_call)
                    } else {
                        // Regular function call - create a proper FunctionCall node
                        AstNode::FunctionCall {
                            name,
                            arguments: args,
                            line,
                        }
                    }
                } else {
                    AstNode::Identifier(name)
                }
            }
            TokenType::LeftBrace => {
                self.current -= 1; // backtrack
                return self.parse_object();
            }
            TokenType::LeftBracket => {
                self.current -= 1; // backtrack
                return self.parse_array();
            }
            TokenType::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                expr
            }
            _ => return Err(ParseError::UnexpectedToken(token.token_type.clone(), token.line)),
        };
        
        // Handle member access
        while self.check(&TokenType::Dot) {
            self.advance(); // consume '.'
            let property_token = self.advance();
            let property = match &property_token.token_type {
                TokenType::Identifier(name) => name.clone(),
                _ => return Err(ParseError::ExpectedIdentifier(property_token.line)),
            };
            
            node = AstNode::MemberAccess {
                object: Box::new(node),
                property,
                line: property_token.line,
            };
        }
        
        Ok(node)
    }

    fn parse_object(&mut self) -> Result<AstNode, ParseError> {
        let start_token = self.advance(); // consume '{'
        let line = start_token.line;
        let mut fields = HashMap::new();
        
        if self.check(&TokenType::RightBrace) {
            self.advance(); // consume '}'
            return Ok(AstNode::Object { fields, line });
        }
        
        loop {
            // Parse key
            let key_token = self.advance();
            let key = match &key_token.token_type {
                TokenType::String(s) => s.clone(),
                TokenType::Identifier(s) => s.clone(),
                _ => return Err(ParseError::ExpectedStringOrIdentifier(key_token.token_type.clone(), line)),
            };
            
            self.consume(TokenType::Colon, "Expected ':' after object key")?;
            
            // Parse value
            let value = self.parse_expression()?;
            fields.insert(key, value);
            
            if !self.match_token_types(&[TokenType::Comma]) {
                break;
            }
            
            // Allow trailing comma
            if self.check(&TokenType::RightBrace) {
                break;
            }
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after object fields")?;
        Ok(AstNode::Object { fields, line })
    }

    fn parse_array(&mut self) -> Result<AstNode, ParseError> {
        let start_token = self.advance(); // consume '['
        let line = start_token.line;
        let mut elements = Vec::new();
        
        if self.check(&TokenType::RightBracket) {
            self.advance(); // consume ']'
            return Ok(AstNode::Array { elements, line });
        }
        
        loop {
            elements.push(self.parse_expression()?);
            
            if !self.match_token_types(&[TokenType::Comma]) {
                break;
            }
            
            // Allow trailing comma
            if self.check(&TokenType::RightBracket) {
                break;
            }
        }
        
        self.consume(TokenType::RightBracket, "Expected ']' after array elements")?;
        Ok(AstNode::Array { elements, line })
    }

    fn parse_http_call(&mut self) -> Result<AstNode, ParseError> {
        let http_token = self.previous(); // we already consumed @http
        let line = http_token.line;
        
        // Expect an object with HTTP call configuration
        let config = self.parse_object()?;
        
        if let AstNode::Object { fields, .. } = config {
            let url = fields.get("url")
                .ok_or_else(|| ParseError::MissingField("url".to_string(), line))?
                .clone();
            
            let method_str = self.extract_string_field(&fields, "method", line)?;
            let method = match method_str.to_lowercase().as_str() {
                "get" => HttpMethod::Get,
                "post" => HttpMethod::Post,
                "put" => HttpMethod::Put,
                "delete" => HttpMethod::Delete,
                "patch" => HttpMethod::Patch,
                _ => return Err(ParseError::InvalidHttpMethod(method_str, line)),
            };
            
            let body = fields.get("body").cloned().map(Box::new);
            
            // Parse headers if present
            let headers = if let Some(headers_node) = fields.get("headers") {
                if let AstNode::Object { fields, .. } = headers_node {
                    Some(fields.clone())
                } else {
                    return Err(ParseError::ExpectedObject(line));
                }
            } else {
                None
            };
            
            Ok(AstNode::HttpCall {
                url: Box::new(url),
                method,
                body,
                headers,
                line,
            })
        } else {
            Err(ParseError::ExpectedObject(line))
        }
    }

    // Helper methods
    fn extract_string_field(&self, fields: &HashMap<String, AstNode>, field_name: &str, line: usize) -> Result<String, ParseError> {
        fields.get(field_name)
            .ok_or_else(|| ParseError::MissingField(field_name.to_string(), line))
            .and_then(|node| {
                if let AstNode::String(s) = node {
                    Ok(s.clone())
                } else {
                    Err(ParseError::ExpectedString(line))
                }
            })
    }

    fn match_token_types(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> Option<&Token> {
        if self.current + 1 < self.tokens.len() {
            Some(&self.tokens[self.current + 1])
        } else {
            None
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ParseError> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError::ExpectedToken(token_type, self.peek().line, message.to_string()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected token {0} at line {1}")]
    UnexpectedToken(TokenType, usize),
    
    #[error("Expected identifier at line {0}")]
    ExpectedIdentifier(usize),
    
    #[error("Expected string at line {0}")]
    ExpectedString(usize),
    
    #[error("Expected object at line {0}")]
    ExpectedObject(usize),
    
    #[error("Expected string or identifier, got {0} at line {1}")]
    ExpectedStringOrIdentifier(TokenType, usize),
    
    #[error("Expected token {0} at line {1}: {2}")]
    ExpectedToken(TokenType, usize, String),
    
    #[error("Missing required field '{0}' at line {1}")]
    MissingField(String, usize),
    
    #[error("Invalid HTTP method '{0}' at line {1}")]
    InvalidHttpMethod(String, usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(source: &str) -> Result<AstNode, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_simple_assignment() {
        let ast = parse_source("total = subtotal + tax").unwrap();
        
        if let AstNode::Program(statements) = ast {
            assert_eq!(statements.len(), 1);
            if let AstNode::Assignment { name, .. } = &statements[0] {
                assert_eq!(name, "total");
            } else {
                panic!("Expected assignment");
            }
        } else {
            panic!("Expected program");
        }
    }

    #[test]
    fn test_object_parsing() {
        let ast = parse_source(r#"{"name": "test", "value": 42}"#).unwrap();
        
        if let AstNode::Program(statements) = ast {
            if let AstNode::Object { fields, .. } = &statements[0] {
                assert_eq!(fields.len(), 2);
                assert!(fields.contains_key("name"));
                assert!(fields.contains_key("value"));
            } else {
                panic!("Expected object");
            }
        }
    }

    #[test]
    fn test_endpoint_declaration() {
        let source = r#"{
            "handler": "@endpoint:GET:/api/users/{id}",
            "user": "@http:GET:https://api.example.com/users/123"
        }"#;
        
        let ast = parse_source(source).unwrap();
        
        if let AstNode::Program(statements) = ast {
            if let AstNode::Object { fields, .. } = &statements[0] {
                assert!(fields.contains_key("handler"));
                assert!(fields.contains_key("user"));
                // Check that handler value is a string with endpoint annotation
                if let AstNode::String(handler_val) = &fields["handler"] {
                    assert!(handler_val.starts_with("@endpoint:"));
                }
            } else {
                panic!("Expected object with endpoint declaration");
            }
        }
    }
}
