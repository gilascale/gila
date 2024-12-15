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

    // CALL_KW <location of fn> <location of tuple containing arg names> <args starting register>
    CALL_KW,

    // NATIVE_CALL <name of fn string> <args starting register> <num args> <destination is implicitly the register after>
    NATIVE_CALL,
    // NEW <location of type> <args starting register> <number of args>
    NEW,
    // LOAD_CONST <constant index> <> <destination>
    LOAD_CONST,

    // JUMP IF ITS FALSE
    IF_JMP_FALSE, // IF <value> <jump to instruction> <>

    // JMP <dest>
    JMP,

    // BUILD_SLICE <starting reg> <num args> <destination>
    BUILD_SLICE,

    // BUILD_FN <code obj> <destination>
    // the purpose of this is so function specifications can be evaluated at runtime, i.e. is it static, is it a method etc.
    // it also processes default arguments etc
    BUILD_FN,

    // INDEX <item> <index> <destination>
    INDEX,

    STRUCT_ACCESS,

    // STRUCT_SET <obj> <member> <value>
    // we store the result in arg_1 which is the member string
    STRUCT_SET,

    // IMPORT <module path> <dest>
    IMPORT,
    // FOR_ITER <iter obj> <where to jump if done> <iter result reg>
    FOR_ITER,
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

    pub fn dump_to_file_format(&self, source: &String) -> String {
        let source_split = source.split('\n');

        let mut s = "".to_string();

        let mut lines_for_instr: HashMap<usize, Vec<Instruction>> = HashMap::new();

        let mut i = 0;
        for instr in &self.instructions {
            let line = self.debug_line_info[i];

            if !lines_for_instr.contains_key(&line) {
                lines_for_instr.insert(line, vec![]);
            }

            lines_for_instr.get_mut(&line).unwrap().push(instr.clone());

            i += 1;
        }

        let mut i = 0;
        for line in source_split {
            s.push_str(format!("{:<5}{}", i, line).as_str());

            let all_instructions = lines_for_instr.get(&i);
            if all_instructions.is_some() {
                for instr in all_instructions.unwrap() {
                    s.push_str(
                        format!(
                            "{:>75}{:3?}{:3?}{:3?}\n",
                            format!("{:?}", instr.op_instruction),
                            instr.arg_0,
                            instr.arg_1,
                            instr.arg_2
                        )
                        .as_str(),
                    );
                }
            }
            i += 1;
        }
        return s;
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
            Statement::TEST(name, body) => {
                self.gen_test(annotation_context, ast.position.clone(), &name, &body)
            }
            Statement::IF(cond, body, else_body) => self.gen_if(
                annotation_context,
                ast.position.clone(),
                &cond,
                &body,
                &else_body,
            ),
            Statement::FOR(var, range_start, range_end, body) => self.generate_for(
                annotation_context,
                ast.position.clone(),
                &var,
                &range_start,
                &range_end,
                &body,
            ),
            Statement::VARIABLE(v) => {
                self.gen_variable(annotation_context, ast.position.clone(), v)
            }
            Statement::DEFINE(var, typ, value) => {
                self.gen_define(annotation_context, ast.position.clone(), var, value)
            }
            Statement::ASSIGN(lhs, rhs) => {
                self.gen_assign(annotation_context, ast.position.clone(), lhs, rhs)
            }
            Statement::LITERAL_NUM(n) => {
                self.gen_literal_num(annotation_context, ast.position.clone(), n)
            }
            Statement::LITERAL_BOOL(b) => {
                self.gen_literal_bool(annotation_context, ast.position.clone(), *b)
            }
            Statement::ATOM(a) => self.gen_atom(annotation_context, ast.position.clone(), a),
            Statement::STRING(s) => self.gen_string(annotation_context, ast.position.clone(), s),
            Statement::CALL(b, args) => {
                self.gen_call(annotation_context, ast.position.clone(), b, args)
            }
            Statement::BIN_OP(e1, e2, op) => {
                self.gen_bin_op(annotation_context, ast.position.clone(), &e1, &e2, &op)
            }
            Statement::NAMED_FUNCTION(t, params, return_type, statement) => {
                self.gen_named_function(annotation_context, &t, &params, &return_type, &statement)
            }
            Statement::NAMED_TYPE_DECL(t, decls) => {
                self.gen_named_type(annotation_context, &t, &decls)
            }
            Statement::SLICE(items) => {
                self.gen_slice(annotation_context, ast.position.clone(), &items)
            }
            Statement::INDEX(obj, index) => self.gen_index(annotation_context, &obj, &index),
            Statement::ANNOTATION(annotation, args, expr) => {
                self.gen_annotation(annotation_context, &annotation, &args, &expr)
            }
            Statement::RETURN(value) => self.gen_return(annotation_context, &ast.position, &value),
            Statement::STRUCT_ACCESS(expr, field) => {
                self.gen_struct_access(annotation_context, &expr, &field)
            }
            Statement::IMPORT(path) => self.gen_import(annotation_context, path),
            Statement::TRY(rhs) => self.gen_try(annotation_context, rhs),
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

    fn gen_test(
        &mut self,
        annotation_context: AnnotationContext,
        position: Position,
        name: &ASTNode,
        body: &ASTNode,
    ) -> u8 {
        0
    }

    fn gen_if(
        &mut self,
        annotation_context: AnnotationContext,
        position: Position,
        cond: &ASTNode,
        body: &ASTNode,
        else_body: &Option<Box<ASTNode>>,
    ) -> u8 {
        // todo
        let value_register = self.visit(annotation_context.clone(), cond);
        let saved_if_ip = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();

        // if the if condition evaluates to false, we jump to the else, otherwise we execute what we got bby
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

        let jump_ip = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::JMP,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            0,
        );

        let ip_after_body = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();

        if else_body.is_some() {
            self.visit(annotation_context, &else_body.as_ref().unwrap());
        }

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [saved_if_ip]
            .arg_1 = ip_after_body.try_into().unwrap();

        // now insert the jump after the body
        let ip_at_end = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [jump_ip]
            .arg_0 = ip_at_end.try_into().unwrap();

        0
    }

    fn lookup_var(&self, var: String) -> Option<&u8> {
        return self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&Type::IDENTIFIER(Rc::new(var)));
    }

    fn generate_for(
        &mut self,
        annotation_context: AnnotationContext,
        position: Position,
        var: &Token,
        range_start: &Token,
        range_end: &Token,
        body: &Box<ASTNode>,
    ) -> u8 {
        // construct the iterator

        // setup the ("counter", "limit") tuple
        let mut kwarg_strings: Vec<Object> = vec![];
        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::STRING(StringObject {
            s: Rc::new("counter".to_owned()),
        }));
        kwarg_strings.push(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));
        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::STRING(StringObject {
            s: Rc::new("limit".to_owned()),
        }));
        kwarg_strings.push(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));

        let counter = match &range_start.typ {
            Type::NUMBER(n) => n.parse::<u8>().unwrap(),
            _ => panic!(),
        };
        let limit = match &range_end.typ {
            Type::NUMBER(n) => n.parse::<u8>().unwrap(),
            _ => panic!(),
        };

        // get the numbers setup for the 0..3
        let first_arg_register = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::ADDI,
                arg_0: 0,
                arg_1: counter,
                arg_2: first_arg_register,
            },
            position.line as usize,
        );
        let second_arg_register = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::ADDI,
                arg_0: 0,
                arg_1: limit,
                arg_2: second_arg_register,
            },
            position.line as usize,
        );

        // now construct the duple and do the call on the RangeIterator
        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::TUPLE(kwarg_strings));
        let constant_idx = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));

        let const_reg = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant_idx,
                arg_1: 0,
                arg_2: const_reg,
            },
            position.line as usize,
        );
        let range_iterator_type = &self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&Type::IDENTIFIER(Rc::new("RangeIterator".to_string())));
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::CALL_KW,
                arg_0: *range_iterator_type.unwrap(),
                arg_1: const_reg,
                arg_2: first_arg_register,
            },
            position.line as usize,
        );

        let range_iterator_reg = first_arg_register + 2;

        let for_iter_instruction_ptr = self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        // now lets actually call the iterator!
        let iter_result_reg = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::FOR_ITER,
                arg_0: range_iterator_reg,
                arg_1: 0, // todo later on in this fn we need to set this to the end
                arg_2: iter_result_reg,
            },
            position.line as usize,
        );
        self.visit(annotation_context, &body);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::JMP,
                arg_0: for_iter_instruction_ptr as u8,
                arg_1: 0,
                arg_2: 0,
            },
            position.line as usize,
        );
        let current_ip = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [for_iter_instruction_ptr]
            .arg_1 = current_ip as u8;

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

    fn gen_literal_bool(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        b: bool,
    ) -> u8 {
        // we need to push the atom as a constant?

        // todo maybe have a constant hashmap?
        let const_index = self.push_constant(Object::BOOL(b));
        let reg = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: const_index,
                arg_1: 0,
                arg_2: reg,
            },
            pos.line.try_into().unwrap(),
        );
        return reg;
    }

    fn gen_atom(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        atom: &Token,
    ) -> u8 {
        // we need to push the atom as a constant?

        if let Type::IDENTIFIER(i) = &atom.typ {
            let const_index = self.push_constant(Object::ATOM(i.clone()));
            let reg = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::LOAD_CONST,
                    arg_0: const_index,
                    arg_1: 0,
                    arg_2: reg,
                },
                pos.line.try_into().unwrap(),
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

    fn create_constant_string(&mut self, s: String, position: &Position) -> u8 {
        let constant = self.gen_string_constant(s.to_string());

        let dest = self.get_available_register();
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: 0,
                arg_2: dest,
            },
            position.line as usize,
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
            return self.create_constant_string(str.to_string(), &pos);
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
                        pos.line as usize,
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

    fn gen_assign(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        lhs: &Box<ASTNode>,
        rhs: &Box<ASTNode>,
    ) -> u8 {
        match &lhs.statement {
            Statement::STRUCT_ACCESS(obj_to_access, token) => {
                if let Type::IDENTIFIER(i) = &token.typ {
                    let obj_to_access_reg = self.visit(annotation_context.clone(), &obj_to_access);
                    let string_reg = self.create_constant_string(i.to_string(), &pos);

                    let value_reg = self.visit(annotation_context.clone(), rhs);

                    // todo how do we do this?
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::STRUCT_SET,
                            arg_0: obj_to_access_reg,
                            arg_1: string_reg,
                            arg_2: value_reg,
                        },
                        pos.line.try_into().unwrap(),
                    );

                    return string_reg;
                }
                panic!()
            }
            _ => todo!(),
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
                        pos.line as usize,
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
                        pos.line as usize,
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

            let mut is_kw_call = false;
            let mut kw_args_vec: Vec<Object> = vec![];

            let mut kwarg_strings: Vec<Object> = vec![];
            let mut num_kwargs = 0;

            let mut arg_registers: Vec<u8> = vec![];
            for arg in args {
                match &arg.statement {
                    Statement::ASSIGN(lhs, rhs) => {
                        is_kw_call = true;
                        match &lhs.statement {
                            Statement::VARIABLE(v) => match &v.typ {
                                Type::IDENTIFIER(i) => {
                                    let gc_ref_data_idx =
                                        self.push_gc_ref_data(GCRefData::STRING(StringObject {
                                            s: i.clone(),
                                        }));
                                    kwarg_strings.push(Object::GC_REF(GCRef {
                                        index: gc_ref_data_idx as usize,
                                        marked: false,
                                    }));

                                    arg_registers.push(self.visit(annotation_context.clone(), rhs))
                                }
                                _ => panic!(),
                            },
                            _ => panic!(),
                        }

                        // todo CALL_KW
                    }
                    _ => arg_registers.push(self.visit(annotation_context.clone(), arg)),
                }
            }
            num_kwargs = kwarg_strings.len();

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

            if is_kw_call {
                // build the tuple
                // todo the issue is the gc ref strings wont get set when we init constants in the execution engine
                // so we need to init nested gc refs
                let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::TUPLE(kwarg_strings));
                let constant_idx = self.push_constant(Object::GC_REF(GCRef {
                    index: gc_ref_data_idx as usize,
                    marked: false,
                }));

                let const_reg = self.get_available_register();
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::LOAD_CONST,
                        arg_0: constant_idx,
                        arg_1: 0,
                        arg_2: const_reg,
                    },
                    pos.line as usize,
                );

                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::CALL_KW,
                        arg_0: callee_register,
                        arg_1: const_reg,
                        arg_2: first_arg_register,
                    },
                    pos.line as usize,
                );

                return first_arg_register + num_kwargs as u8;
            } else {
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::CALL,
                        arg_0: callee_register,
                        arg_1: first_arg_register,
                        arg_2: arg_registers.len() as u8,
                    },
                    pos.line as usize,
                );
            }

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
                    pos.line as usize,
                );

                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::ADD,
                        arg_0: register,
                        arg_1: rhs_register,
                        arg_2: register,
                    },
                    pos.line as usize,
                );

                return register;
            } else if let Statement::VARIABLE(v1) = &e1.statement {
                // dealing with an identifier here so load it and perform add

                let register = self.get_available_register();
                let variable_register =
                    self.get_variable(annotation_context.clone(), pos.clone(), v1);

                let rhs_register = self.visit(annotation_context.clone(), e2);

                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::ADD,
                        arg_0: variable_register,
                        arg_1: rhs_register,
                        arg_2: register,
                    },
                    pos.line as usize,
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
                    pos.line as usize,
                );

                return register;
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

        panic!("failing bin op {:?}", op);
    }

    fn gen_named_function(
        &mut self,
        annotation_context: AnnotationContext,
        token: &Token,
        params: &Vec<ASTNode>,
        return_type: &Option<DataType>,
        statement: &ASTNode,
    ) -> u8 {
        // how do

        // todo check if its a method!

        let mut is_method = false;
        let mut method_obj: u8 = 0;
        if params.len() > 0 {
            if let Statement::DEFINE(t, typ, _) = &params[0].statement {
                if let Type::IDENTIFIER(i) = &t.typ {
                    if i.to_string().eq("self") {
                        let t = typ.clone().unwrap();
                        if let DataType::DYNAMIC_OBJECT(d) = t {
                            is_method = true;
                            // todo add this function as a method
                            method_obj = *self.codegen_context.chunks
                                [self.codegen_context.current_chunk_pointer]
                                .variable_map
                                .get(&Type::IDENTIFIER(d))
                                .unwrap();
                        }
                    }
                }
            }
        }

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

        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::BUILD_FN,
                arg_0: location,
                arg_1: 0,
                arg_2: 0,
            },
            token.pos.line as usize,
        );

        // FIXME
        // this obviously has+ to be a constant

        self.push_chunk();

        // setup locals
        let mut param_slots: Vec<u8> = vec![];
        for param in params {
            if let Statement::DEFINE(v, _, _) = &param.statement {
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
            requires_method_binding: is_method,
            method_to_object: Some(method_obj),
            param_slots: param_slots,
            bounded_object: None,
        });
        0
    }

    fn atom_from_type(&self, data_type: DataType) -> Object {
        match data_type {
            // todo use object "types" rather than atoms
            DataType::U32 => Object::ATOM(Rc::new("u32".to_string())),
            DataType::SLICE(t) => Object::ATOM(Rc::new("slice".to_string())),
            DataType::DYNAMIC_OBJECT(d) => Object::ATOM(Rc::new(d.to_string())),
            _ => panic!("cant create atom from type {:?}", data_type),
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

    fn gen_slice(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        items: &Vec<ASTNode>,
    ) -> u8 {
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
            pos.line as usize,
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
            obj.position.line as usize,
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
        pos: &Position,
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
            pos.line as usize,
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
            let field = self.create_constant_string(i.to_string(), &expr.position);
            let register = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::STRUCT_ACCESS,
                    arg_0: lhs,
                    arg_1: field,
                    arg_2: register,
                },
                expr.position.line as usize,
            );
            return register;
        }
        panic!();
    }

    fn gen_import(&mut self, mut annotation_context: AnnotationContext, path: &Token) -> u8 {
        if let Type::IDENTIFIER(i) = &path.typ {
            let s = self.create_constant_string(i.to_string(), &path.pos);
            let destination = self.get_available_register();
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::IMPORT,
                    arg_0: s,
                    arg_1: destination,
                    arg_2: 0,
                },
                path.pos.line as usize,
            );
            return destination;
        }
        panic!()
    }

    fn gen_try(&mut self, mut annotation_context: AnnotationContext, rhs: &ASTNode) -> u8 {
        // first generate the rhs
        let rhs_reg = self.visit(annotation_context, rhs);

        // todo we need to now insert some code that checks the result

        rhs_reg
    }
}
