/// Opcodes for the VM

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Control operations
    Nop = 0x00,        // No operation
    Halt = 0x01,       // End execution

    // IO operations
    Print = 0x02,      // Print top value

    // Stack manipulation
    Push = 0x10,       // Push constant onto stack (1 operand)
    Pop = 0x11,        // Pop top value from stack
    Dup = 0x12,        // Duplicate top value
    Swap = 0x13,       // Swap top two values
    
    // Arithmetic operations
    Add = 0x20,        // Add top two values
    Sub = 0x21,        // Subtract top value from second top value
    Mul = 0x22,        // Multiply top two values
    Div = 0x23,        // Divide/Modulo second top value by top value (quotient on top)
    Neg = 0x24,        // Negate top value
    
    // Bitwise operations
    BitAnd = 0x30,     // Bitwise AND
    BitOr = 0x31,      // Bitwise OR
    BitXor = 0x32,     // Bitwise XOR
    BitNot = 0x33,     // Bitwise NOT
    ShiftLeft = 0x34,  // Bitwise shift left
    ShiftRight = 0x35, // Bitwise shift right
    RotateLeft = 0x36, // Rotate left
    RotateRight = 0x37,// Rotate right
    
    // Logical operations
    And = 0x40,        // Logical AND
    Or = 0x41,         // Logical OR
    Not = 0x42,        // Logical NOT
    
    // Comparison operations
    Eq = 0x50,         // Equality comparison
    Neq = 0x51,        // Not equal comparison
    Lt = 0x52,         // Less than
    Lte = 0x53,        // Less than or equal
    Gt = 0x54,         // Greater than
    Gte = 0x55,        // Greater than or equal
    
    // Control flow
    Jump = 0x60,       // Jump to location (1 operand)
    JumpIf = 0x61,     // Jump if condition is true (1 operand)
    JumpIfNot = 0x62,  // Jump if condition is false (1 operand)
    Call = 0x63,       // Call a function (1 operand)
    Return = 0x64,     // Return from function
    
    // Exception handling
    TryBlock = 0x6A,   // Start of try block, with target catch/finally blocks (3 operands)
    CatchBlock = 0x6B, // Start of catch block
    FinallyBlock = 0x6C, // Start of finally block
    EndTry = 0x6D,     // End of try/catch/finally
    Throw = 0x6E,      // Throw an exception
    Rethrow = 0x6F,    // Rethrow the current exception
    
    // Local variables
    StoreLocal = 0x70, // Store top value in local variable slot (1 operand)
    LoadLocal = 0x71,  // Load local variable onto stack (1 operand)
    
    // Heap operations
    CreateHeap = 0x80, // Create a new heap
    SwitchHeap = 0x81, // Switch to a different heap
    CollectHeap = 0x82,// Force garbage collection on a heap

    // Array operations
    NewArray = 0x90,   // Create a new array with n elements from stack (1 operand)
    GetElement = 0x91, // Get element from array at index
    SetElement = 0x92, // Set element in array at index
    ArrayLength = 0x93,// Get length of array
}

const NOP: u8 = OpCode::Nop as u8;
const HALT: u8 = OpCode::Halt as u8;
const PRINT: u8 = OpCode::Print as u8;

const PUSH: u8 = OpCode::Push as u8;
const POP: u8 = OpCode::Pop as u8;
const DUP: u8 = OpCode::Dup as u8;
const SWAP: u8 = OpCode::Swap as u8;

const ADD: u8 = OpCode::Add as u8;
const SUB: u8 = OpCode::Sub as u8;
const MUL: u8 = OpCode::Mul as u8;
const DIV: u8 = OpCode::Div as u8;
const NEG: u8 = OpCode::Neg as u8;

const BIT_AND: u8 = OpCode::BitAnd as u8;
const BIT_OR: u8 = OpCode::BitOr as u8;
const BIT_XOR: u8 = OpCode::BitXor as u8;
const BIT_NOT: u8 = OpCode::BitNot as u8;
const SHIFT_LEFT: u8 = OpCode::ShiftLeft as u8;
const SHIFT_RIGHT: u8 = OpCode::ShiftRight as u8;
const ROTATE_LEFT: u8 = OpCode::RotateLeft as u8;
const ROTATE_RIGHT: u8 = OpCode::RotateRight as u8;

