use std::vec;

use crate::ast::Statement;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpInstruction {
    RETURN = 0,
    ADD,
    ADDI,
}

// #[repr(packed(1))]
// all instructions are 32 bit
#[derive(Debug)]
pub struct Instruction {
    pub op_instruction: OpInstruction,
    pub arg_0: u8,
    pub arg_1: u8,
    pub arg_2: u8,
}

#[derive(Debug)]
pub struct Value {}

#[derive(Debug)]
pub struct Chunk {
    pub instructions: std::vec::Vec<Instruction>,
    pub constant_pool: std::vec::Vec<Value>,
}

#[derive(Debug)]
pub struct Bytecode {
    pub instructions: std::vec::Vec<Instruction>,
}

pub struct BytecodeGenerator {}

impl BytecodeGenerator {
    pub fn generate(&self, ast: &Statement) -> Bytecode {
        return Bytecode {
            instructions: vec![
                // right now assume the stack is zero'd out
                Instruction {
                    // put 10 in register 10
                    op_instruction: OpInstruction::ADDI,
                    arg_0: 0,
                    arg_1: 10,
                    arg_2: 0,
                },
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: 0,
                    arg_1: 20,
                    arg_2: 0,
                },
                Instruction {
                    op_instruction: OpInstruction::ADD,
                    arg_0: 0,
                    arg_1: 0,
                    arg_2: 1,
                },
                Instruction {
                    op_instruction: OpInstruction::RETURN,
                    arg_0: 0,
                    arg_1: 0,
                    arg_2: 0,
                },
            ],
        };
    }
}
