use std::io::{Error as IoError, Read};
use byteorder::{ReadBytesExt, BigEndian};
use thiserror::Error;
use crate::bytecode::{BytecodeModule, Constant, Instruction, OpCode};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    IoError(#[from] IoError),
    
    #[error("Invalid bytecode format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported bytecode version: {0}")]
    UnsupportedVersion(u8),
}

pub struct Parser;

impl Parser {
    /// Parse bytecode from a reader (file, memory buffer, etc.)
    pub fn parse<R: Read>(reader: &mut R) -> Result<BytecodeModule, ParseError> {
        // Read magic number and version
        let magic = reader.read_u32::<BigEndian>()?;
        if magic != 0x4C4F4146 {  // "LOAF" in ASCII
            return Err(ParseError::InvalidFormat("Invalid magic number".to_string()));
        }
        
        let version = reader.read_u8()?;
        if version != 1 {
            return Err(ParseError::UnsupportedVersion(version));
        }
        let _minor_version = reader.read_u8()?;
        let _patch_version = reader.read_u16::<BigEndian>()?;
        
        // Read module name length and name
        let name_len = reader.read_u32::<BigEndian>()? as usize;
        let mut name_bytes = vec![0u8; name_len];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8_lossy(&name_bytes).to_string();
        
        let mut module = BytecodeModule::new(&name);
        
        // Read constants
        let constants_len = reader.read_u32::<BigEndian>()? as usize;
        for _ in 0..constants_len {
            let const_type = reader.read_u8()?;
            let constant = match const_type {
                0 => Constant::Null,
                1 => {
                    let value = reader.read_i64::<BigEndian>()?;
                    Constant::Integer(value)
                },
                2 => {
                    let value = reader.read_f64::<BigEndian>()?;
                    Constant::Float(value)
                },
                3 => {
                    let str_len = reader.read_u32::<BigEndian>()? as usize;
                    let mut str_bytes = vec![0u8; str_len];
                    reader.read_exact(&mut str_bytes)?;
                    let string = String::from_utf8_lossy(&str_bytes).to_string();
                    Constant::String(string)
                },
                4 => {
                    let value = reader.read_u8()? != 0;
                    Constant::Boolean(value)
                },
                _ => return Err(ParseError::InvalidFormat(format!("Unknown constant type: {}", const_type))),
            };
            module.constants.push(constant);
        }
        
        // Read instructions
        let instructions_len = reader.read_u32::<BigEndian>()? as usize;
        let mut address_map = module.address_map;
        let mut address = 0;
        let mut code_page = module.code_page;

        for idx in 0..instructions_len {
            address_map.insert(address, idx);
            let opcode_byte = reader.read_u8()?;
            code_page.push(opcode_byte);
            let opcode = OpCode::from(opcode_byte);
            let operand_count = opcode.num_operands() as u32;
            address += 1 + operand_count * 4; // 1 byte for opcode + 4 bytes for each operand
            
            let mut instruction = Instruction::new(opcode);
            for _ in 0..operand_count {
                let operand = reader.read_u32::<BigEndian>()?;
                code_page.push((operand >> 24) as u8);
                code_page.push((operand >> 16) as u8);
                code_page.push((operand >> 8) as u8);
                code_page.push(operand as u8);
                instruction = instruction.with_operand(operand);
            }
            
            module.instructions.push(instruction);
        }

        module.code_page = code_page;
        module.address_map = address_map;
        
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor};
    use byteorder::{WriteBytesExt, BigEndian};
    use crate::bytecode::{Constant, OpCode};

    /// Helper function to create valid bytecode header
    fn create_valid_header() -> Vec<u8> {
        let mut data = Vec::new();
        data.write_u32::<BigEndian>(0x4C4F4146).unwrap(); // Magic "LOAF"
        data.write_u8(1).unwrap(); // Major version
        data.write_u8(0).unwrap(); // Minor version  
        data.write_u16::<BigEndian>(0).unwrap(); // Patch version
        data
    }

