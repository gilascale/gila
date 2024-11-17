use std::vec;

use crate::{
    ast::{ASTNode, Op, Statement},
    execution::{FnObject, HeapObject, HeapObjectData, Object},
    lex::{Token, Type},
};

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpInstruction {
    RETURN = 0,
    ADD,
    // ADDI i1 i2 <desination>
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

    pub fn generate(&mut self, ast: &ASTNode) -> Chunk {
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

        self.visit(ast);

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

    fn visit(&mut self, ast: &ASTNode) {
        match &ast.statement {
            Statement::PROGRAM(p) => self.gen_program(&p),
            Statement::BLOCK(b) => self.gen_block(&b),
            Statement::BIN_OP(e1, e2, op) => self.gen_bin_op(&e1, &e2, &op),
            _ => panic!(),
        }
    }

    fn gen_program(&mut self, p: &Vec<ASTNode>) {
        for instruction in p {
            self.visit(instruction);
        }
    }

    fn gen_block(&mut self, b: &Vec<ASTNode>) {
        for instruction in b {
            self.visit(instruction);
        }
    }

    fn gen_bin_op(&mut self, e1: &Box<ASTNode>, e2: &Box<ASTNode>, op: &Op) {
        // todo only do literals & we need to deal with slot allocation

        // lets see if e1 and e2 can fit in registers
        match &e1.statement {
            Statement::LITERAL_NUM(i1) => match &e2.statement {
                Statement::LITERAL_NUM(i2) => match &i1.typ {
                    Type::NUMBER(n1) => match &i2.typ {
                        Type::NUMBER(n2) => {
                            // lets try doing addi
                            self.push_instruction(
                                Instruction {
                                    op_instruction: OpInstruction::ADDI,
                                    arg_0: n1.to_string().parse::<u8>().unwrap(),
                                    arg_1: n2.to_string().parse::<u8>().unwrap(),
                                    arg_2: 0,
                                },
                                0,
                            );
                            return;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::ADDI,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            0,
        );
    }
}