const AND: u8 = OpCode::And as u8;
const OR: u8 = OpCode::Or as u8;
const NOT: u8 = OpCode::Not as u8;

const EQ: u8 = OpCode::Eq as u8;
const NEQ: u8 = OpCode::Neq as u8;
const LT: u8 = OpCode::Lt as u8;
const LTE: u8 = OpCode::Lte as u8;
const GT: u8 = OpCode::Gt as u8;
const GTE: u8 = OpCode::Gte as u8;

const JUMP: u8 = OpCode::Jump as u8;
const JUMP_IF: u8 = OpCode::JumpIf as u8;
const JUMP_IF_NOT: u8 = OpCode::JumpIfNot as u8;
const CALL: u8 = OpCode::Call as u8;
const RETURN: u8 = OpCode::Return as u8;

// Exception handling
const TRY_BLOCK: u8 = OpCode::TryBlock as u8;
const CATCH_BLOCK: u8 = OpCode::CatchBlock as u8;
const FINALLY_BLOCK: u8 = OpCode::FinallyBlock as u8;
const END_TRY: u8 = OpCode::EndTry as u8;
const THROW: u8 = OpCode::Throw as u8;
const RETHROW: u8 = OpCode::Rethrow as u8;

const NEW_ARRAY: u8 = OpCode::NewArray as u8;
const GET_ELEMENT: u8 = OpCode::GetElement as u8;
const SET_ELEMENT: u8 = OpCode::SetElement as u8;
const ARRAY_LENGTH: u8 = OpCode::ArrayLength as u8;

const STORE_LOCAL: u8 = OpCode::StoreLocal as u8;
const LOAD_LOCAL: u8 = OpCode::LoadLocal as u8;

const CREATE_HEAP: u8 = OpCode::CreateHeap as u8;
const SWITCH_HEAP: u8 = OpCode::SwitchHeap as u8;
const COLLECT_HEAP: u8 = OpCode::CollectHeap as u8;

impl OpCode {
    /// Convert a byte to an opcode
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            NOP => Some(OpCode::Nop),
            HALT => Some(OpCode::Halt),
            PRINT => Some(OpCode::Print),

            PUSH => Some(OpCode::Push),
            POP => Some(OpCode::Pop),
            DUP => Some(OpCode::Dup),
            SWAP => Some(OpCode::Swap),

            ADD => Some(OpCode::Add),
            SUB => Some(OpCode::Sub),
            MUL => Some(OpCode::Mul),
            DIV => Some(OpCode::Div),
            NEG => Some(OpCode::Neg),

            BIT_AND => Some(OpCode::BitAnd),
            BIT_OR => Some(OpCode::BitOr),
            BIT_XOR => Some(OpCode::BitXor),
            BIT_NOT => Some(OpCode::BitNot),
            SHIFT_LEFT => Some(OpCode::ShiftLeft),
            SHIFT_RIGHT => Some(OpCode::ShiftRight),
            ROTATE_LEFT => Some(OpCode::RotateLeft),
            ROTATE_RIGHT => Some(OpCode::RotateRight),

            AND => Some(OpCode::And),
            OR => Some(OpCode::Or),
            NOT => Some(OpCode::Not),
            EQ => Some(OpCode::Eq),
            NEQ => Some(OpCode::Neq),
            LT => Some(OpCode::Lt),
            LTE => Some(OpCode::Lte),
            GT => Some(OpCode::Gt),
            GTE => Some(OpCode::Gte),

            JUMP => Some(OpCode::Jump),
            JUMP_IF => Some(OpCode::JumpIf),
            JUMP_IF_NOT => Some(OpCode::JumpIfNot),
            CALL => Some(OpCode::Call),
            RETURN => Some(OpCode::Return),
            
            // Exception handling opcodes
            TRY_BLOCK => Some(OpCode::TryBlock),
            CATCH_BLOCK => Some(OpCode::CatchBlock),
            FINALLY_BLOCK => Some(OpCode::FinallyBlock),
            END_TRY => Some(OpCode::EndTry),
            THROW => Some(OpCode::Throw),
            RETHROW => Some(OpCode::Rethrow),

