use std::vec;

use crate::{
    ast::Statement,
    execution::{FnObject, HeapObject, HeapObjectData, Object},
};

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpInstruction {
    RETURN = 0,
    ADD,
    ADDI,
    // NEW <location of fn> <args starting register> <number of args>
    CALL,
    // NEW <location of type> <args starting register> <number of args>
    NEW,
    LOAD_CONST,
}

// #[repr(packed(1))]
// all instructions are 32 bit
#[derive(Debug, Clone)]
pub struct Instruction {
    pub op_instruction: OpInstruction,
    pub arg_0: u8,
    pub arg_1: u8,
    pub arg_2: u8,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: std::vec::Vec<Instruction>,
    // todo only enable this in debug mode
    pub debug_line_info: std::vec::Vec<usize>,
    pub constant_pool: std::vec::Vec<Object>,
}

impl Chunk {
    pub fn print(&self) {
        println!("Chunk:");
        let mut counter: usize = 0;
        for instruction in &self.instructions {
            println!(
                "{:?} = {:?} {:?} {:?} {:?} ",
                self.debug_line_info[counter],
                instruction.op_instruction,
                instruction.arg_0,
                instruction.arg_1,
                instruction.arg_2
            );
            counter += 1;
        }
    }
}

#[derive(Debug)]
pub struct Bytecode {
    pub instructions: std::vec::Vec<Instruction>,
}

pub struct BytecodeGenerator {
    current_chunk: Chunk,
}

impl BytecodeGenerator {
    pub fn new() -> BytecodeGenerator {
        return BytecodeGenerator {
            current_chunk: Chunk {
                debug_line_info: vec![],
                constant_pool: vec![],
                instructions: vec![],
            },
        };
    }
    pub fn generate(&mut self, ast: &Statement) -> Chunk {
        let mut print_chunk = Chunk {
            instructions: vec![Instruction {
                op_instruction: OpInstruction::RETURN,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            }],
            debug_line_info: vec![],
            constant_pool: vec![],
        };
        self.push_constant(Object::HEAP_OBJECT(Box::new(HeapObject {
            data: HeapObjectData::FN(FnObject { chunk: print_chunk }),
            is_marked: false,
        })));

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            0,
        );
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::CALL,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            0,
        );

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::RETURN,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            3,
        );

        return self.current_chunk.clone();
    }

    fn push_instruction(&mut self, instruction: Instruction, line: usize) {
        self.current_chunk.instructions.push(instruction);
        self.current_chunk.debug_line_info.push(line);
    }

    fn push_constant(&mut self, constant: Object) {
        self.current_chunk.constant_pool.push(constant);
    }
}
