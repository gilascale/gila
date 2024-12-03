use deepsize::DeepSizeOf;
use std::{collections::HashMap, hash::Hash, rc::Rc, vec};

use crate::{
    ast::{ASTNode, Op, Statement},
    config::Config,
    execution::{DynamicObject, FnObject, GCRef, GCRefData, Heap, Object, StringObject},
    lex::{Position, Token, Type},
    r#type::DataType,
};

#[derive(Debug, Clone, DeepSizeOf)]
#[repr(u8)]
pub enum OpInstruction {
    // RETURN <location of values> <num values>
    RETURN = 0,

    EQUAL,
    NOT_EQUALS,
    GREATER_THAN,
    GREATER_EQUAL,
    LESS_THAN,
    LESS_EQUAL,
    LOGICAL_OR,

    LOAD_CLOSURE,

    ADD,
    // ADDI <r1> <r2> <desination>
    ADDI,
    // ADDI <i1> <i2> <desination>
    SUBI,
    // CALL <location of fn> <args starting register> <num args>
    CALL,
    // NATIVE_CALL <name of fn string> <args starting register> <num args> <destination is implicitly the register after>
    NATIVE_CALL,
    // NEW <location of type> <args starting register> <number of args>
    NEW,
    // LOAD_CONST <constant index> <> <destination>
    LOAD_CONST,
    IF_JMP_FALSE, // IF <value> <jump to instruction> <>
    // BUILD_SLICE <starting reg> <num args> <destination>
    BUILD_SLICE,
    // INDEX <item> <index> <destination>
    INDEX,
    STRUCT_ACCESS,
    // IMPORT <module path> <dest>
    IMPORT,
}

// todo put these in the enum
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
    pub current_register: u8,
    pub instructions: std::vec::Vec<Instruction>,
    // todo only enable this in debug mode
    pub debug_line_info: std::vec::Vec<usize>,
    // maybe constant pools should be global...?
    pub constant_pool: std::vec::Vec<Object>,
    pub gc_ref_data: std::vec::Vec<GCRefData>,
    pub variable_map: HashMap<Type, u8>,
    pub string_interns: HashMap<String, u8>,
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

#[derive(Clone, PartialEq)]
pub enum Annotation {
    DLL_CALL(String),
    NATIVE_CALL,
}

#[derive(Clone)]
pub struct AnnotationContext {
    pub annotations: Vec<Annotation>,
}

#[derive(Debug)]
pub struct CodegenContext {
    pub current_chunk_pointer: usize,
    pub chunks: Vec<Chunk>,
}

pub struct BytecodeGenerator<'a> {
    config: &'a Config,
    codegen_context: &'a mut CodegenContext,
}