            STORE_LOCAL => Some(OpCode::StoreLocal),
            LOAD_LOCAL => Some(OpCode::LoadLocal),

            CREATE_HEAP => Some(OpCode::CreateHeap),
            SWITCH_HEAP => Some(OpCode::SwitchHeap),
            COLLECT_HEAP => Some(OpCode::CollectHeap),

            NEW_ARRAY => Some(OpCode::NewArray),
            GET_ELEMENT => Some(OpCode::GetElement),
            SET_ELEMENT => Some(OpCode::SetElement),
            ARRAY_LENGTH => Some(OpCode::ArrayLength),

            _ => None,
        }
    }
    
    /// Convert an opcode to a byte
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }

    /// Get the number of operands for the opcode
    pub fn num_operands(&self) -> usize {
        match self {
            OpCode::Push |
            OpCode::Jump | OpCode::JumpIf | OpCode::JumpIfNot | OpCode::Call |
            OpCode::StoreLocal | OpCode::LoadLocal |
            OpCode::CollectHeap |
            OpCode::NewArray => 1,

            OpCode::TryBlock => 3, // catch_pc, finally_pc, end_try_pc

            _ => 0,
        }
    }
}

// Add From/Into trait implementations
impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        OpCode::from_byte(byte).unwrap_or(OpCode::Nop)
    }
}

