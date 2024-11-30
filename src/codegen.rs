use deepsize::DeepSizeOf;
use std::{collections::HashMap, vec};

use crate::{
    ast::{ASTNode, Op, Statement},
    execution::{DynamicObject, FnObject, GCRef, GCRefData, Heap, Object, StringObject},
    lex::{Position, Token, Type},
};

#[derive(Debug, Clone, DeepSizeOf)]
#[repr(u8)]
pub enum OpInstruction {
    RETURN = 0,
    ADD,
    // ADDI <r1> <r2> <desination>
    ADDI,
    // ADDI <i1> <i2> <desination>
    SUBI,
    // NEW <location of fn> <args starting register> <destination>
    CALL,
    //
    CALL_EXTERN,
    // NEW <location of type> <args starting register> <number of args>
    NEW,
    // LOAD_CONST <constant index> <> <destination>
    LOAD_CONST,
    IF_JMP_FALSE, // IF <value> <jump to instruction> <>
}

// #[repr(packed(1))]
// all instructions are 32 bit
#[derive(Debug, Clone, DeepSizeOf)]
pub struct Instruction {
    pub op_instruction: OpInstruction,
    pub arg_0: u8,
    pub arg_1: u8,
    pub arg_2: u8,
}
#[derive(DeepSizeOf, Debug, Clone)]
pub struct Chunk {
    pub instructions: std::vec::Vec<Instruction>,
    // todo only enable this in debug mode
    pub debug_line_info: std::vec::Vec<usize>,
    // maybe constant pools should be global...?
    pub constant_pool: std::vec::Vec<Object>,
    pub gc_ref_data: std::vec::Vec<GCRefData>,
    pub variable_map: HashMap<Type, u8>,
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
    current_chunk_pointer: usize,

    // current_chunk: Chunk,
    chunks: Vec<Chunk>,
}

impl BytecodeGenerator {
    pub fn new() -> BytecodeGenerator {
        return BytecodeGenerator {
            current_register: 0,
            current_chunk_pointer: 0,
            chunks: vec![Chunk {
                debug_line_info: vec![],
                constant_pool: vec![],
                gc_ref_data: vec![],
                instructions: vec![],
                variable_map: HashMap::new(),
            }],
        };
    }

    pub fn generate(&mut self, ast: &ASTNode) -> Chunk {
        // let mut print_chunk = Chunk {
        //     instructions: vec![Instruction {
        //         op_instruction: OpInstruction::RETURN,
        //         arg_0: 0,
        //         arg_1: 0,
        //         arg_2: 0,
        //     }],
        //     debug_line_info: vec![],
        //     constant_pool: vec![],
        // };
        // self.push_constant(Object::HEAP_OBJECT(Box::new(HeapObject {
        //     data: HeapObjectData::FN(FnObject { chunk: print_chunk }),
        //     is_marked: false,
        // })));

        // todo do i want first class maps???
        // FIXME
        // self.push_constant(Object::HEAP_OBJECT(Box::new(HeapObject {
        //     data: HeapObjectData::DYNAMIC_OBJECT(DynamicObject {
        //         fields: HashMap::from([("field_definitions".to_string(), Object::I64(1))]),
        //     }),
        //     is_marked: false,
        // })));

        self.visit(ast);

        return self.chunks[self.current_chunk_pointer].clone();
    }

    fn get_available_register(&mut self) -> u8 {
        let next = self.current_register;
        self.current_register += 1;
        next
    }

    fn push_instruction(&mut self, instruction: Instruction, line: usize) {
        self.chunks[self.current_chunk_pointer]
            .instructions
            .push(instruction);
        self.chunks[self.current_chunk_pointer]
            .debug_line_info
            .push(line);
    }