impl BytecodeGenerator<'_> {
    pub fn new<'a>(
        config: &'a Config,
        codegen_context: &'a mut CodegenContext,
    ) -> BytecodeGenerator<'a> {
        return BytecodeGenerator {
            config,
            codegen_context,
        };
    }

    pub fn generate(&mut self, ast: &ASTNode) -> Chunk {
        let annotation_context = AnnotationContext {
            annotations: vec![],
        };
        self.visit(annotation_context, ast);

        return self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].clone();
    }

    fn get_available_register(&mut self) -> u8 {
        let next = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .current_register;
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].current_register +=
            1;
        next
    }

    fn push_instruction(&mut self, instruction: Instruction, line: usize) {
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .push(instruction);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .debug_line_info
            .push(line);
    }

    fn push_gc_ref_data(&mut self, gc_ref_data: GCRefData) -> u8 {
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .gc_ref_data
            .push(gc_ref_data);
        return (self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .gc_ref_data
            .len()
            - 1)
        .try_into()
        .unwrap();
    }
    fn push_constant(&mut self, constant: Object) -> u8 {
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .constant_pool
            .push(constant);
        return (self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .constant_pool
            .len()
            - 1)
        .try_into()
        .unwrap();
    }

    fn push_chunk(&mut self) {
        self.codegen_context.chunks.push(Chunk {
            current_register: 0,
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
            string_interns: HashMap::new(),
        });
        self.codegen_context.current_chunk_pointer += 1;
    }

    fn pop_chunk(&mut self) -> Chunk {
        let c = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].clone();
        self.codegen_context.chunks.pop();
        self.codegen_context.current_chunk_pointer -= 1;
        return c;
    }

    fn visit(&mut self, annotation_context: AnnotationContext, ast: &ASTNode) -> u8 {
        match &ast.statement {
            Statement::PROGRAM(p) => self.gen_program(annotation_context, &p),
            Statement::BLOCK(b) => self.gen_block(annotation_context, &b),
            Statement::IF(cond, body, else_body) => {
                self.gen_if(annotation_context, ast.position.clone(), &cond, &body)
            }
            Statement::VARIABLE(v) => {
                self.gen_variable(annotation_context, ast.position.clone(), v)
            }
            Statement::DEFINE(var, typ, value) => {
                self.gen_define(annotation_context, ast.position.clone(), var, value)
            }
            Statement::LITERAL_NUM(n) => {
                self.gen_literal_num(annotation_context, ast.position.clone(), n)
            }
            Statement::STRING(s) => self.gen_string(annotation_context, ast.position.clone(), s),
            Statement::CALL(b, args) => {
                self.gen_call(annotation_context, ast.position.clone(), b, args)
            }
            Statement::BIN_OP(e1, e2, op) => {
                self.gen_bin_op(annotation_context, ast.position.clone(), &e1, &e2, &op)
            }
            Statement::NAMED_FUNCTION(t, params, statement) => {
                self.gen_named_function(annotation_context, &t, &params, &statement)
            }
            Statement::NAMED_TYPE_DECL(t, decls) => {
                self.gen_named_type(annotation_context, &t, &decls)
            }
            Statement::SLICE(items) => self.gen_slice(annotation_context, &items),
            Statement::INDEX(obj, index) => self.gen_index(annotation_context, &obj, &index),
            Statement::ANNOTATION(annotation, args, expr) => {
                self.gen_annotation(annotation_context, &annotation, &args, &expr)
            }
            Statement::RETURN(value) => self.gen_return(annotation_context, &value),
            Statement::STRUCT_ACCESS(expr, field) => {
                self.gen_struct_access(annotation_context, &expr, &field)
            }
            Statement::IMPORT(path) => self.gen_import(annotation_context, path),
            _ => panic!(),
        }
    }

    fn gen_program(&mut self, annotation_context: AnnotationContext, p: &Vec<ASTNode>) -> u8 {
        for instruction in p {
            self.visit(annotation_context.clone(), instruction);
        }
        0
    }

    fn gen_block(&mut self, annotation_context: AnnotationContext, b: &Vec<ASTNode>) -> u8 {
        for instruction in b {
            self.visit(annotation_context.clone(), instruction);
        }
        0
    }

    fn gen_if(
        &mut self,
        annotation_context: AnnotationContext,
        position: Position,
        cond: &ASTNode,
        body: &ASTNode,
    ) -> u8 {
        // todo
        let value_register = self.visit(annotation_context.clone(), cond);
        let saved_if_ip = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::IF_JMP_FALSE,
                arg_0: value_register,
                arg_1: 0,
                arg_2: 0,
            },
            position.line.try_into().unwrap(),
        );
        self.visit(annotation_context.clone(), body);

        let ip = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [saved_if_ip]
            .arg_1 = ip.try_into().unwrap();

        0
    }

    fn gen_literal_num(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
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

    fn gen_string_constant(&mut self, s: String) -> u8 {
        if self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .string_interns
            .get(&s)
            .is_some()
        {
            return *self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                .string_interns
                .get(&s)
                .unwrap();
        } else {
            let gc_ref_index = self.push_gc_ref_data(GCRefData::STRING(StringObject {
                s: Rc::new(s.to_string()),
            }));
            let constant_idx = self.push_constant(Object::GC_REF(GCRef {
                index: gc_ref_index as usize,
                marked: false,
            }));
            self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                .string_interns
                .insert(s, constant_idx);
            return constant_idx;
        }
    }

    fn create_constant_string(&mut self, s: String) -> u8 {
        let constant = self.gen_string_constant(s.to_string());

        let dest = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: 0,
                arg_2: dest,
            },
            0,
        );
        dest
    }

    fn gen_string(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        s: &Token,
    ) -> u8 {
        //FIXME
        if let Type::STRING(str) = &s.typ {
            return self.create_constant_string(str.to_string());
        }
        panic!()
    }

    // todo we need a map or something to map these to registers
    fn gen_variable(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
        // todo we assume it exists so return the map
        let result = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&t.typ);

        if let Some(v) = result {
            return *v;
        } else {
            let reg = self.get_available_register();
            let mut counter = self.codegen_context.current_chunk_pointer;
            loop {
                let result = self.codegen_context.chunks[counter]
                    .variable_map
                    .get(&t.typ);
                if let Some(v) = result {
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::LOAD_CLOSURE,
                            arg_0: counter as u8,
                            arg_1: *v,
                            arg_2: reg,
                        },
                        0,
                    );
                    return reg;
                }
                if counter == 0 {
                    break;
                }
                counter -= 1;
            }
        }

        panic!("{:?}", t)
    }

    fn get_variable(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
        let result = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&t.typ);
        if let Some(v) = result {
            return *v;
        }
        panic!();
    }

    // todo we need to check if the symbol exists, if it does, then do a assign not define
    fn gen_define(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        var: &Token,
        value: &Option<Box<ASTNode>>,
    ) -> u8 {
        if value.is_none() {
            // todo definitely define, lets initialise to 'blank'
        }

        match value {
            Some(v) => {
                let location = self.visit(annotation_context, &v);
                // todo what happened here
                self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
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

    fn gen_call(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        callee: &Box<ASTNode>,
        args: &Vec<ASTNode>,
    ) -> u8 {
        if annotation_context
            .annotations
            .contains(&Annotation::NATIVE_CALL)
        {
            if let Statement::VARIABLE(v) = &callee.statement {
                if let Type::IDENTIFIER(i) = &v.typ {
                    // gen string

                    let gc_ref_index =
                        self.push_gc_ref_data(GCRefData::STRING(StringObject { s: i.clone() }));
                    let string_object = Object::GC_REF(GCRef {
                        index: gc_ref_index as usize,
                        marked: false,
                    });

                    let const_index = self.push_constant(string_object);

                    let name_reg = self.get_available_register();
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::LOAD_CONST,
                            arg_0: const_index,
                            arg_1: 0,
                            arg_2: name_reg,
                        },
                        0,
                    );

                    let mut arg_registers: Vec<u8> = vec![];
                    for arg in args {
                        arg_registers.push(self.visit(annotation_context.clone(), arg));
                    }

                    let destination = self.get_available_register();
                    let first_arg_register = {
                        if arg_registers.len() > 0 {
                            arg_registers[0]
                        } else {
                            // if we have no args, just encode the destination!
                            destination
                        }
                    };

                    // increment one as we allocate end for the return
                    self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                        .current_register += 1;
                    // figure out where to put the result
                    let destination: u8 = {
                        if arg_registers.len() > 0 {
                            arg_registers[0] + arg_registers.len() as u8
                        } else {
                            destination
                        }
                    };

                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::NATIVE_CALL,
                            arg_0: name_reg,
                            arg_1: first_arg_register,
                            arg_2: arg_registers.len() as u8,
                        },
                        0,
                    );
                    return destination;
                }
            }

            panic!();

            // todo
            // push arg
        } else {
            let callee_register = self.visit(annotation_context.clone(), &callee);

            // todo uhh how do we construct a tuple literally...

            // todo we need to find some contiguous registers, for now just alloc

            let mut arg_registers: Vec<u8> = vec![];
            for arg in args {
                arg_registers.push(self.visit(annotation_context.clone(), arg));
            }

            let destination = self.get_available_register();

            let first_arg_register = {
                if arg_registers.len() > 0 {
                    arg_registers[0]
                } else {
                    // if we have no args, just encode the destination!
                    destination
                }
            };

            // increment one as we allocate end for the return
            self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                .current_register += 1;
            // figure out where to put the result
            let destination: u8 = {
                if arg_registers.len() > 0 {
                    arg_registers[0] + arg_registers.len() as u8
                } else {
                    // if we have no args, then just encode the destination!
                    destination
                }
            };

            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::CALL,
                    arg_0: callee_register,
                    arg_1: first_arg_register,
                    arg_2: arg_registers.len() as u8,
                },
                pos.line.try_into().unwrap(),
            );

            return destination.try_into().unwrap();
        }

        0
    }

    fn gen_bin_op(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        e1: &Box<ASTNode>,
        e2: &Box<ASTNode>,
        op: &Op,
    ) -> u8 {
        // todo only do literals & we need to deal with slot allocation

        // lets see if e1 and e2 can fit in registers

        if op == &Op::ADD || op == &Op::SUB {
            if let (Statement::LITERAL_NUM(i1), Statement::LITERAL_NUM(i2)) =
                (&e1.statement, &e2.statement)
            {
                if let (Some(n1), Some(n2)) = (
                    self.parse_embedding_instruction_number(&i1.typ),
                    self.parse_embedding_instruction_number(&i2.typ),
                ) {
                    let register = self.get_available_register();

                    // todo check instruction type
                    self.push_instruction(
                        Instruction {
                            op_instruction: match op {
                                Op::ADD => OpInstruction::ADDI,
                                Op::SUB => OpInstruction::SUBI,
                                Op::MUL => todo!(),
                                Op::DIV => todo!(),
                                _ => panic!(),
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

                let rhs_register = self.visit(annotation_context, &e2);
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
                let variable_register = self.get_variable(annotation_context.clone(), pos, v1);

                let rhs_register = self.visit(annotation_context.clone(), e2);

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
                let lhs_register = self.visit(annotation_context.clone(), &e1);
                let rhs_register = self.visit(annotation_context.clone(), &e2);

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
        } else {
            let lhs = self.visit(annotation_context.clone(), e1);
            let rhs = self.visit(annotation_context.clone(), e2);
            let register = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: match op {
                        Op::EQ => OpInstruction::EQUAL,
                        Op::NEQ => OpInstruction::NOT_EQUALS,
                        Op::GT => OpInstruction::GREATER_THAN,
                        Op::GE => OpInstruction::GREATER_EQUAL,
                        Op::LT => OpInstruction::LESS_THAN,
                        Op::LE => OpInstruction::LESS_EQUAL,
                        Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                        _ => panic!(),
                    },
                    arg_0: lhs,
                    arg_1: rhs,
                    arg_2: register,
                },
                pos.line.try_into().unwrap(),
            );
            return register;
        }

        panic!();
    }

    fn gen_named_function(
        &mut self,
        annotation_context: AnnotationContext,
        token: &Token,
        params: &Vec<ASTNode>,
        statement: &ASTNode,
    ) -> u8 {
        let mut name = "anon".to_string();
        if let Type::IDENTIFIER(i) = &token.typ {
            name = i.to_string();
        }

        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::STRING(StringObject {
            s: Rc::new("tmp".to_string()),
        }));

        let constant = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));

        // todo set as a local as it is named?

        let location = self.get_available_register();

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(token.typ.clone(), location);

        // fixme do we actually need to load this?...
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: 0,
                arg_2: location,
            },
            token.pos.line.try_into().unwrap(),
        );

        // FIXME
        // this obviously has+ to be a constant

        self.push_chunk();

        // setup locals
        let mut param_slots: Vec<u8> = vec![];
        for param in params {
            if let Statement::VARIABLE(v) = &param.statement {
                let loc = self.get_available_register();
                // todo what happened here
                self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                    .variable_map
                    .insert(v.typ.clone(), loc);
                param_slots.push(loc);
            } else {
                panic!();
            }
        }

        // todo enter new block?
        self.generate(statement);

        let c = self.pop_chunk();

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].gc_ref_data
            [gc_ref_data_idx as usize] = GCRefData::FN(FnObject {
            chunk: c,
            name: name,
            param_slots: param_slots,
        });
        0
    }

    fn atom_from_type(&self, data_type: DataType) -> Object {
        match data_type {
            // todo use object "types" rather than atoms
            DataType::U32 => Object::ATOM(Rc::new("u32".to_string())),
            DataType::SLICE(t) => Object::ATOM(Rc::new("slice".to_string())),
            _ => panic!(),
        }
    }

    fn gen_named_type(
        &mut self,
        annotation_context: AnnotationContext,
        token: &Token,
        decls: &Vec<ASTNode>,
    ) -> u8 {
        // FIXME
        let mut field_definitions: HashMap<String, Object> = HashMap::new();

        for decl in decls {
            if let Statement::DEFINE(token, typ, val) = &decl.statement {
                if let Type::IDENTIFIER(i) = &token.typ {
                    field_definitions
                        .insert(i.to_string(), self.atom_from_type(typ.clone().unwrap()));
                    continue;
                }
            }
            panic!();
        }

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
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(token.typ.clone(), reg);
        // this is TERRIBLE, we need to somehow reference constants as variables
        reg
    }

    fn gen_slice(&mut self, annotation_context: AnnotationContext, items: &Vec<ASTNode>) -> u8 {
        // todo we should probably do what python does and do a BUILD_SLICE command

        let mut registers: Vec<u8> = vec![];
        for item in items {
            registers.push(self.visit(annotation_context.clone(), item));
        }

        let dest = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::BUILD_SLICE,
                arg_0: registers[0],
                arg_1: registers.len() as u8,
                arg_2: dest,
            },
            0,
        );
        dest
    }

    fn gen_index(
        &mut self,
        annotation_context: AnnotationContext,
        obj: &Box<ASTNode>,
        index: &Box<ASTNode>,
    ) -> u8 {
        let dest = self.get_available_register();

        let obj_reg = self.visit(annotation_context.clone(), &obj);
        let val_reg = self.visit(annotation_context.clone(), &index);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::INDEX,
                arg_0: obj_reg,
                arg_1: val_reg,
                arg_2: dest,
            },
            0,
        );

        dest
    }

    fn gen_annotation(
        &mut self,
        mut annotation_context: AnnotationContext,
        annotation: &Token,
        args: &Vec<Token>,
        expr: &Box<ASTNode>,
    ) -> u8 {
        if let Type::IDENTIFIER(i) = &annotation.typ {
            match i.as_str() {
                "native_call" => {
                    annotation_context.annotations.push(Annotation::NATIVE_CALL);
                }
                "dll_call" => {
                    if let Type::IDENTIFIER(i) = &args[0].typ {
                        annotation_context
                            .annotations
                            .push(Annotation::DLL_CALL(i.to_string()));
                    }
                }
                _ => panic!("unknown annotation {:?}", i),
            }
            return self.visit(annotation_context, &expr);
        }
        panic!()
    }

    fn gen_return(
        &mut self,
        mut annotation_context: AnnotationContext,
        expr: &Option<Box<ASTNode>>,
    ) -> u8 {
        let val_register = {
            if let Some(expr_val) = expr.as_ref() {
                self.visit(annotation_context.clone(), &expr_val)
            } else {
                0
            }
        };

        let num_vals = {
            if expr.is_some() {
                1
            } else {
                0
            }
        };

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::RETURN,
                arg_0: val_register,
                arg_1: num_vals,
                arg_2: 0,
            },
            0,
        );

        0
    }

    fn gen_struct_access(
        &mut self,
        mut annotation_context: AnnotationContext,
        expr: &Box<ASTNode>,
        field: &Token,
    ) -> u8 {
        // todo think how we do this... we should use indexes really

        if let Type::IDENTIFIER(i) = &field.typ {
            let lhs = self.visit(annotation_context.clone(), &expr);
            let field = self.create_constant_string(i.to_string());
            let register = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::STRUCT_ACCESS,
                    arg_0: lhs,
                    arg_1: field,
                    arg_2: register,
                },
                0,
            );
            return register;
        }
        panic!();
    }

    fn gen_import(&mut self, mut annotation_context: AnnotationContext, path: &Token) -> u8 {
        if let Type::IDENTIFIER(i) = &path.typ {
            let s = self.create_constant_string(i.to_string());
            let destination = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::IMPORT,
                    arg_0: s,
                    arg_1: destination,
                    arg_2: 0,
                },
                0,
            );
            return destination;
        }
        panic!()
    }
}