impl From<OpCode> for u8 {
    fn from(opcode: OpCode) -> Self {
        opcode.to_byte()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_byte_conversion() {
        // Test control operations
        assert_eq!(OpCode::Nop.to_byte(), 0x00);
        assert_eq!(OpCode::Halt.to_byte(), 0x01);
        assert_eq!(OpCode::Print.to_byte(), 0x02);

        // Test stack operations
        assert_eq!(OpCode::Push.to_byte(), 0x10);
        assert_eq!(OpCode::Pop.to_byte(), 0x11);
        assert_eq!(OpCode::Dup.to_byte(), 0x12);
        assert_eq!(OpCode::Swap.to_byte(), 0x13);

        // Test arithmetic operations
        assert_eq!(OpCode::Add.to_byte(), 0x20);
        assert_eq!(OpCode::Sub.to_byte(), 0x21);
        assert_eq!(OpCode::Mul.to_byte(), 0x22);
        assert_eq!(OpCode::Div.to_byte(), 0x23);
        assert_eq!(OpCode::Neg.to_byte(), 0x24);

        // Test exception handling
        assert_eq!(OpCode::TryBlock.to_byte(), 0x6A);
        assert_eq!(OpCode::CatchBlock.to_byte(), 0x6B);
        assert_eq!(OpCode::FinallyBlock.to_byte(), 0x6C);
        assert_eq!(OpCode::EndTry.to_byte(), 0x6D);
        assert_eq!(OpCode::Throw.to_byte(), 0x6E);
        assert_eq!(OpCode::Rethrow.to_byte(), 0x6F);
    }

    #[test]
    fn test_byte_to_opcode_conversion() {
        // Test valid conversions
        assert_eq!(OpCode::from_byte(0x00), Some(OpCode::Nop));
        assert_eq!(OpCode::from_byte(0x01), Some(OpCode::Halt));
        assert_eq!(OpCode::from_byte(0x10), Some(OpCode::Push));
        assert_eq!(OpCode::from_byte(0x20), Some(OpCode::Add));
        assert_eq!(OpCode::from_byte(0x6A), Some(OpCode::TryBlock));
        assert_eq!(OpCode::from_byte(0x90), Some(OpCode::NewArray));

        // Test invalid conversion
        assert_eq!(OpCode::from_byte(0xFF), None);
        assert_eq!(OpCode::from_byte(0x99), None);
    }

    #[test]
    fn test_operand_counts() {
        // Test opcodes with 0 operands
        assert_eq!(OpCode::Nop.num_operands(), 0);
        assert_eq!(OpCode::Halt.num_operands(), 0);
        assert_eq!(OpCode::Add.num_operands(), 0);
        assert_eq!(OpCode::Pop.num_operands(), 0);
        assert_eq!(OpCode::CatchBlock.num_operands(), 0);
        assert_eq!(OpCode::EndTry.num_operands(), 0);

        // Test opcodes with 1 operand
        assert_eq!(OpCode::Push.num_operands(), 1);
        assert_eq!(OpCode::Jump.num_operands(), 1);
        assert_eq!(OpCode::JumpIf.num_operands(), 1);
        assert_eq!(OpCode::JumpIfNot.num_operands(), 1);
        assert_eq!(OpCode::Call.num_operands(), 1);
        assert_eq!(OpCode::StoreLocal.num_operands(), 1);
        assert_eq!(OpCode::LoadLocal.num_operands(), 1);
        assert_eq!(OpCode::CollectHeap.num_operands(), 1);
        assert_eq!(OpCode::NewArray.num_operands(), 1);

        // Test opcodes with 3 operands
        assert_eq!(OpCode::TryBlock.num_operands(), 3);
    }

    #[test]
    fn test_from_u8_trait() {
        // Test valid conversions using From trait
        let opcode: OpCode = 0x00u8.into();
        assert_eq!(opcode, OpCode::Nop);

        let opcode: OpCode = 0x10u8.into();
        assert_eq!(opcode, OpCode::Push);

        let opcode: OpCode = 0x6Au8.into();
        assert_eq!(opcode, OpCode::TryBlock);

        // Test invalid conversion defaults to Nop
        let opcode: OpCode = 0xFFu8.into();
        assert_eq!(opcode, OpCode::Nop);
    }

    #[test]
    fn test_into_u8_trait() {
        // Test conversion from OpCode to u8 using Into trait
        let byte: u8 = OpCode::Nop.into();
        assert_eq!(byte, 0x00);

        let byte: u8 = OpCode::Push.into();
        assert_eq!(byte, 0x10);

        let byte: u8 = OpCode::TryBlock.into();
        assert_eq!(byte, 0x6A);

        let byte: u8 = OpCode::NewArray.into();
        assert_eq!(byte, 0x90);
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test that converting to byte and back gives the same opcode
        let opcodes = [
            OpCode::Nop,
            OpCode::Halt,
            OpCode::Push,
            OpCode::Add,
            OpCode::Jump,
            OpCode::TryBlock,
            OpCode::CatchBlock,
            OpCode::NewArray,
        ];

        for opcode in &opcodes {
            let byte = opcode.to_byte();
            let converted_back = OpCode::from_byte(byte);
            assert_eq!(converted_back, Some(*opcode));
        }
    }

    #[test]
    fn test_opcode_equality() {
        // Test that OpCode implements PartialEq correctly
        assert_eq!(OpCode::Nop, OpCode::Nop);
        assert_ne!(OpCode::Nop, OpCode::Halt);
        assert_eq!(OpCode::TryBlock, OpCode::TryBlock);
        assert_ne!(OpCode::TryBlock, OpCode::CatchBlock);
    }

    #[test]
    fn test_opcode_debug() {
        // Test that OpCode implements Debug correctly
        let debug_string = format!("{:?}", OpCode::TryBlock);
        assert_eq!(debug_string, "TryBlock");

        let debug_string = format!("{:?}", OpCode::Push);
        assert_eq!(debug_string, "Push");
    }

    #[test]
    fn test_opcode_clone_copy() {
        // Test that OpCode implements Clone and Copy correctly
        let original = OpCode::TryBlock;
        let cloned = original.clone();
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);
    }

    #[test]
    fn test_all_exception_handling_opcodes() {
        // Verify all exception handling opcodes are properly defined
        let exception_opcodes = [
            (OpCode::TryBlock, 0x6A, 3),
            (OpCode::CatchBlock, 0x6B, 0),
            (OpCode::FinallyBlock, 0x6C, 0),
            (OpCode::EndTry, 0x6D, 0),
            (OpCode::Throw, 0x6E, 0),
            (OpCode::Rethrow, 0x6F, 0),
        ];

        for (opcode, expected_byte, expected_operands) in &exception_opcodes {
            assert_eq!(opcode.to_byte(), *expected_byte);
            assert_eq!(opcode.num_operands(), *expected_operands);
            assert_eq!(OpCode::from_byte(*expected_byte), Some(*opcode));
        }
    }
}