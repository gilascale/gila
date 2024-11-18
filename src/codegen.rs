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
    // ADDI <r1> <r2> <desination>
    ADDI,
    // ADDI <i1> <i2> <desination>
    SUBI,
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
    current_register: u8,
    current_chunk: Chunk,
}

impl BytecodeGenerator {
    pub fn new() -> BytecodeGenerator {
        return BytecodeGenerator {
            current_register: 0,
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

        // self.push_instruction(
        //     Instruction {
        //         op_instruction: OpInstruction::LOAD_CONST,
        //         arg_0: 0,
        //         arg_1: 0,
        //         arg_2: 0,
        //     },
        //     0,
        // );
        // self.push_instruction(
        //     Instruction {
        //         op_instruction: OpInstruction::CALL,
        //         arg_0: 0,
        //         arg_1: 0,
        //         arg_2: 0,
        //     },
        //     0,
        // );

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

    fn get_available_register(&mut self) -> u8 {
        let next = self.current_register;
        self.current_register += 1;
        next
    }

    fn push_instruction(&mut self, instruction: Instruction, line: usize) {
        self.current_chunk.instructions.push(instruction);
        self.current_chunk.debug_line_info.push(line);
    }

    fn push_constant(&mut self, constant: Object) {
        self.current_chunk.constant_pool.push(constant);
    }

    fn visit(&mut self, ast: &ASTNode) -> u8 {
        match &ast.statement {
            Statement::PROGRAM(p) => self.gen_program(&p),
            Statement::BLOCK(b) => self.gen_block(&b),
            Statement::BIN_OP(e1, e2, op) => self.gen_bin_op(&e1, &e2, &op),
            _ => panic!(),
        }
    }

    fn gen_program(&mut self, p: &Vec<ASTNode>) -> u8 {
        for instruction in p {
            self.visit(instruction);
        }
        0
    }

    fn gen_block(&mut self, b: &Vec<ASTNode>) -> u8 {
        for instruction in b {
            self.visit(instruction);
        }
        0
    }

    fn parse_embedding_instruction_number(&self, typ: &Type) -> Option<u8> {
        if let Type::NUMBER(n) = typ {
            n.to_string().parse::<u8>().ok()
        } else {
            None
        }
    }

    fn gen_bin_op(&mut self, e1: &Box<ASTNode>, e2: &Box<ASTNode>, op: &Op) -> u8 {
        // todo only do literals & we need to deal with slot allocation

        // lets see if e1 and e2 can fit in registers

        if let (Statement::LITERAL_NUM(i1), Statement::LITERAL_NUM(i2)) =
            (&e1.statement, &e2.statement)
        {
            if let (Some(n1), Some(n2)) = (
                self.parse_embedding_instruction_number(&i1.typ),
                self.parse_embedding_instruction_number(&i2.typ),
            ) {
                let register = self.get_available_register();

                self.push_instruction(
                    Instruction {
                        op_instruction: match op {
                            Op::ADD => OpInstruction::ADDI,
                            Op::SUB => OpInstruction::SUBI,
                            Op::MUL => todo!(),
                            Op::DIV => todo!(),
                        },
                        arg_0: n1,
                        arg_1: n2,
                        arg_2: register,
                    },
                    0,
                );
                return register;
            }
        } else if let Statement::LITERAL_NUM(i1) = &e1.statement {
            // store the number in register 0

            let register = self.get_available_register();

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: register,
                    arg_1: self.parse_embedding_instruction_number(&i1.typ).unwrap(),
                    arg_2: register,
                },
                0,
            );

            let rhs_register = self.visit(&e2);

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADD,
                    arg_0: register,
                    arg_1: rhs_register,
                    arg_2: register,
                },
                0,
            );

            return register;
        } else if let Statement::LITERAL_NUM(i2) = &e2.statement {
            // store the number in register 0

            let register = self.get_available_register();

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: register,
                    arg_1: self.parse_embedding_instruction_number(&i2.typ).unwrap(),
                    arg_2: register,
                },
                0,
            );

            let rhs_register = self.visit(&e1);

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADD,
                    arg_0: register,
                    arg_1: rhs_register,
                    arg_2: register,
                },
                0,
            );

            return register;
        }

        panic!();
    }
}
