use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    
    // Identifiers and Keywords
    Identifier(String),
    
    // JSON Structure
    LeftBrace,     // {
    RightBrace,    // }
    LeftBracket,   // [
    RightBracket,  // ]
    LeftParen,     // (
    RightParen,    // )
    Comma,         // ,
    Colon,         // :
    Dot,           // .
    
    // Operators
    Plus,          // +
    Minus,         // -
    Multiply,      // *
    Divide,        // /
    Equal,         // =
    
    // Special Keywords
    Promise,       // @promise
    Endpoint,      // @endpoint
    Method,        // @method
    Http,          // @http
    
    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::String(s) => write!(f, "\"{}\"", s),
            TokenType::Number(n) => write!(f, "{}", n),
            TokenType::Boolean(b) => write!(f, "{}", b),
            TokenType::Null => write!(f, "null"),
            TokenType::Identifier(s) => write!(f, "{}", s),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::Comma => write!(f, ","),
            TokenType::Colon => write!(f, ":"),
            TokenType::Dot => write!(f, "."),
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Multiply => write!(f, "*"),
            TokenType::Divide => write!(f, "/"),
            TokenType::Equal => write!(f, "="),
            TokenType::Promise => write!(f, "@promise"),
            TokenType::Endpoint => write!(f, "@endpoint"),
            TokenType::Method => write!(f, "@method"),
            TokenType::Http => write!(f, "@http"),
            TokenType::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(Token::new(TokenType::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        let ch = self.advance();
        
        let token_type = match ch {
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            ',' => TokenType::Comma,
            ':' => TokenType::Colon,
            '.' => TokenType::Dot,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Multiply,
            '/' => TokenType::Divide,
            '=' => TokenType::Equal,
            '"' => self.string()?,
            '@' => self.annotation()?,
            _ if ch.is_ascii_digit() || ch == '.' => {
                self.position -= 1;
                self.column -= 1;
                self.number()?
            }
            _ if ch.is_ascii_alphabetic() || ch == '_' => {
                self.position -= 1;
                self.column -= 1;
                self.identifier_or_keyword()?
            }
            _ => return Err(LexerError::UnexpectedCharacter(ch, self.line, self.column)),
        };
        
        Ok(Token::new(token_type, start_line, start_column))
    }

    fn string(&mut self) -> Result<TokenType, LexerError> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.advance();
            if ch == '\\' {
                if self.is_at_end() {
                    return Err(LexerError::UnterminatedString(self.line, self.column));
                }
                let escaped = self.advance();
                match escaped {
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    '/' => value.push('/'),
                    'b' => value.push('\x08'),
                    'f' => value.push('\x0C'),
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    'u' => {
                        // Unicode escape sequence
                        let mut unicode = String::new();
                        for _ in 0..4 {
                            if self.is_at_end() {
                                return Err(LexerError::UnterminatedString(self.line, self.column));
                            }
                            unicode.push(self.advance());
                        }
                        if let Ok(code_point) = u32::from_str_radix(&unicode, 16) {
                            if let Some(ch) = char::from_u32(code_point) {
                                value.push(ch);
                            } else {
                                return Err(LexerError::InvalidUnicodeEscape(self.line, self.column));
                            }
                        } else {
                            return Err(LexerError::InvalidUnicodeEscape(self.line, self.column));
                        }
                    }
                    _ => return Err(LexerError::InvalidEscapeSequence(escaped, self.line, self.column)),
                }
            } else {
                value.push(ch);
            }
        }
        
        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(self.line, self.column));
        }
        
        self.advance(); // Consume closing quote
        Ok(TokenType::String(value))
    }

    fn annotation(&mut self) -> Result<TokenType, LexerError> {
        let mut annotation = String::from("@");
        
        // Read the full annotation including any special characters
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_whitespace() || ch == ',' || ch == '}' || ch == ')' || ch == ']' {
                break;
            }
            annotation.push(self.advance());
        }
        
        // Return as a special identifier
        Ok(TokenType::Identifier(annotation))
    }

    fn number(&mut self) -> Result<TokenType, LexerError> {
        let mut number_str = String::new();
        
        // Handle negative numbers
        if self.peek() == '-' {
            number_str.push(self.advance());
        }
        
        // Integer part
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            number_str.push(self.advance());
        }
        
        // Decimal part
        if !self.is_at_end() && self.peek() == '.' {
            number_str.push(self.advance());
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                number_str.push(self.advance());
            }
        }
        
        // Exponential part
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            number_str.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                number_str.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                number_str.push(self.advance());
            }
        }
        
        number_str.parse::<f64>()
            .map(TokenType::Number)
            .map_err(|_| LexerError::InvalidNumber(number_str, self.line, self.column))
    }

    fn identifier_or_keyword(&mut self) -> Result<TokenType, LexerError> {
        let mut identifier = String::new();
        
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            identifier.push(self.advance());
        }
        
        match identifier.as_str() {
            "true" => Ok(TokenType::Boolean(true)),
            "false" => Ok(TokenType::Boolean(false)),
            "null" => Ok(TokenType::Null),
            _ => Ok(TokenType::Identifier(identifier)),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }
                '/' if self.peek_next() == Some('/') => {
                    // Line comment
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                '/' if self.peek_next() == Some('*') => {
                    // Block comment
                    self.advance(); // consume '/'
                    self.advance(); // consume '*'
                    while !self.is_at_end() {
                        if self.peek() == '*' && self.peek_next() == Some('/') {
                            self.advance(); // consume '*'
                            self.advance(); // consume '/'
                            break;
                        }
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.column = 1;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> char {
        let ch = self.input[self.position];
        self.position += 1;
        self.column += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("Unexpected character '{0}' at line {1}, column {2}")]
    UnexpectedCharacter(char, usize, usize),
    
    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),
    
    #[error("Invalid escape sequence '\\{0}' at line {1}, column {2}")]
    InvalidEscapeSequence(char, usize, usize),
    
    #[error("Invalid unicode escape at line {0}, column {1}")]
    InvalidUnicodeEscape(usize, usize),
    
    #[error("Invalid number '{0}' at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),
    
    #[error("Unknown annotation '@{0}' at line {1}, column {2}")]
    UnknownAnnotation(String, usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_json_tokens() {
        let mut lexer = Lexer::new(r#"{"key": "value", "number": 42, "bool": true}"#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::LeftBrace);
        assert_eq!(tokens[1].token_type, TokenType::String("key".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Colon);
        assert_eq!(tokens[3].token_type, TokenType::String("value".to_string()));
    }

    #[test]
    fn test_annotations() {
        let mut lexer = Lexer::new("@endpoint @promise @method");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Identifier("@endpoint".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("@promise".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Identifier("@method".to_string()));
    }

    #[test]
    fn test_mathematical_expressions() {
        let mut lexer = Lexer::new("total = subtotal + tax");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Identifier("total".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Equal);
        assert_eq!(tokens[2].token_type, TokenType::Identifier("subtotal".to_string()));
        assert_eq!(tokens[3].token_type, TokenType::Plus);
        assert_eq!(tokens[4].token_type, TokenType::Identifier("tax".to_string()));
    }

    #[test]
    fn test_complex_annotations() {
        let mut lexer = Lexer::new("@endpoint:GET:/api/users @promise:fetch_user");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Identifier("@endpoint:GET:/api/users".to_string()));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("@promise:fetch_user".to_string()));
    }
}
