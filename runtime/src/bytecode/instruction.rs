use crate::bytecode::OpCode;

/// Represents a single bytecode instruction with its operands
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operands: Vec<u32>,  // Operands can be indexes into constant pool, registers, etc.
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operands: Vec::new(),
        }
    }

    pub fn with_operand(mut self, operand: u32) -> Self {
        self.operands.push(operand);
        self
    }

    pub fn with_operands(mut self, operands: Vec<u32>) -> Self {
        self.operands.extend(operands);
        self
    }
}