    /// Helper function to create a complete valid bytecode
    fn create_valid_bytecode() -> Vec<u8> {
        let mut data = create_valid_header();
        
        // Module name "test"
        data.write_u32::<BigEndian>(4).unwrap();
        data.extend_from_slice(b"test");
        
        // Constants: [42, 3.14, "hello", true, null]
        data.write_u32::<BigEndian>(5).unwrap();
        
        // Integer constant
        data.write_u8(1).unwrap();
        data.write_i64::<BigEndian>(42).unwrap();
        
        // Float constant
        data.write_u8(2).unwrap();
        data.write_f64::<BigEndian>(3.14).unwrap();
        
        // String constant
        data.write_u8(3).unwrap();
        data.write_u32::<BigEndian>(5).unwrap();
        data.extend_from_slice(b"hello");
        
        // Boolean constant
        data.write_u8(4).unwrap();
        data.write_u8(1).unwrap();
        
        // Null constant
        data.write_u8(0).unwrap();
        
        // Instructions: [Push(0), Print, Halt]
        data.write_u32::<BigEndian>(3).unwrap();
        
        // Push instruction with operand 0
        data.write_u8(OpCode::Push.into()).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        
        // Print instruction (no operands)
        data.write_u8(OpCode::Print.into()).unwrap();
        
        // Halt instruction (no operands)
        data.write_u8(OpCode::Halt.into()).unwrap();
        
        data
    }

    #[test]
    fn test_parse_valid_bytecode() {
        let data = create_valid_bytecode();
        let mut cursor = Cursor::new(data);
        
        let result = Parser::parse(&mut cursor);
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.name, "test");
        assert_eq!(module.constants.len(), 5);
        assert_eq!(module.instructions.len(), 3);
        
        // Verify constants
        match &module.constants[0] {
            Constant::Integer(42) => {},
            _ => panic!("Expected Integer(42)"),
        }
        
        match &module.constants[1] {
            Constant::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected Float(3.14)"),
        }
        
        match &module.constants[2] {
            Constant::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String(hello)"),
        }
        
        match &module.constants[3] {
            Constant::Boolean(true) => {},
            _ => panic!("Expected Boolean(true)"),
        }
        
        match &module.constants[4] {
            Constant::Null => {},
            _ => panic!("Expected Null"),
        }
        