    fn push_gc_ref_data(&mut self, gc_ref_data: GCRefData) -> u8 {
        self.chunks[self.current_chunk_pointer]
            .gc_ref_data
            .push(gc_ref_data);
        return (self.chunks[self.current_chunk_pointer].gc_ref_data.len() - 1)
            .try_into()
            .unwrap();
    }
    fn push_constant(&mut self, constant: Object) -> u8 {
        self.chunks[self.current_chunk_pointer]
            .constant_pool
            .push(constant);
        return (self.chunks[self.current_chunk_pointer].constant_pool.len() - 1)
            .try_into()
            .unwrap();
    }

    fn push_chunk(&mut self) {
        self.chunks.push(Chunk {
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
        });
        self.current_chunk_pointer += 1;
    }

    fn pop_chunk(&mut self) -> Chunk {
        let c = self.chunks[self.current_chunk_pointer].clone();
        self.chunks.pop();
        self.current_chunk_pointer -= 1;
        return c;
    }

    fn visit(&mut self, ast: &ASTNode) -> u8 {
        match &ast.statement {
            Statement::PROGRAM(p) => self.gen_program(&p),
            Statement::BLOCK(b) => self.gen_block(&b),
            Statement::IF(cond, body, else_body) => self.gen_if(ast.position.clone(), &cond, &body),
            Statement::VARIABLE(v) => self.gen_variable(ast.position.clone(), v),
            Statement::DEFINE(var, typ, value) => self.gen_define(ast.position.clone(), var, value),
            Statement::LITERAL_NUM(n) => self.gen_literal_num(ast.position.clone(), n),
            Statement::STRING(s) => self.gen_string(ast.position.clone(), s),
            Statement::CALL(b) => self.gen_call(ast.position.clone(), b),
            Statement::BIN_OP(e1, e2, op) => self.gen_bin_op(ast.position.clone(), &e1, &e2, &op),
            Statement::NAMED_FUNCTION(t, statement) => self.gen_named_function(&t, &statement),
            Statement::NAMED_TYPE_DECL(t, decls) => self.gen_named_type(&t, &decls),
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

    fn gen_if(&mut self, position: Position, cond: &ASTNode, body: &ASTNode) -> u8 {
        // todo
        let value_register = self.visit(cond);
        let saved_if_ip = self.chunks[self.current_chunk_pointer].instructions.len();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::IF_JMP_FALSE,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            position.line.try_into().unwrap(),
        );
        self.visit(body);

        let ip = self.chunks[self.current_chunk_pointer].instructions.len();
        self.chunks[self.current_chunk_pointer].instructions[saved_if_ip].arg_1 =
            ip.try_into().unwrap();

        0
    }

    fn gen_literal_num(&mut self, pos: Position, t: &Token) -> u8 {
        // so currently we just add to a new register
        if let Some(n) = self.parse_embedding_instruction_number(&t.typ) {
            let reg = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: 0,
                    arg_1: n,
                    arg_2: reg,
                },
                t.pos.line.try_into().unwrap(),
            );
            return reg;
        }
        panic!();
    }

    fn gen_string(&mut self, pos: Position, s: &Token) -> u8 {
        //FIXME
        0
        // if let Type::STRING(str) = &s.typ {
        //     let index = self.push_constant(Object::HEAP_OBJECT(Box::new(HeapObject {
        //         data: HeapObjectData::STRING(StringObject { s: str.clone() }),
        //         is_marked: false,
        //     })));

        //     let reg = self.get_available_register();
        //     self.push_instruction(
        //         Instruction {
        //             op_instruction: OpInstruction::LOAD_CONST,
        //             arg_0: index,
        //             arg_1: 0,
        //             arg_2: reg,
        //         },
        //         pos.line.try_into().unwrap(),
        //     );

        //     return reg;
        // }
        // panic!();
    }

    // todo we need a map or something to map these to registers
    fn gen_variable(&mut self, pos: Position, t: &Token) -> u8 {
        // todo we assume it exists so return the map
        let result = self.chunks[self.current_chunk_pointer]
            .variable_map
            .get(&t.typ);

        if let Some(v) = result {
            return *v;
        }
        panic!("{:?}", t)
    }

    fn get_variable(&mut self, pos: Position, t: &Token) -> u8 {
        let result = self.chunks[self.current_chunk_pointer]
            .variable_map
            .get(&t.typ);
        if let Some(v) = result {
            return *v;
        }
        panic!();
    }

    fn gen_define(&mut self, pos: Position, var: &Token, value: &Option<Box<ASTNode>>) -> u8 {
        match value {
            Some(v) => {
                let location = self.visit(&v);
                // todo what happened here
                self.chunks[self.current_chunk_pointer]
                    .variable_map
                    .insert(var.typ.clone(), location);
                return location;
            }
            None => panic!(),
        }
    }

    fn parse_embedding_instruction_number(&self, typ: &Type) -> Option<u8> {
        if let Type::NUMBER(n) = typ {
            n.to_string().parse::<u8>().ok()
        } else {
            None
        }
    }

    fn gen_call(&mut self, pos: Position, callee: &Box<ASTNode>) -> u8 {
        let callee_register = self.visit(&callee);

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::CALL,
                arg_0: callee_register,
                arg_1: 0,
                arg_2: 0,
            },
            pos.line.try_into().unwrap(),
        );

        0
    }

    fn gen_bin_op(&mut self, pos: Position, e1: &Box<ASTNode>, e2: &Box<ASTNode>, op: &Op) -> u8 {
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
                    pos.line.try_into().unwrap(),
                );
                return register;
            }
        } else if let Statement::LITERAL_NUM(i1) = &e1.statement {
            // store the number in register 0

            let rhs_register = self.visit(&e2);
            let register = self.get_available_register();

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: 0,
                    arg_1: self.parse_embedding_instruction_number(&i1.typ).unwrap(),
                    arg_2: register,
                },
                0,
            );

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
        } else if let Statement::VARIABLE(v1) = &e1.statement {
            // dealing with an identifier here so load it and perform add

            let register = self.get_available_register();
            let variable_register = self.get_variable(pos, v1);

            let rhs_register = self.visit(e2);

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADD,
                    arg_0: variable_register,
                    arg_1: rhs_register,
                    arg_2: register,
                },
                0,
            );

            return register;
        } else {
            let lhs_register = self.visit(&e1);
            let rhs_register = self.visit(&e2);

            let register = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADD,
                    arg_0: lhs_register,
                    arg_1: rhs_register,
                    arg_2: register,
                },
                0,
            );
        }

        panic!();
    }

    fn gen_named_function(&mut self, token: &Token, statement: &ASTNode) -> u8 {
        // FIXME
        // this obviously has+ to be a constant

        self.push_chunk();
        // todo enter new block?
        self.generate(statement);

        let c = self.pop_chunk();

        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::FN(FnObject { chunk: c }));

        let constant = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));

        // todo set as a local as it is named?

        let location = self.get_available_register();

        self.chunks[self.current_chunk_pointer]
            .variable_map
            .insert(token.typ.clone(), location);

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: 0,
                arg_2: location,
            },
            token.pos.line.try_into().unwrap(),
        );
        0
    }

    fn gen_named_type(&mut self, token: &Token, decls: &Vec<ASTNode>) -> u8 {
        // FIXME
        let field_definitions: HashMap<String, Object> =
            HashMap::from([("x".to_string(), Object::ATOM("u32".to_string().into()))]);

        let gc_ref_data_index = self.push_gc_ref_data(GCRefData::DYNAMIC_OBJECT(DynamicObject {
            fields: field_definitions,
        }));
        let index = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_index as usize,
            marked: false,
        }));

        let reg = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: index,
                arg_1: 0,
                arg_2: reg,
            },
            token.pos.line.try_into().unwrap(),
        );
        self.chunks[self.current_chunk_pointer]
            .variable_map
            .insert(token.typ.clone(), reg);
        // this is TERRIBLE, we need to somehow reference constants as variables
        reg
    }
}
