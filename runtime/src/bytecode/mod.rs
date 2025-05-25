mod instruction;
mod opcode;
mod parser;

pub use instruction::Instruction;
pub use opcode::OpCode;
pub use parser::{Parser, ParseError};

/// Represents a constant value in the bytecode
#[derive(Debug, Clone)]
pub enum Constant {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// Represents a bytecode module
#[derive(Debug, Clone)]
pub struct BytecodeModule {
    pub name: String,
    pub code_page: Vec<u8>,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub address_map: std::collections::HashMap<u32, usize>,
}

impl BytecodeModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            code_page: Vec::new(),
            instructions: Vec::new(),
            constants: Vec::new(),
            address_map: std::collections::HashMap::new(),
        }
    }
}