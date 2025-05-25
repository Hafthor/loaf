//! Utility functions for the Loaf bytecode runtime

use std::fs::File;
use std::io::{BufWriter, Write, Error as IoError};
use std::path::Path;
use byteorder::{BigEndian, WriteBytesExt};
use crate::bytecode::{BytecodeModule, Constant};

/// Writes a bytecode module to a file
pub fn write_bytecode<P: AsRef<Path>>(module: &BytecodeModule, path: P) -> Result<(), IoError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    
    // Write magic number "LOAF"
    writer.write_u32::<BigEndian>(0x4C4F4146)?;
    
    // Write version
    writer.write_u8(1)?; // Major version
    writer.write_u8(0)?; // Minor version
    writer.write_u16::<BigEndian>(0)?; // Patch version
    
    // Write module name
    writer.write_u32::<BigEndian>(module.name.len() as u32)?;
    writer.write_all(module.name.as_bytes())?;
    
    // Write constants
    writer.write_u32::<BigEndian>(module.constants.len() as u32)?;
    for constant in &module.constants {
        match constant {
            Constant::Null => {
                writer.write_u8(0)?;
            },
            Constant::Integer(i) => {
                writer.write_u8(1)?;
                writer.write_i64::<BigEndian>(*i)?;
            },
            Constant::Float(f) => {
                writer.write_u8(2)?;
                writer.write_f64::<BigEndian>(*f)?;
            },
            Constant::String(s) => {
                writer.write_u8(3)?;
                writer.write_u32::<BigEndian>(s.len() as u32)?;
                writer.write_all(s.as_bytes())?;
            },
            Constant::Boolean(b) => {
                writer.write_u8(4)?;
                writer.write_u8(*b as u8)?;
            },
        }
    }
    
    // Write instructions
    writer.write_u32::<BigEndian>(module.instructions.len() as u32)?;
    for instruction in &module.instructions {
        // Convert the opcode to its byte representation using From<OpCode> for u8
        let opcode_byte: u8 = instruction.opcode.into();
        writer.write_u8(opcode_byte)?;
        if instruction.opcode.num_operands() != instruction.operands.len() {
            return Err(IoError::new(std::io::ErrorKind::InvalidData, "Operand count mismatch"));
        }
        for operand in &instruction.operands {
            writer.write_u32::<BigEndian>(*operand)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

/// Generate a simple demonstration bytecode module
pub fn generate_demo_module() -> BytecodeModule {
    use crate::bytecode::{OpCode, Instruction};
    
    let mut module = BytecodeModule::new("demo");
    
    // Add constants
    module.constants.push(Constant::Integer(42));          // 0
    module.constants.push(Constant::Integer(58));          // 1
    module.constants.push(Constant::String("Hello, ".to_string()));  // 2
    module.constants.push(Constant::String("Loaf!".to_string()));    // 3
    module.constants.push(Constant::String("Division by zero!".to_string()));  // 4
    module.constants.push(Constant::String("DivisionError".to_string()));      // 5
    module.constants.push(Constant::String("Exception caught: ".to_string())); // 6
    module.constants.push(Constant::Integer(10));          // 7
    module.constants.push(Constant::Integer(0));           // 8
    module.constants.push(Constant::String("Made it to finally block!".to_string())); // 9
    module.constants.push(Constant::String("After try-catch-finally block".to_string())); // 10
    
    // Simple arithmetic operations in the default heap
    /* 0000 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(0)); // Push 42 - Instruction 0
    /* 0005 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(1)); // Push 58 - Instruction 1
    /* 000A */ module.instructions.push(Instruction::new(OpCode::Add)); // 42 + 58 = 100 - Instruction 2
    /* 000B */ module.instructions.push(Instruction::new(OpCode::Print)); // Print 100 - Instruction 3
    
    // Create a new heap and get its ID
    /* 000C */ module.instructions.push(Instruction::new(OpCode::CreateHeap)); // Creates heap and pushes ID - Instruction 4
    /* 000D */ module.instructions.push(Instruction::new(OpCode::Print)); // Print the heap ID - Instruction 5
    
    // String operations in the default heap
    /* 000E */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(2)); // Push "Hello, " - Instruction 6
    /* 0013 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(3)); // Push "Loaf!" - Instruction 7
    /* 0018 */ module.instructions.push(Instruction::new(OpCode::Add)); // Concatenate to "Hello, Loaf!" - Instruction 8
    /* 0019 */ module.instructions.push(Instruction::new(OpCode::Print)); // Print the concatenated string - Instruction 9
    
    // Exception handling demo
    // TryBlock operands: catch_pc, finally_pc, end_try_pc
    let catch_pc = 0x43; // Instruction position of CatchBlock
    let finally_pc = 0x51; // Instruction position of FinallyBlock
    let end_try_pc = 0x59; // Instruction position after EndTry
    
    // Start try block - Instruction 10
    /* 001A */ module.instructions.push(Instruction::new(OpCode::TryBlock)
        .with_operands(vec![catch_pc, finally_pc, end_try_pc]));
    {
        // Code that will throw an exception (divide by zero)
        /* 0027 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(7)); // Push 10 - Instruction 11
        /* 002C */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(8)); // Push 0 - Instruction 12
        
        // Try to divide by zero (will throw)
        /* 0031 */ module.instructions.push(Instruction::new(OpCode::Div)); // 10 / 0 - Instruction 13
        
        // This should be skipped due to the exception
        /* 0032 */ module.instructions.push(Instruction::new(OpCode::Print)); // Print result - Instruction 14
        
        // Create and throw a custom exception - Instruction 15
        /* 0033 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(5)); // "DivisionError"
        /* 0038 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(4)); // "Division by zero!"
        /* 003D */ module.instructions.push(Instruction::new(OpCode::NewArray).with_operand(2)); // Create [type, message]
        /* 0042 */ module.instructions.push(Instruction::new(OpCode::Throw)); // Throw the exception
        
        // Start catch block - Instruction 18
        /* 0043 */ module.instructions.push(Instruction::new(OpCode::CatchBlock));
        {
            // The exception is now on top of the stack
            /* 0044 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(6)); // "Exception caught: " - Instruction 19
            /* 0049 */ module.instructions.push(Instruction::new(OpCode::Swap)); // Swap to get exception on top
            /* 004A */ module.instructions.push(Instruction::new(OpCode::Add)); // Concatenate - Instruction 20
            /* 004B */ module.instructions.push(Instruction::new(OpCode::Print)); // Print message with exception - Instruction 21
            /* 004C */ module.instructions.push(Instruction::new(OpCode::Jump).with_operand(finally_pc)); // Jump to finally - Instruction 22
        }
        // Start finally block - Instruction 24
        /* 0051 */ module.instructions.push(Instruction::new(OpCode::FinallyBlock));
        {
            // Print that we reached the finally block
            /* 0052 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(9)); // "Made it to finally block!" - Instruction 25
            /* 0057 */ module.instructions.push(Instruction::new(OpCode::Print)); // Print message - Instruction 26
        }
    }
    // End of try-catch-finally - Instruction 27
    /* 0058 */ module.instructions.push(Instruction::new(OpCode::EndTry));
    
    // Print a message showing we're past the try-catch block - Instruction 28
    /* 0059 */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(10)); // "After try-catch-finally block"
    /* 005E */ module.instructions.push(Instruction::new(OpCode::Print)); // Print message
    
    // Return value and halt
    /* 005F */ module.instructions.push(Instruction::new(OpCode::Push).with_operand(0)); // Push 42 for return value - Instruction 30
    /* 0064 */ module.instructions.push(Instruction::new(OpCode::Halt)); // End execution - Instruction 31
    
    module
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Read};
    use byteorder::{BigEndian, ReadBytesExt};
    use tempfile::tempdir;
    use crate::bytecode::{OpCode, Instruction, Parser};

    #[test]
    fn test_write_bytecode_empty_module() {
        let module = BytecodeModule::new("empty");
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Verify file exists
        assert!(file_path.exists());
        
        // Read back and verify basic structure
        let mut file = File::open(&file_path).unwrap();
        let mut reader = BufReader::new(&mut file);
        
        // Check magic number
        assert_eq!(reader.read_u32::<BigEndian>().unwrap(), 0x4C4F4146);
        
        // Check version
        assert_eq!(reader.read_u8().unwrap(), 1); // Major
        assert_eq!(reader.read_u8().unwrap(), 0); // Minor
        assert_eq!(reader.read_u16::<BigEndian>().unwrap(), 0); // Patch
        
        // Check module name
        let name_len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(name_len, 5);
        let mut name_bytes = vec![0u8; name_len as usize];
        reader.read_exact(&mut name_bytes).unwrap();
        assert_eq!(String::from_utf8(name_bytes).unwrap(), "empty");
        
        // Check constants count
        assert_eq!(reader.read_u32::<BigEndian>().unwrap(), 0);
        
        // Check instructions count
        assert_eq!(reader.read_u32::<BigEndian>().unwrap(), 0);
    }

    #[test]
    fn test_write_bytecode_with_all_constant_types() {
        let mut module = BytecodeModule::new("test_constants");
        
        // Add all types of constants
        module.constants.push(Constant::Null);
        module.constants.push(Constant::Integer(42));
        module.constants.push(Constant::Float(3.14));
        module.constants.push(Constant::String("Hello".to_string()));
        module.constants.push(Constant::Boolean(true));
        module.constants.push(Constant::Boolean(false));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("constants.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.name, "test_constants");
        assert_eq!(parsed_module.constants.len(), 6);
        
        match &parsed_module.constants[0] {
            Constant::Null => {},
            _ => panic!("Expected Null constant"),
        }
        
        match &parsed_module.constants[1] {
            Constant::Integer(i) => assert_eq!(*i, 42),
            _ => panic!("Expected Integer constant"),
        }
        
        match &parsed_module.constants[2] {
            Constant::Float(f) => assert_eq!(*f, 3.14),
            _ => panic!("Expected Float constant"),
        }
        
        match &parsed_module.constants[3] {
            Constant::String(s) => assert_eq!(s, "Hello"),
            _ => panic!("Expected String constant"),
        }
        
        match &parsed_module.constants[4] {
            Constant::Boolean(b) => assert_eq!(*b, true),
            _ => panic!("Expected Boolean constant"),
        }
        
        match &parsed_module.constants[5] {
            Constant::Boolean(b) => assert_eq!(*b, false),
            _ => panic!("Expected Boolean constant"),
        }
    }

    #[test]
    fn test_write_bytecode_with_instructions() {
        let mut module = BytecodeModule::new("test_instructions");
        
        // Add some constants for instructions to reference
        module.constants.push(Constant::Integer(10));
        module.constants.push(Constant::Integer(20));
        
        // Add various instructions
        module.instructions.push(Instruction::new(OpCode::Nop));
        module.instructions.push(Instruction::new(OpCode::Push).with_operand(0));
        module.instructions.push(Instruction::new(OpCode::Push).with_operand(1));
        module.instructions.push(Instruction::new(OpCode::Add));
        module.instructions.push(Instruction::new(OpCode::Print));
        module.instructions.push(Instruction::new(OpCode::Halt));
        module.instructions.push(Instruction::new(OpCode::TryBlock).with_operands(vec![10, 20, 30]));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("instructions.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.name, "test_instructions");
        assert_eq!(parsed_module.instructions.len(), 7);
        
        assert_eq!(parsed_module.instructions[0].opcode, OpCode::Nop);
        assert_eq!(parsed_module.instructions[0].operands.len(), 0);
        
        assert_eq!(parsed_module.instructions[1].opcode, OpCode::Push);
        assert_eq!(parsed_module.instructions[1].operands, vec![0]);
        
        assert_eq!(parsed_module.instructions[2].opcode, OpCode::Push);
        assert_eq!(parsed_module.instructions[2].operands, vec![1]);
        
        assert_eq!(parsed_module.instructions[3].opcode, OpCode::Add);
        assert_eq!(parsed_module.instructions[3].operands.len(), 0);
        
        assert_eq!(parsed_module.instructions[4].opcode, OpCode::Print);
        assert_eq!(parsed_module.instructions[4].operands.len(), 0);
        
        assert_eq!(parsed_module.instructions[5].opcode, OpCode::Halt);
        assert_eq!(parsed_module.instructions[5].operands.len(), 0);
        
        assert_eq!(parsed_module.instructions[6].opcode, OpCode::TryBlock);
        assert_eq!(parsed_module.instructions[6].operands, vec![10, 20, 30]);
    }

    #[test]
    fn test_write_bytecode_large_string_constant() {
        let mut module = BytecodeModule::new("large_string_test");
        
        // Create a large string constant
        let large_string = "x".repeat(10000);
        module.constants.push(Constant::String(large_string.clone()));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("large_string.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        match &parsed_module.constants[0] {
            Constant::String(s) => {
                assert_eq!(s.len(), 10000);
                assert_eq!(s, &large_string);
            },
            _ => panic!("Expected String constant"),
        }
    }

    #[test]
    fn test_write_bytecode_unicode_strings() {
        let mut module = BytecodeModule::new("unicode_test");
        
        // Add Unicode string constants
        module.constants.push(Constant::String("🚀".to_string()));
        module.constants.push(Constant::String("Hello, 世界!".to_string()));
        module.constants.push(Constant::String("Ñoño ümlaut café".to_string()));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("unicode.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.constants.len(), 3);
        
        match &parsed_module.constants[0] {
            Constant::String(s) => assert_eq!(s, "🚀"),
            _ => panic!("Expected String constant"),
        }
        
        match &parsed_module.constants[1] {
            Constant::String(s) => assert_eq!(s, "Hello, 世界!"),
            _ => panic!("Expected String constant"),
        }
        
        match &parsed_module.constants[2] {
            Constant::String(s) => assert_eq!(s, "Ñoño ümlaut café"),
            _ => panic!("Expected String constant"),
        }
    }

    #[test]
    fn test_write_bytecode_extreme_values() {
        let mut module = BytecodeModule::new("extreme_values");
        
        // Add extreme values
        module.constants.push(Constant::Integer(i64::MAX));
        module.constants.push(Constant::Integer(i64::MIN));
        module.constants.push(Constant::Float(f64::MAX));
        module.constants.push(Constant::Float(f64::MIN));
        module.constants.push(Constant::Float(f64::INFINITY));
        module.constants.push(Constant::Float(f64::NEG_INFINITY));
        module.constants.push(Constant::Float(f64::NAN));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("extreme.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        
        match &parsed_module.constants[0] {
            Constant::Integer(i) => assert_eq!(*i, i64::MAX),
            _ => panic!("Expected Integer constant"),
        }
        
        match &parsed_module.constants[1] {
            Constant::Integer(i) => assert_eq!(*i, i64::MIN),
            _ => panic!("Expected Integer constant"),
        }
        
        match &parsed_module.constants[2] {
            Constant::Float(f) => assert_eq!(*f, f64::MAX),
            _ => panic!("Expected Float constant"),
        }
        
        match &parsed_module.constants[3] {
            Constant::Float(f) => assert_eq!(*f, f64::MIN),
            _ => panic!("Expected Float constant"),
        }
        
        match &parsed_module.constants[4] {
            Constant::Float(f) => assert_eq!(*f, f64::INFINITY),
            _ => panic!("Expected Float constant"),
        }
        
        match &parsed_module.constants[5] {
            Constant::Float(f) => assert_eq!(*f, f64::NEG_INFINITY),
            _ => panic!("Expected Float constant"),
        }
        
        match &parsed_module.constants[6] {
            Constant::Float(f) => assert!(f.is_nan()),
            _ => panic!("Expected Float constant"),
        }
    }

    #[test]
    fn test_write_bytecode_operand_count_mismatch() {
        let mut module = BytecodeModule::new("mismatch_test");
        
        // Create an instruction with mismatched operand count
        let mut instruction = Instruction::new(OpCode::Push); // Push expects 1 operand
        instruction.operands = vec![1, 2]; // But we give it 2 operands
        module.instructions.push(instruction);
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("mismatch.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_err());
        
        match result.unwrap_err().kind() {
            std::io::ErrorKind::InvalidData => {},
            _ => panic!("Expected InvalidData error"),
        }
    }

    #[test]
    fn test_write_bytecode_file_creation_error() {
        let module = BytecodeModule::new("test");
        
        // Try to write to an invalid path (directory that doesn't exist)
        let invalid_path = "/nonexistent/directory/file.bytecode";
        let result = write_bytecode(&module, invalid_path);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_write_bytecode_empty_string_constant() {
        let mut module = BytecodeModule::new("empty_string_test");
        
        module.constants.push(Constant::String(String::new()));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_string.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        match &parsed_module.constants[0] {
            Constant::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected String constant"),
        }
    }

    #[test]
    fn test_write_bytecode_zero_operand_instructions() {
        let mut module = BytecodeModule::new("zero_operand_test");
        
        // Add instructions with zero operands
        module.instructions.push(Instruction::new(OpCode::Nop));
        module.instructions.push(Instruction::new(OpCode::Add));
        module.instructions.push(Instruction::new(OpCode::Sub));
        module.instructions.push(Instruction::new(OpCode::Pop));
        module.instructions.push(Instruction::new(OpCode::Halt));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("zero_operand.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.instructions.len(), 5);
        
        for instruction in &parsed_module.instructions {
            assert_eq!(instruction.operands.len(), 0);
        }
    }

    #[test]
    fn test_write_bytecode_multiple_operand_instructions() {
        let mut module = BytecodeModule::new("multi_operand_test");
        
        // Add instructions with multiple operands
        module.instructions.push(Instruction::new(OpCode::TryBlock).with_operands(vec![10, 20, 30]));
        module.instructions.push(Instruction::new(OpCode::NewArray).with_operand(5));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("multi_operand.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.instructions.len(), 2);
        
        assert_eq!(parsed_module.instructions[0].operands, vec![10, 20, 30]);
        assert_eq!(parsed_module.instructions[1].operands, vec![5]);
    }

    #[test]
    fn test_write_bytecode_long_module_name() {
        let long_name = "a".repeat(1000);
        let module = BytecodeModule::new(&long_name);
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("long_name.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.name, long_name);
    }

    #[test]
    fn test_write_bytecode_demo_module() {
        let demo_module = generate_demo_module();
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("demo.bytecode");
        
        let result = write_bytecode(&demo_module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify it matches the original
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        
        assert_eq!(parsed_module.name, demo_module.name);
        assert_eq!(parsed_module.constants.len(), demo_module.constants.len());
        assert_eq!(parsed_module.instructions.len(), demo_module.instructions.len());
        
        // Verify a few key constants
        match &parsed_module.constants[0] {
            Constant::Integer(i) => assert_eq!(*i, 42),
            _ => panic!("Expected Integer constant"),
        }
        
        match &parsed_module.constants[2] {
            Constant::String(s) => assert_eq!(s, "Hello, "),
            _ => panic!("Expected String constant"),
        }
    }

    #[test]
    fn test_write_bytecode_all_opcodes() {
        let mut module = BytecodeModule::new("all_opcodes_test");
        
        // Add constants that instructions might reference
        module.constants.push(Constant::Integer(0));
        
        // Test various opcodes with correct operand counts
        module.instructions.push(Instruction::new(OpCode::Nop));
        module.instructions.push(Instruction::new(OpCode::Halt));
        module.instructions.push(Instruction::new(OpCode::Print));
        module.instructions.push(Instruction::new(OpCode::Push).with_operand(0));
        module.instructions.push(Instruction::new(OpCode::Pop));
        module.instructions.push(Instruction::new(OpCode::Dup));
        module.instructions.push(Instruction::new(OpCode::Swap));
        module.instructions.push(Instruction::new(OpCode::Add));
        module.instructions.push(Instruction::new(OpCode::Sub));
        module.instructions.push(Instruction::new(OpCode::Mul));
        module.instructions.push(Instruction::new(OpCode::Div));
        module.instructions.push(Instruction::new(OpCode::Jump).with_operand(0));
        module.instructions.push(Instruction::new(OpCode::JumpIf).with_operand(0));
        module.instructions.push(Instruction::new(OpCode::Call).with_operand(0));
        module.instructions.push(Instruction::new(OpCode::Return));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("all_opcodes.bytecode");
        
        let result = write_bytecode(&module, &file_path);
        assert!(result.is_ok());
        
        // Parse it back and verify all opcodes are preserved
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.instructions.len(), module.instructions.len());
        
        for (original, parsed) in module.instructions.iter().zip(parsed_module.instructions.iter()) {
            assert_eq!(original.opcode, parsed.opcode);
            assert_eq!(original.operands, parsed.operands);
        }
    }

    #[test]
    fn test_write_bytecode_file_overwrite() {
        let module1 = BytecodeModule::new("first");
        let mut module2 = BytecodeModule::new("second");
        module2.constants.push(Constant::Integer(123));
        
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("overwrite.bytecode");
        
        // Write first module
        let result1 = write_bytecode(&module1, &file_path);
        assert!(result1.is_ok());
        
        // Write second module to same path (should overwrite)
        let result2 = write_bytecode(&module2, &file_path);
        assert!(result2.is_ok());
        
        // Parse and verify second module is what's in the file
        let parsed_module = Parser::parse(&mut BufReader::new(File::open(&file_path).unwrap())).unwrap();
        assert_eq!(parsed_module.name, "second");
        assert_eq!(parsed_module.constants.len(), 1);
        match &parsed_module.constants[0] {
            Constant::Integer(i) => assert_eq!(*i, 123),
            _ => panic!("Expected Integer constant"),
        }
    }
}