        // Verify instructions
        assert_eq!(module.instructions[0].opcode, OpCode::Push);
        assert_eq!(module.instructions[0].operands, vec![0]);
        assert_eq!(module.instructions[1].opcode, OpCode::Print);
        assert_eq!(module.instructions[1].operands.len(), 0);
        assert_eq!(module.instructions[2].opcode, OpCode::Halt);
        assert_eq!(module.instructions[2].operands.len(), 0);
    }

    #[test]
    fn test_parse_invalid_magic_number() {
        let mut data = Vec::new();
        data.write_u32::<BigEndian>(0x12345678).unwrap(); // Invalid magic
        data.write_u8(1).unwrap(); // Version
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidFormat(msg) => assert_eq!(msg, "Invalid magic number"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_unsupported_version() {
        let mut data = Vec::new();
        data.write_u32::<BigEndian>(0x4C4F4146).unwrap(); // Valid magic "LOAF"
        data.write_u8(2).unwrap(); // Unsupported version
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnsupportedVersion(2) => {},
            _ => panic!("Expected UnsupportedVersion(2) error"),
        }
    }

    #[test]
    fn test_parse_empty_module() {
        let mut data = create_valid_header();
        
        // Module name ""
        data.write_u32::<BigEndian>(0).unwrap();
        
        // No constants
        data.write_u32::<BigEndian>(0).unwrap();
        
        // No instructions
        data.write_u32::<BigEndian>(0).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "");
        assert_eq!(module.constants.len(), 0);
        assert_eq!(module.instructions.len(), 0);
    }

    #[test]
    fn test_parse_all_constant_types() {
        let mut data = create_valid_header();
        
        // Module name "constants_test"
        data.write_u32::<BigEndian>(14).unwrap();
        data.extend_from_slice(b"constants_test");
        
        // All 5 constant types
        data.write_u32::<BigEndian>(5).unwrap();
        
        // Null
        data.write_u8(0).unwrap();
        
        // Integer: -999
        data.write_u8(1).unwrap();
        data.write_i64::<BigEndian>(-999).unwrap();
        
        // Float: -2.718
        data.write_u8(2).unwrap();
        data.write_f64::<BigEndian>(-2.718).unwrap();
        
        // String: "test string"
        data.write_u8(3).unwrap();
        data.write_u32::<BigEndian>(11).unwrap();
        data.extend_from_slice(b"test string");
        
        // Boolean: false
        data.write_u8(4).unwrap();
        data.write_u8(0).unwrap();
        
        // No instructions
        data.write_u32::<BigEndian>(0).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.constants.len(), 5);
        
        match &module.constants[0] {
            Constant::Null => {},
            _ => panic!("Expected Null"),
        }
        
        match &module.constants[1] {
            Constant::Integer(-999) => {},
            _ => panic!("Expected Integer(-999)"),
        }
        
        match &module.constants[2] {
            Constant::Float(f) => assert!((f + 2.718).abs() < f64::EPSILON),
            _ => panic!("Expected Float(-2.718)"),
        }
        
        match &module.constants[3] {
            Constant::String(s) => assert_eq!(s, "test string"),
            _ => panic!("Expected String(test string)"),
        }
        
        match &module.constants[4] {
            Constant::Boolean(false) => {},
            _ => panic!("Expected Boolean(false)"),
        }
    }

    #[test]
    fn test_parse_unknown_constant_type() {
        let mut data = create_valid_header();
        
        // Module name "test"
        data.write_u32::<BigEndian>(4).unwrap();
        data.extend_from_slice(b"test");
        
        // One invalid constant
        data.write_u32::<BigEndian>(1).unwrap();
        data.write_u8(99).unwrap(); // Unknown constant type
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidFormat(msg) => assert_eq!(msg, "Unknown constant type: 99"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_instructions_with_operands() {
        let mut data = create_valid_header();
        
        // Module name "ops_test"
        data.write_u32::<BigEndian>(8).unwrap();
        data.extend_from_slice(b"ops_test");
        
        // No constants
        data.write_u32::<BigEndian>(0).unwrap();
        
        // Test different opcodes with different operand counts
        data.write_u32::<BigEndian>(4).unwrap();
        
        // Push (1 operand)
        data.write_u8(OpCode::Push.into()).unwrap();
        data.write_u32::<BigEndian>(123).unwrap();
        
        // Add (0 operands)
        data.write_u8(OpCode::Add.into()).unwrap();
        
        // Jump (1 operand)
        data.write_u8(OpCode::Jump.into()).unwrap();
        data.write_u32::<BigEndian>(456).unwrap();
        
        // TryBlock (3 operands)
        data.write_u8(OpCode::TryBlock.into()).unwrap();
        data.write_u32::<BigEndian>(10).unwrap();
        data.write_u32::<BigEndian>(20).unwrap();
        data.write_u32::<BigEndian>(30).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.instructions.len(), 4);
        
        // Verify Push instruction
        assert_eq!(module.instructions[0].opcode, OpCode::Push);
        assert_eq!(module.instructions[0].operands, vec![123]);
        
        // Verify Add instruction
        assert_eq!(module.instructions[1].opcode, OpCode::Add);
        assert_eq!(module.instructions[1].operands.len(), 0);
        
        // Verify Jump instruction
        assert_eq!(module.instructions[2].opcode, OpCode::Jump);
        assert_eq!(module.instructions[2].operands, vec![456]);
        
        // Verify TryBlock instruction
        assert_eq!(module.instructions[3].opcode, OpCode::TryBlock);
        assert_eq!(module.instructions[3].operands, vec![10, 20, 30]);
    }

    #[test]
    fn test_parse_truncated_data() {
        // Test various truncation points
        let valid_data = create_valid_bytecode();
        
        // Test truncation at different points
        for truncate_at in [3, 7, 10, 15] {
            let mut truncated = valid_data.clone();
            truncated.truncate(truncate_at);
            
            let mut cursor = Cursor::new(truncated);
            let result = Parser::parse(&mut cursor);
            
            assert!(result.is_err());
            match result.unwrap_err() {
                ParseError::IoError(_) => {}, // Expected
                _ => panic!("Expected IoError for truncated data"),
            }
        }
    }

    #[test]
    fn test_parse_large_string_constant() {
        let mut data = create_valid_header();
        
        // Module name "large_test"
        data.write_u32::<BigEndian>(10).unwrap();
        data.extend_from_slice(b"large_test");
        
        // One large string constant
        data.write_u32::<BigEndian>(1).unwrap();
        data.write_u8(3).unwrap(); // String type
        
        let large_string = "a".repeat(1000);
        data.write_u32::<BigEndian>(large_string.len() as u32).unwrap();
        data.extend_from_slice(large_string.as_bytes());
        
        // No instructions
        data.write_u32::<BigEndian>(0).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.constants.len(), 1);
        
        match &module.constants[0] {
            Constant::String(s) => {
                assert_eq!(s.len(), 1000);
                assert!(s.chars().all(|c| c == 'a'));
            },
            _ => panic!("Expected large String"),
        }
    }

    #[test]
    fn test_parse_unicode_strings() {
        let mut data = create_valid_header();
        
        // Module name with unicode
        let module_name = "тест_模块_🚀";
        data.write_u32::<BigEndian>(module_name.len() as u32).unwrap();
        data.extend_from_slice(module_name.as_bytes());
        
        // Unicode string constant
        data.write_u32::<BigEndian>(1).unwrap();
        data.write_u8(3).unwrap(); // String type
        
        let unicode_string = "Hello 世界 🌍 Привет мир";
        data.write_u32::<BigEndian>(unicode_string.len() as u32).unwrap();
        data.extend_from_slice(unicode_string.as_bytes());
        
        // No instructions
        data.write_u32::<BigEndian>(0).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, module_name);
        assert_eq!(module.constants.len(), 1);
        
        match &module.constants[0] {
            Constant::String(s) => assert_eq!(s, unicode_string),
            _ => panic!("Expected Unicode String"),
        }
    }

    #[test]
    fn test_parse_extreme_values() {
        let mut data = create_valid_header();
        
        // Module name "extreme"
        data.write_u32::<BigEndian>(7).unwrap();
        data.extend_from_slice(b"extreme");
        
        // Test extreme integer and float values
        data.write_u32::<BigEndian>(4).unwrap();
        
        // Max i64
        data.write_u8(1).unwrap();
        data.write_i64::<BigEndian>(i64::MAX).unwrap();
        
        // Min i64
        data.write_u8(1).unwrap();
        data.write_i64::<BigEndian>(i64::MIN).unwrap();
        
        // Positive infinity
        data.write_u8(2).unwrap();
        data.write_f64::<BigEndian>(f64::INFINITY).unwrap();
        
        // Negative infinity
        data.write_u8(2).unwrap();
        data.write_f64::<BigEndian>(f64::NEG_INFINITY).unwrap();
        
        // No instructions
        data.write_u32::<BigEndian>(0).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.constants.len(), 4);
        
        match &module.constants[0] {
            Constant::Integer(i) => assert_eq!(*i, i64::MAX),
            _ => panic!("Expected i64::MAX"),
        }
        
        match &module.constants[1] {
            Constant::Integer(i) => assert_eq!(*i, i64::MIN),
            _ => panic!("Expected i64::MIN"),
        }
        
        match &module.constants[2] {
            Constant::Float(f) => assert_eq!(*f, f64::INFINITY),
            _ => panic!("Expected f64::INFINITY"),
        }
        
        match &module.constants[3] {
            Constant::Float(f) => assert_eq!(*f, f64::NEG_INFINITY),
            _ => panic!("Expected f64::NEG_INFINITY"),
        }
    }

    #[test]
    fn test_parse_address_map_generation() {
        let mut data = create_valid_header();
        
        // Module name "address_test"
        data.write_u32::<BigEndian>(12).unwrap();
        data.extend_from_slice(b"address_test");
        
        // No constants
        data.write_u32::<BigEndian>(0).unwrap();
        
        // Instructions with different operand counts to test address calculation
        data.write_u32::<BigEndian>(4).unwrap();
        
        // Halt (0 operands) - should be at address 0
        data.write_u8(OpCode::Halt.into()).unwrap();
        
        // Push (1 operand) - should be at address 1
        data.write_u8(OpCode::Push.into()).unwrap();
        data.write_u32::<BigEndian>(42).unwrap();
        
        // Add (0 operands) - should be at address 6 (1 + 1 + 4)
        data.write_u8(OpCode::Add.into()).unwrap();
        
        // TryBlock (3 operands) - should be at address 7
        data.write_u8(OpCode::TryBlock.into()).unwrap();
        data.write_u32::<BigEndian>(10).unwrap();
        data.write_u32::<BigEndian>(20).unwrap();
        data.write_u32::<BigEndian>(30).unwrap();
        
        let mut cursor = Cursor::new(data);
        let result = Parser::parse(&mut cursor);
        
        assert!(result.is_ok());
        let module = result.unwrap();
        
        // Verify address mapping
        assert_eq!(module.address_map.get(&0), Some(&0));  // Halt at address 0, instruction 0
        assert_eq!(module.address_map.get(&1), Some(&1));  // Push at address 1, instruction 1  
        assert_eq!(module.address_map.get(&6), Some(&2));  // Add at address 6, instruction 2
        assert_eq!(module.address_map.get(&7), Some(&3));  // TryBlock at address 7, instruction 3
    }
}