use deepsize::DeepSizeOf;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::format,
    rc::Rc,
    vec,
};

use crate::{
    ast::{ASTNode, Op, Statement},
    config::Config,
    execution::{DynamicObject, FnObject, GCRef, GCRefData, Object, StringObject},
    lex::{Position, Token, Type},
    r#type::DataType,
};

macro_rules! current_ip {
    ($self:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .instructions
            .len()
    };
}

macro_rules! set_arg_value_at_loc {
    ($self:expr,$location:expr,$arg_num:ident,$value:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer].instructions
            [$location]
            .$arg_num = $value
    };
}

macro_rules! alloc_slot {
    ($self:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .slot_manager
            .allocate_slot()
    };
}

macro_rules! free_slot {
    ($self:expr, $slot:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .slot_manager
            .free_slot($slot)
    };
}

macro_rules! alloc_perm_slot {
    ($self:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .slot_manager
            .allocate_perm_slot()
    };
}

macro_rules! find_contiguous_slots {
    ($self:expr,$num:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .slot_manager
            .find_contiguous_slots($num)
    };
}

macro_rules! take_slot {
    ($self:expr,$num:expr) => {
        $self.codegen_context.chunks[$self.codegen_context.current_chunk_pointer]
            .slot_manager
            .take_slot($num)
    };
}

#[derive(Debug, Clone, DeepSizeOf)]
#[repr(u8)]
pub enum OpInstruction {
    // RETURN <location of values> <num values>
    RETURN = 0,
    TRY,
    EQUAL,
    NOT_EQUALS,
    GREATER_THAN,
    GREATER_EQUAL,
    LESS_THAN,
    LESS_EQUAL,
    LOGICAL_OR,
    BITWISE_OR,
    LOAD_CLOSURE,
    // MUL <r1> <r2> <desination>
    MUL,
    // DIV <r1> <r2> <desination>
    DIV,
    // ADD <r1> <r2> <desination>
    ADD,
    // ADDI <r1> <r2> <desination>
    ADDI,
    // ADDI <i1> <i2> <desination>
    SUBI,

    // FOR BOTH OF THESE, THE DESTINATION GOES IN arg_1
    // CALL <location of fn> <args starting register> <num args>
    CALL,
    // CALL_KW <location of fn> <location of tuple containing arg names> <args starting register>
    CALL_KW,

    // NATIVE_CALL <name of fn string> <args starting register> <num args> <destination is implicitly the register after>
    NATIVE_CALL,
    // LOAD_CONST <constant index> <destination>
    LOAD_CONST,
    // JUMP IF ITS FALSE
    IF_JMP_FALSE, // IF <value> <jump to instruction> <>
    // JUMP IF TRUE
    // <value> <to instruction> <>
    IF_JMP_TRUE,
    // JMP <dest>
    JMP,
    // BUILD_SLICE <starting reg> <num args> <destination>
    BUILD_SLICE,
    // BUILD_TUPLE <staring reg> <num args> <destination>
    BUILD_TUPLE,
    // BUILD_FN <code obj> <destination>
    // the purpose of this is so function specifications can be evaluated at runtime, i.e. is it static, is it a method etc.
    // it also processes default arguments etc
    BUILD_FN,
    // INDEX <item> <index> <destination>
    INDEX,
    // STRUCT_ACCESS <obj> <member> <dest>
    STRUCT_ACCESS,
    // STRUCT_SET <obj> <member> <value>
    // we store the result in arg_1 which is the member string
    STRUCT_SET,
    // IMPORT <module path> <dest>
    IMPORT,
    // FOR_ITER <iter obj> <where to jump if done> <iter result reg>
    FOR_ITER,

    // this is just a hack to make variables work
    // MOV <from> <to>
    MOV,
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

impl Instruction {
    pub fn to_string(&self) -> String {
        match self.op_instruction {
            OpInstruction::BUILD_SLICE => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
                format!("r{}", self.arg_2),
            ),
            OpInstruction::BUILD_TUPLE => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
                format!("r{}", self.arg_2),
            ),
            OpInstruction::IF_JMP_FALSE => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
            ),
            OpInstruction::IF_JMP_TRUE => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
            ),
            OpInstruction::TRY => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
            ),
            OpInstruction::ADDI => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("{}", self.arg_0),
                format!("{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::ADD => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::SUBI => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("{}", self.arg_0),
                format!("{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::BUILD_FN => format!(
                "{:>75}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
            ),
            OpInstruction::MOV => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1)
            ),
            OpInstruction::LOAD_CONST => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("{}", self.arg_0),
                format!("r{}", self.arg_1)
            ),
            OpInstruction::CALL => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("{}", self.arg_2)
            ),
            OpInstruction::CALL_KW => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::FOR_ITER => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::STRUCT_SET => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::RETURN => format!(
                "{:>75}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("{}", self.arg_1),
            ),
            OpInstruction::STRUCT_ACCESS => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::LOGICAL_OR => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::BITWISE_OR => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::GREATER_THAN => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::GREATER_EQUAL => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::LESS_THAN => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::LESS_EQUAL => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::EQUAL => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            OpInstruction::NOT_EQUALS => format!(
                "{:>75}{:>5}{:>5}{:>5}\n",
                format!("{:?}", self.op_instruction),
                format!("r{}", self.arg_0),
                format!("r{}", self.arg_1),
                format!("r{}", self.arg_2)
            ),
            _ => format!(
                "{:>75}{:5?}{:5?}{:5?}\n",
                format!("{:?}", self.op_instruction),
                self.arg_0,
                self.arg_1,
                self.arg_2
            ),
        }
    }
}

// todo custom DeepSizeOf because the other stuff doesn't mattter
#[derive(Clone, DeepSizeOf, Debug)]
pub struct Chunk {
    // pub current_register: u8,
    pub slot_manager: SlotManager,
    pub instructions: std::vec::Vec<Instruction>,
    // todo only enable this in debug mode
    pub debug_line_info: std::vec::Vec<usize>,
    // maybe constant pools should be global...?
    pub constant_pool: std::vec::Vec<Object>,
    pub gc_ref_data: std::vec::Vec<GCRefData>,
    // todo make this a hashmap of Rc<String>
    pub variable_map: HashMap<Rc<String>, u8>,
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

    pub fn dump_constant(&self, constant: &Object) -> String {
        let mut s: String = "".to_string();
        match constant {
            Object::GC_REF(gc_ref) => {
                let data = &self.gc_ref_data[gc_ref.index];

                match &data {
                    GCRefData::FN(f) => {
                        s.push_str("function ");
                        s.push_str(&f.name);
                        s.push_str(":\n");
                        s.push_str(&f.chunk.dump_instructions());
                    }
                    GCRefData::STRING(ss) => {
                        s.push_str("string: ");
                        s.push_str(&ss.s);
                    }
                    GCRefData::DYNAMIC_OBJECT(d) => {
                        s.push_str("dynamic object:\n");
                        for field in &d.fields {
                            s.push_str(&format!("    {}\n", field.0));
                        }
                    }
                    GCRefData::TUPLE(t) => {
                        s.push_str("tuple: \n");
                        for field in t.iter() {
                            s.push_str(&format!("    {:?}\n", field));
                        }
                    }
                    _ => todo!(),
                }
            }
            Object::BOOL(b) => {
                s.push_str(&format!("bool: {}", b));
            }
            Object::I64(i64) => {
                s.push_str(&format!("i64: {}", i64));
            }
            _ => todo!(),
        }
        return s;
    }

    pub fn dump_instructions(&self) -> String {
        let mut s = "".to_string();
        let mut i = 0;
        s.push_str("constants:\n");
        for constant in &self.constant_pool {
            s.push_str(&format!("{} {}\n", i, self.dump_constant(constant)));
            i += 1;
        }
        for instr in &self.instructions {
            s.push_str(&instr.to_string().as_str());
        }
        return s;
    }
    pub fn dump_to_file_format(&self, source: &String) -> String {
        let source_split = source.split('\n');

        let mut s = "".to_string();

        s.push_str("constants:\n");
        let mut i = 0;
        for constant in &self.constant_pool {
            s.push_str(&format!("{} {}\n", i, self.dump_constant(constant)));
            i += 1;
        }

        s.push_str("code:\n");
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
                    s.push_str(&instr.to_string().as_str());
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

// #[derive(Debug, Clone)]
// pub struct SlotManager {
//     allocated: HashSet<u8>,      // Tracks currently allocated temporary slots
//     allocated_perm: HashSet<u8>, // Tracks currently allocated permanent slots
//     free_slots: Vec<u8>,         // Reusable slots (stack for LIFO reuse)
// }

// impl DeepSizeOf for SlotManager {
//     // todo
//     fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
//         0
//     }
// }

// impl SlotManager {
//     pub fn new() -> Self {
//         SlotManager {
//             allocated: HashSet::new(),
//             allocated_perm: HashSet::new(),
//             free_slots: Vec::new(),
//         }
//     }

//     /// Allocate a temporary slot
//     pub fn allocate_slot(&mut self) -> u8 {
//         if let Some(slot) = self.free_slots.pop() {
//             self.allocated.insert(slot);
//             slot
//         } else {
//             let new_slot = self.next_available_slot();
//             self.allocated.insert(new_slot);
//             new_slot
//         }
//     }

//     /// Allocate a permanent slot
//     pub fn allocate_perm_slot(&mut self) -> u8 {
//         let new_slot = self.next_available_slot();
//         // todo this is a hack, we need to rewrite this system
//         for i in 0..self.free_slots.len() {
//             if self.free_slots[i] == new_slot {
//                 self.free_slots.remove(i);
//                 break;
//             }
//         }
//         self.allocated_perm.insert(new_slot);
//         new_slot
//     }

//     /// Free a temporary slot
//     pub fn free_slot(&mut self, slot: u8) {
//         // Do not free permanent slots
//         if self.allocated.remove(&slot) {
//             self.free_slots.push(slot);
//         }
//     }

//     pub fn find_contiguous_slots(&mut self, num: u8) -> u8 {
//         // Find the next unused slot (incrementally grow slot numbers)

//         for i in 0..255 {
//             let mut counter = 0;
//             while !self.is_allocated(i + counter) && counter < num {
//                 counter += 1;
//             }
//             if counter == num {
//                 return i;
//             }
//         }
//         panic!();
//     }

//     pub fn take_slot(&mut self, slot: u8) {
//         self.allocated.insert(slot);
//         for i in 0..self.free_slots.len() {
//             if self.free_slots[i] == slot {
//                 self.free_slots.remove(i);
//                 break;
//             }
//         }
//     }

//     /// Check if a slot is allocated (temporary or permanent)
//     pub fn is_allocated(&self, slot: u8) -> bool {
//         self.allocated.contains(&slot) || self.allocated_perm.contains(&slot)
//     }

//     /// Determine the next available slot
//     fn next_available_slot(&self) -> u8 {
//         // Find the next unused slot (incrementally grow slot numbers)
//         let mut new_slot = 0;
//         while self.is_allocated(new_slot) {
//             // println!("checking new slot {}", new_slot);
//             new_slot += 1;
//         }
//         new_slot
//     }
// }

#[derive(Debug, Clone)]
pub struct SlotManager {
    allocated: HashSet<u8>,      // Tracks currently allocated temporary slots
    allocated_perm: HashSet<u8>, // Tracks currently allocated permanent slots
    free_slots: VecDeque<u8>,    // Reusable slots (stack for LIFO reuse)
}

impl DeepSizeOf for SlotManager {
    // todo
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        0
    }
}

impl SlotManager {
    pub fn new() -> Self {
        SlotManager {
            allocated: HashSet::new(),
            allocated_perm: HashSet::new(),
            free_slots: VecDeque::new(),
        }
    }

    /// Allocate a temporary slot
    pub fn allocate_slot(&mut self) -> u8 {
        if let Some(slot) = self.free_slots.pop_front() {
            self.allocated.insert(slot);
            slot
        } else {
            let new_slot = self.next_available_slot();
            self.allocated.insert(new_slot);
            new_slot
        }
    }

    /// Allocate a permanent slot
    pub fn allocate_perm_slot(&mut self) -> u8 {
        let new_slot = self.next_available_slot();
        // todo this is a hack, we need to rewrite this system
        for i in 0..self.free_slots.len() {
            if self.free_slots[i] == new_slot {
                self.free_slots.remove(i);
                break;
            }
        }
        self.allocated_perm.insert(new_slot);
        new_slot
    }

    /// Free a temporary slot
    pub fn free_slot(&mut self, slot: u8) {
        // Do not free permanent slots
        if self.allocated.remove(&slot) {
            self.free_slots.push_back(slot);
        }
    }

    // given an array of existing slots i.e. [5,6,7],
    // return an array of contiguous slots that best match it.
    // [5,6,7] -> [5,6,7]
    // [5,6,8] -> [9,10,11]
    // [8,10,11] -> [12,13,14]
    pub fn find_contiguous_slots(&mut self, existing_slots: &Vec<u8>) -> Vec<u8> {
        if existing_slots.len() == 0 {
            return existing_slots.clone();
        }
        // Find the next unused slot (incrementally grow slot numbers)

        // start at the current first slot, we want to check this first which
        // will allow us to optimise. this does mean if we don't find a match at first
        // then we will always use subsequently bigger slots
        for i in existing_slots[0]..255 {
            let mut found = true;
            for j in 0..existing_slots.len() {
                let slot = i + j as u8;
                // if the slot isn't free and were not already using it then its not valid
                if self.is_allocated(slot) && slot != existing_slots[j] {
                    found = false;
                }
            }
            if found {
                let mut slots: Vec<u8> = vec![];
                for k in i..i + existing_slots.len() as u8 {
                    self.take_slot(k);
                    slots.push(k);
                }

                return slots;
            }
        }

        // for i in 0..255 {
        //     let mut counter = 0;
        //     while !self.is_allocated(i + counter) && counter < existing_slots.len() as u8 {
        //         counter += 1;
        //     }
        //     if counter == existing_slots.len() as u8 {
        //         return i;
        //     }
        // }
        panic!();
    }

    pub fn take_slot(&mut self, slot: u8) {
        self.allocated.insert(slot);
        for i in 0..self.free_slots.len() {
            if self.free_slots[i] == slot {
                self.free_slots.remove(i);
                break;
            }
        }
    }

    /// Check if a slot is allocated (temporary or permanent)
    pub fn is_allocated(&self, slot: u8) -> bool {
        self.allocated.contains(&slot) || self.allocated_perm.contains(&slot)
    }

    /// Determine the next available slot
    fn next_available_slot(&self) -> u8 {
        // Find the next unused slot (incrementally grow slot numbers)
        let mut new_slot = 0;
        while self.is_allocated(new_slot) {
            // println!("checking new slot {}", new_slot);
            new_slot += 1;
        }
        new_slot
    }
}

#[derive(Clone, Debug)]
pub struct CodegenContext {
    pub current_chunk_pointer: usize,
    pub chunks: Vec<Chunk>,
}

#[derive(Clone)]
pub struct CodegenResult {
    pub codegen_context: CodegenContext,
}

pub struct BytecodeGenerator {
    config: Config,
    codegen_context: CodegenContext,
}

impl BytecodeGenerator {
    pub fn new<'a>(config: Config, codegen_context: CodegenContext) -> BytecodeGenerator {
        return BytecodeGenerator {
            config,
            codegen_context,
        };
    }

    pub fn generate(&mut self, ast: &ASTNode) -> CodegenResult {
        let annotation_context = AnnotationContext {
            annotations: vec![],
        };
        self.visit(annotation_context, ast);

        return CodegenResult {
            codegen_context: self.codegen_context.clone(),
        };
    }

    pub fn init_builtins(&mut self) {
        let print_reg = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(Rc::new("print".to_string()), print_reg);
        let len_reg = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(Rc::new("len".to_string()), len_reg);

        let load_gila_abi_dll_perm_slot = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(
                Rc::new("load_gila_abi_dll".to_string()),
                load_gila_abi_dll_perm_slot,
            );

        let load_c_abi_dll_perm_slot = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(
                Rc::new("load_c_abi_dll".to_string()),
                load_c_abi_dll_perm_slot,
            );

        let __platform___reg = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(Rc::new("__platform__".to_string()), __platform___reg);
        let gila_socket_reg = alloc_perm_slot!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(Rc::new("gila_socket".to_string()), gila_socket_reg);
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
            slot_manager: SlotManager::new(),
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
            Statement::TUPLE(t) => self.gen_tuple(annotation_context, &ast.position.clone(), &t),
            Statement::MATCH(to_match, patterns) => {
                self.gen_match(annotation_context, &to_match, &patterns)
            }
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
            Statement::FOR(var, iter_obj, body) => self.generate_for(
                annotation_context,
                ast.position.clone(),
                &var,
                &iter_obj,
                &body,
            ),
            Statement::RANGE(first, second) => {
                self.generate_range(annotation_context, &ast.position.clone(), &first, &second)
            }
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
            Statement::ASSERT(expr, msg) => self.gen_assert(annotation_context, expr, msg),
            _ => panic!(),
        }
    }

    fn gen_program(&mut self, annotation_context: AnnotationContext, p: &Vec<ASTNode>) -> u8 {
        for instruction in p {
            let result_slot = self.visit(annotation_context.clone(), instruction);
            free_slot!(self, result_slot);
        }
        alloc_slot!(self)
    }

    fn gen_block(&mut self, annotation_context: AnnotationContext, b: &Vec<ASTNode>) -> u8 {
        for instruction in b {
            let result_slot = self.visit(annotation_context.clone(), instruction);
            // free_slot!(self, result_slot);
        }
        alloc_slot!(self)
    }

    fn gen_tuple(
        &mut self,
        annotation_context: AnnotationContext,
        position: &Position,
        t: &Vec<ASTNode>,
    ) -> u8 {
        let mut registers: Vec<u8> = vec![];
        for item in t {
            registers.push(self.visit(annotation_context.clone(), item));
        }

        let new_arg_registers = find_contiguous_slots!(self, &registers);
        for i in 0..registers.len() {
            let current_arg_reg = registers[i];
            let new_reg = new_arg_registers[i];

            if current_arg_reg != new_reg {
                // allocate the slot and do a MOV
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::MOV,
                        arg_0: current_arg_reg,
                        arg_1: new_reg,
                        arg_2: 0,
                    },
                    position.line as usize,
                );
            }
        }

        let dest = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::BUILD_TUPLE,
                arg_0: new_arg_registers[0],
                arg_1: new_arg_registers.len() as u8,
                arg_2: dest,
            },
            position.line as usize,
        );

        for reg in registers {
            free_slot!(self, reg);
        }

        for reg in new_arg_registers {
            free_slot!(self, reg);
        }

        dest
    }

    fn gen_match(
        &mut self,
        annotation_context: AnnotationContext,
        to_match: &Box<ASTNode>,
        patterns: &Vec<ASTNode>,
    ) -> u8 {
        let to_match_register = self.visit(annotation_context.clone(), &to_match);

        for match_case in patterns {
            match &match_case.statement {
                Statement::MATCH_CASE(_, expr) => {
                    self.visit(annotation_context.clone(), &expr);
                }
                _ => panic!(),
            }
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
        let func_name = match &name.statement {
            Statement::STRING(s) => format!("test_{}", s.as_string().to_string()),
            _ => panic!("expected string but got {:?}.", name),
        };
        self.create_function(
            annotation_context,
            &position,
            Rc::new(func_name),
            body,
            &vec![],
            &None,
        )
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
        let saved_if_ip = current_ip!(self);

        // if the if condition evaluates to false, we jump to the else, otherwise we execute what we got bby
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::IF_JMP_FALSE,
                arg_0: value_register,
                arg_1: 0,
                arg_2: 0,
            },
            position.line as usize,
        );
        self.visit(annotation_context.clone(), body);

        let jump_ip = current_ip!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::JMP,
                arg_0: 0,
                arg_1: 0,
                arg_2: 0,
            },
            position.line as usize,
        );

        let ip_after_body = current_ip!(self);
        if else_body.is_some() {
            self.visit(annotation_context, &else_body.as_ref().unwrap());
        }
        set_arg_value_at_loc!(self, saved_if_ip, arg_1, ip_after_body.try_into().unwrap());
        // now insert the jump after the body
        let ip_at_end = current_ip!(self);
        set_arg_value_at_loc!(self, jump_ip, arg_0, ip_at_end.try_into().unwrap());

        alloc_slot!(self)
    }

    fn lookup_var(&self, var: String) -> Option<&u8> {
        return self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&Rc::new(var));
    }

    fn generate_range(
        &mut self,
        annotation_context: AnnotationContext,
        position: &Position,
        first: &Box<ASTNode>,
        second: &Box<ASTNode>,
    ) -> u8 {
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

        let first_arg_register = self.visit(annotation_context.clone(), &first);
        let second_arg_register = self.visit(annotation_context.clone(), &second);

        let mut arg_registers = vec![first_arg_register, second_arg_register];
        let allocated_destination = alloc_slot!(self);
        arg_registers.push(allocated_destination);

        let new_arg_registers = find_contiguous_slots!(self, &arg_registers);
        for i in 0..arg_registers.len() {
            let current_arg_reg = arg_registers[i];
            let new_reg = new_arg_registers[i];

            if current_arg_reg != new_reg {
                // allocate the slot and do a MOV
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::MOV,
                        arg_0: current_arg_reg,
                        arg_1: new_reg,
                        arg_2: 0,
                    },
                    position.line as usize,
                );
            }
        }

        let new_allocated_destination = new_arg_registers[new_arg_registers.len() - 1];

        let first_arg_register: u8 = {
            if new_arg_registers.len() > 0 {
                new_arg_registers[0]
            } else {
                // if we have no args, just encode the destination!
                new_allocated_destination
            }
        };

        // now construct the duple and do the call on the RangeIterator
        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::TUPLE(kwarg_strings));
        let constant_idx = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));

        let const_reg = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant_idx,
                arg_1: const_reg,
                arg_2: 0,
            },
            position.line as usize,
        );
        let range_iterator_type = &self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&Rc::new("RangeIterator".to_string()));
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::CALL_KW,
                arg_0: *range_iterator_type.unwrap(),
                arg_1: const_reg,
                arg_2: first_arg_register,
            },
            position.line as usize,
        );

        new_allocated_destination
    }

    fn generate_for(
        &mut self,
        annotation_context: AnnotationContext,
        position: Position,
        var: &Token,
        iter_obj: &Box<ASTNode>,
        body: &Box<ASTNode>,
    ) -> u8 {
        let range_iterator_reg = self.visit(annotation_context.clone(), &iter_obj);

        let for_iter_instruction_ptr = self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        // now lets actually call the iterator!
        let iter_result_reg = alloc_slot!(self);
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
        let current_ip = current_ip!(self);
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [for_iter_instruction_ptr]
            .arg_1 = current_ip as u8;

        // todo also free the kwarg
        free_slot!(self, range_iterator_reg);
        free_slot!(self, iter_result_reg);

        // todo somehow free the range iterator arg registers!
        // for slot in &arg_registers {
        //     free_slot!(self, *slot);
        // }
        // for i in first_arg_register + 1..first_arg_register + new_arg_registers.len() as u8 + 1 {
        //     free_slot!(self, i);
        // }

        alloc_slot!(self)
    }

    fn gen_literal_num(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
        // so currently we just add to a new register
        if let Some(n) = self.parse_embedding_instruction_number(&t.typ) {
            let reg = alloc_slot!(self);
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::ADDI,
                    arg_0: 0,
                    arg_1: n,
                    arg_2: reg,
                },
                pos.line as usize,
            );
            return reg;
        } else {
            let constant_idx = match self.parse_literal_num_string(t.as_number()) {
                DataType::I64 => self.push_constant(Object::I64(self.parse_i64(t.as_number()))),
                DataType::F64 => self.push_constant(Object::F64(self.parse_f64(t.as_number()))),
                _ => panic!(),
            };

            let dest = alloc_slot!(self);
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::LOAD_CONST,
                    arg_0: constant_idx,
                    arg_1: dest,
                    arg_2: 0,
                },
                pos.line as usize,
            );
            return dest;
        }
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
        let reg = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: const_index,
                arg_1: reg,
                arg_2: 0,
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
            let reg = alloc_slot!(self);
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::LOAD_CONST,
                    arg_0: const_index,
                    arg_1: reg,
                    arg_2: 0,
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

        let dest = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: dest,
                arg_2: 0,
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
        if let Type::STRING_LITERAL(str) = &s.typ {
            return self.create_constant_string(str.to_string(), &pos);
        }
        panic!()
    }

    fn lookup_variable_recursively(
        &mut self,
        annotation_context: AnnotationContext,
        pos: &Position,
        t: &Token,
    ) -> Option<u8> {
        let result = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&t.as_identifier());

        if let Some(v) = result {
            return Some(*v);
        } else {
            let reg = alloc_slot!(self);
            let mut counter = self.codegen_context.current_chunk_pointer;
            loop {
                let result = self.codegen_context.chunks[counter]
                    .variable_map
                    .get(&t.as_identifier());
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
                    return Some(reg);
                }
                if counter == 0 {
                    return None;
                }
                counter -= 1;
            }
        }
    }

    // todo we need a map or something to map these to registers
    fn gen_variable(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
        // todo we assume it exists so return the map
        let res = self.lookup_variable_recursively(annotation_context, &pos, t);
        if res.is_some() {
            return res.unwrap();
        }
        panic!();
    }

    fn get_variable(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        t: &Token,
    ) -> u8 {
        let result = self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .get(&t.as_identifier());
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
                let location = self.visit(annotation_context.clone(), &v);

                let existing_var = self.lookup_variable_recursively(annotation_context, &pos, var);
                let var_location = if existing_var.is_some() {
                    existing_var.unwrap()
                } else {
                    alloc_perm_slot!(self)
                };

                self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                    .variable_map
                    .insert(var.as_identifier(), var_location);

                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::MOV,
                        arg_0: location,
                        arg_1: var_location,
                        arg_2: 0,
                    },
                    pos.line.try_into().unwrap(),
                );

                free_slot!(self, location);
                return var_location;
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

        alloc_slot!(self)
    }

    fn parse_embedding_instruction_number(&self, typ: &Type) -> Option<u8> {
        if let Type::NUMBER(n) = typ {
            n.to_string().parse::<u8>().ok()
        } else {
            None
        }
    }

    fn parse_i64(&self, num: Rc<String>) -> i64 {
        num.parse::<i64>().ok().unwrap()
    }

    fn parse_f64(&self, s: Rc<String>) -> f64 {
        s.parse::<f64>().ok().unwrap()
    }

    fn parse_literal_num_string(&self, num: Rc<String>) -> DataType {
        if num.contains(".") {
            return DataType::F64;
        } else {
            return DataType::I64;
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

                    let name_reg = alloc_slot!(self);
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::LOAD_CONST,
                            arg_0: const_index,
                            arg_1: name_reg,
                            arg_2: 0,
                        },
                        pos.line as usize,
                    );

                    let mut arg_registers: Vec<u8> = vec![];
                    for arg in args {
                        arg_registers.push(self.visit(annotation_context.clone(), arg));
                    }

                    let first_arg_register: u8 = {
                        if arg_registers.len() > 0 {
                            arg_registers[0]
                        } else {
                            // if we have no args, just encode the destination!
                            alloc_slot!(self)
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

                    // todo free slots
                    free_slot!(self, name_reg);
                    for i in first_arg_register + 1..first_arg_register + arg_registers.len() as u8
                    {
                        free_slot!(self, i);
                    }

                    return first_arg_register;
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

            // todo args are currently not in successive registers.
            // tod fix this we have a few options.
            // 1. pass args by a tuple for every call
            // 2. find a successive block of registers, and do MOV to put args there
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
                    }
                    _ => arg_registers.push(self.visit(annotation_context.clone(), arg)),
                }
            }
            num_kwargs = kwarg_strings.len();

            // todo find a register to put the args into!!!

            let allocated_destination = alloc_slot!(self);
            arg_registers.push(allocated_destination);

            let new_arg_registers = find_contiguous_slots!(self, &arg_registers);

            for i in 0..arg_registers.len() {
                let current_arg_reg = arg_registers[i];
                let new_reg = new_arg_registers[i];

                if current_arg_reg != new_reg {
                    // allocate the slot and do a MOV
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::MOV,
                            arg_0: current_arg_reg,
                            arg_1: new_reg,
                            arg_2: 0,
                        },
                        pos.line as usize,
                    );
                }
            }

            let new_allocated_destination = new_arg_registers[new_arg_registers.len() - 1];

            let first_arg_register: u8 = {
                if new_arg_registers.len() > 0 {
                    new_arg_registers[0]
                } else {
                    // if we have no args, just encode the destination!
                    new_allocated_destination
                }
            };

            if is_kw_call {
                // build the tuple
                // todo the issue is the gc ref strings wont get set when we init constants in the execution engine
                // so we need to init nested gc refs
                let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::TUPLE(kwarg_strings));
                let constant_idx = self.push_constant(Object::GC_REF(GCRef {
                    index: gc_ref_data_idx as usize,
                    marked: false,
                }));

                let const_reg = alloc_slot!(self);
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::LOAD_CONST,
                        arg_0: constant_idx,
                        arg_1: const_reg,
                        arg_2: 0,
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

                free_slot!(self, callee_register);
                for slot in &arg_registers {
                    free_slot!(self, *slot);
                }
                for i in
                    first_arg_register + 1..first_arg_register + new_arg_registers.len() as u8 + 1
                {
                    free_slot!(self, i);
                }

                return new_allocated_destination;
            } else {
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::CALL,
                        arg_0: callee_register,
                        arg_1: first_arg_register,
                        arg_2: arg_registers.len() as u8 - 1,
                    },
                    pos.line as usize,
                );

                free_slot!(self, callee_register);
                for slot in &arg_registers {
                    free_slot!(self, *slot);
                }
                // we -1 because the destination is in here
                for i in first_arg_register + 1..first_arg_register + arg_registers.len() as u8 - 1
                {
                    free_slot!(self, i);
                }
                return new_allocated_destination;
            }
        }
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
                    let register = alloc_slot!(self);

                    // todo check instruction type
                    self.push_instruction(
                        Instruction {
                            op_instruction: match op {
                                Op::ADD => OpInstruction::ADDI,
                                _ => panic!(),
                            },
                            arg_0: n1,
                            arg_1: n2,
                            arg_2: register,
                        },
                        pos.line.try_into().unwrap(),
                    );
                    return register;
                } else {
                    let lhs = self.visit(annotation_context.clone(), &e1);
                    let rhs = self.visit(annotation_context.clone(), &e2);
                    let register = alloc_slot!(self);

                    // todo check instruction type
                    self.push_instruction(
                        Instruction {
                            op_instruction: match op {
                                Op::ADD => OpInstruction::ADD,
                                Op::EQ => OpInstruction::EQUAL,
                                Op::NEQ => OpInstruction::NOT_EQUALS,
                                Op::GT => OpInstruction::GREATER_THAN,
                                Op::GE => OpInstruction::GREATER_EQUAL,
                                Op::LT => OpInstruction::LESS_THAN,
                                Op::LE => OpInstruction::LESS_EQUAL,
                                Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                                Op::MUL => OpInstruction::MUL,
                                Op::BITWISE_OR => OpInstruction::BITWISE_OR,
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
            } else if let Statement::LITERAL_NUM(i1) = &e1.statement {
                // store the number in register 0

                let rhs_register = self.visit(annotation_context.clone(), &e2);
                let register: u8;

                let embeddable = self.parse_embedding_instruction_number(&i1.typ);
                if embeddable.is_some() {
                    register = alloc_slot!(self);
                    self.push_instruction(
                        Instruction {
                            op_instruction: OpInstruction::ADDI,
                            arg_0: 0,
                            arg_1: embeddable.unwrap(),
                            arg_2: register,
                        },
                        pos.line as usize,
                    );
                } else {
                    register = self.visit(annotation_context.clone(), &e1);
                }

                self.push_instruction(
                    Instruction {
                        op_instruction: match op {
                            Op::ADD => OpInstruction::ADD,
                            Op::EQ => OpInstruction::EQUAL,
                            Op::NEQ => OpInstruction::NOT_EQUALS,
                            Op::GT => OpInstruction::GREATER_THAN,
                            Op::GE => OpInstruction::GREATER_EQUAL,
                            Op::LT => OpInstruction::LESS_THAN,
                            Op::LE => OpInstruction::LESS_EQUAL,
                            Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                            Op::MUL => OpInstruction::MUL,
                            Op::BITWISE_OR => OpInstruction::BITWISE_OR,
                            _ => panic!(),
                        },
                        arg_0: register,
                        arg_1: rhs_register,
                        arg_2: register,
                    },
                    pos.line as usize,
                );

                return register;
            } else if let Statement::VARIABLE(v1) = &e1.statement {
                // dealing with an identifier here so load it and perform add

                let register = alloc_slot!(self);
                let variable_register =
                    self.get_variable(annotation_context.clone(), pos.clone(), v1);

                let rhs_register = self.visit(annotation_context.clone(), e2);

                self.push_instruction(
                    Instruction {
                        op_instruction: match op {
                            Op::ADD => OpInstruction::ADD,
                            Op::EQ => OpInstruction::EQUAL,
                            Op::NEQ => OpInstruction::NOT_EQUALS,
                            Op::GT => OpInstruction::GREATER_THAN,
                            Op::GE => OpInstruction::GREATER_EQUAL,
                            Op::LT => OpInstruction::LESS_THAN,
                            Op::LE => OpInstruction::LESS_EQUAL,
                            Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                            Op::MUL => OpInstruction::MUL,
                            Op::BITWISE_OR => OpInstruction::BITWISE_OR,
                            _ => panic!(),
                        },
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

                let register = alloc_slot!(self);
                self.push_instruction(
                    Instruction {
                        op_instruction: match op {
                            Op::ADD => OpInstruction::ADD,
                            Op::EQ => OpInstruction::EQUAL,
                            Op::NEQ => OpInstruction::NOT_EQUALS,
                            Op::GT => OpInstruction::GREATER_THAN,
                            Op::GE => OpInstruction::GREATER_EQUAL,
                            Op::LT => OpInstruction::LESS_THAN,
                            Op::LE => OpInstruction::LESS_EQUAL,
                            Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                            Op::MUL => OpInstruction::MUL,
                            Op::BITWISE_OR => OpInstruction::BITWISE_OR,
                            _ => panic!(),
                        },
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
            let register = alloc_slot!(self);
            self.push_instruction(
                Instruction {
                    op_instruction: match op {
                        Op::ADD => OpInstruction::ADD,
                        Op::EQ => OpInstruction::EQUAL,
                        Op::NEQ => OpInstruction::NOT_EQUALS,
                        Op::GT => OpInstruction::GREATER_THAN,
                        Op::GE => OpInstruction::GREATER_EQUAL,
                        Op::LT => OpInstruction::LESS_THAN,
                        Op::LE => OpInstruction::LESS_EQUAL,
                        Op::LOGICAL_OR => OpInstruction::LOGICAL_OR,
                        Op::MUL => OpInstruction::MUL,
                        Op::BITWISE_OR => OpInstruction::BITWISE_OR,
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

    fn create_function(
        &mut self,
        annotation_context: AnnotationContext,
        position: &Position,
        name: Rc<String>,
        body: &ASTNode,
        params: &Vec<ASTNode>,
        return_type: &Option<DataType>,
    ) -> u8 {
        let mut is_method = false;
        let mut method_obj: u8 = 0;
        if params.len() > 0 {
            if let Statement::DEFINE(t, typ, _) = &params[0].statement {
                if let Type::IDENTIFIER(i) = &t.typ {
                    if i.to_string().eq("self") {
                        let t = typ.clone().unwrap();
                        if let DataType::NAMED_REFERENCE(d) = t {
                            is_method = true;
                            // todo add this function as a method
                            method_obj = *self.codegen_context.chunks
                                [self.codegen_context.current_chunk_pointer]
                                .variable_map
                                .get(&d)
                                .unwrap();
                        }
                    }
                }
            }
        }

        // were allocating a spot in the gc ref data slots to later put the function object
        let gc_ref_data_idx = self.push_gc_ref_data(GCRefData::STRING(StringObject {
            s: Rc::new("tmp".to_string()),
        }));
        // and then we get a constant index and load it
        let constant = self.push_constant(Object::GC_REF(GCRef {
            index: gc_ref_data_idx as usize,
            marked: false,
        }));
        let location = alloc_perm_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant,
                arg_1: location,
                arg_2: 0,
            },
            position.line.try_into().unwrap(),
        );

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(name.clone(), location);
        // after loading the function const, we the build it (this is binding self etc)
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::BUILD_FN,
                arg_0: location,
                arg_1: 0,
                arg_2: 0,
            },
            position.line as usize,
        );

        self.push_chunk();

        // setup locals
        let mut param_slots: Vec<u8> = vec![];
        for param in params {
            if let Statement::DEFINE(v, _, _) = &param.statement {
                let loc = alloc_slot!(self);
                // todo what happened here
                self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
                    .variable_map
                    .insert(v.as_identifier(), loc);
                param_slots.push(loc);
            } else {
                panic!();
            }
        }

        // todo enter new block?
        self.visit(annotation_context, body);

        let c = self.pop_chunk();

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].gc_ref_data
            [gc_ref_data_idx as usize] = GCRefData::FN(FnObject {
            chunk: c,
            name: name.clone().to_string(),
            requires_method_binding: is_method,
            method_to_object: Some(method_obj),
            param_slots: param_slots,
            bounded_object: None,
            constants_initialised: false,
        });
        // todo do we need this???
        alloc_slot!(self)
    }

    fn gen_named_function(
        &mut self,
        annotation_context: AnnotationContext,
        token: &Token,
        params: &Vec<ASTNode>,
        return_type: &Option<DataType>,
        statement: &ASTNode,
    ) -> u8 {
        self.create_function(
            annotation_context,
            &token.pos,
            token.as_identifier(),
            statement,
            params,
            return_type,
        )
    }

    fn atom_from_type(&self, data_type: DataType) -> Object {
        match data_type {
            // todo use object "types" rather than atoms
            DataType::ANY => Object::ATOM(Rc::new("any".to_string())),
            DataType::U32 => Object::ATOM(Rc::new("u32".to_string())),
            DataType::I32 => Object::ATOM(Rc::new("i32".to_string())),
            DataType::I64 => Object::ATOM(Rc::new("i64".to_string())),
            DataType::F32 => Object::ATOM(Rc::new("f32".to_string())),
            DataType::F64 => Object::ATOM(Rc::new("f64".to_string())),
            DataType::SLICE(t) => Object::ATOM(Rc::new("slice".to_string())),
            DataType::NAMED_REFERENCE(d) => Object::ATOM(Rc::new(d.to_string())),
            DataType::GENERIC(g) => Object::ATOM(Rc::new(format!("${}", g).to_string())),
            DataType::STRING => Object::ATOM(Rc::new("string".to_string())),
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

        let reg = alloc_perm_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: index,
                arg_1: reg,
                arg_2: 0,
            },
            token.pos.line.try_into().unwrap(),
        );
        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer]
            .variable_map
            .insert(token.as_identifier(), reg);
        // this is TERRIBLE, we need to somehow reference constants as variables
        reg
    }

    fn gen_slice(
        &mut self,
        annotation_context: AnnotationContext,
        pos: Position,
        items: &Vec<ASTNode>,
    ) -> u8 {
        let mut registers: Vec<u8> = vec![];
        // todo contiguous registers
        for item in items {
            registers.push(self.visit(annotation_context.clone(), item));
        }

        let new_arg_registers = find_contiguous_slots!(self, &registers);
        for i in 0..registers.len() {
            let current_arg_reg = registers[i];
            let new_reg = new_arg_registers[i];

            if current_arg_reg != new_reg {
                // allocate the slot and do a MOV
                self.push_instruction(
                    Instruction {
                        op_instruction: OpInstruction::MOV,
                        arg_0: current_arg_reg,
                        arg_1: new_reg,
                        arg_2: 0,
                    },
                    pos.line as usize,
                );
            }
        }

        let dest = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::BUILD_SLICE,
                arg_0: new_arg_registers[0],
                arg_1: new_arg_registers.len() as u8,
                arg_2: dest,
            },
            pos.line as usize,
        );

        for reg in registers {
            free_slot!(self, reg);
        }
        for reg in new_arg_registers {
            free_slot!(self, reg);
        }

        dest
    }

    fn gen_index(
        &mut self,
        annotation_context: AnnotationContext,
        obj: &Box<ASTNode>,
        index: &Box<ASTNode>,
    ) -> u8 {
        let dest = alloc_slot!(self);

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

        alloc_slot!(self)
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
            let register = alloc_slot!(self);
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

    fn gen_import(&mut self, mut annotation_context: AnnotationContext, path: &Vec<Token>) -> u8 {
        let t = &path[0];
        let mut strings: Vec<String> = vec![];
        for p in path {
            strings.push(p.as_identifier().to_string());
        }
        let path_string = strings.join(".");
        if let Type::IDENTIFIER(i) = &t.typ {
            let s = self.create_constant_string(path_string.to_string(), &t.pos);
            let destination = alloc_slot!(self);
            self.push_instruction(
                Instruction {
                    op_instruction: OpInstruction::IMPORT,
                    arg_0: s,
                    arg_1: destination,
                    arg_2: 0,
                },
                t.pos.line as usize,
            );
            return destination;
        }
        panic!()
    }

    fn gen_try(&mut self, mut annotation_context: AnnotationContext, rhs: &ASTNode) -> u8 {
        let rhs_reg = self.visit(annotation_context, rhs);

        let dest = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::TRY,
                arg_0: rhs_reg,
                arg_1: dest,
                arg_2: 0,
            },
            rhs.position.line.try_into().unwrap(),
        );

        free_slot!(self, rhs_reg);

        dest
    }

    fn gen_assert(
        &mut self,
        mut annotation_context: AnnotationContext,
        expr: &Box<ASTNode>,
        msg: &Option<Token>,
    ) -> u8 {
        let result_reg = self.visit(annotation_context, expr);

        let jmp_instr_index = self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .instructions
            .len();
        // todo if result_reg is false, then raise an error
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::IF_JMP_TRUE,
                arg_0: result_reg,
                arg_1: 0,
                arg_2: 0,
            },
            expr.position.line.try_into().unwrap(),
        );

        // setup the error value
        let mut fields = HashMap::new();
        // todo put the prototype and error object in here
        fields.insert("Error".to_string(), Object::I64(123));
        let result_object_gc_ref =
            self.push_gc_ref_data(GCRefData::DYNAMIC_OBJECT(DynamicObject { fields }));
        let obj = Object::GC_REF(GCRef {
            index: result_object_gc_ref as usize,
            marked: false,
        });
        let constant_idx = self.push_constant(obj);
        let slot = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant_idx,
                arg_1: slot,
                arg_2: 0,
            },
            expr.position.line.try_into().unwrap(),
        );
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::RETURN,
                arg_0: slot,
                arg_1: 1,
                arg_2: 0,
            },
            expr.position.line.try_into().unwrap(),
        );

        let current_instr_index = self.codegen_context.chunks
            [self.codegen_context.current_chunk_pointer]
            .instructions
            .len();

        self.codegen_context.chunks[self.codegen_context.current_chunk_pointer].instructions
            [jmp_instr_index]
            .arg_1 = current_instr_index as u8;

        // setup the ok value
        let mut fields = HashMap::new();
        // todo put the prototype and error object in here
        fields.insert("Data".to_string(), Object::I64(123));
        let result_object_gc_ref =
            self.push_gc_ref_data(GCRefData::DYNAMIC_OBJECT(DynamicObject { fields }));
        let obj = Object::GC_REF(GCRef {
            index: result_object_gc_ref as usize,
            marked: false,
        });
        let constant_idx = self.push_constant(obj);
        let slot = alloc_slot!(self);
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::LOAD_CONST,
                arg_0: constant_idx,
                arg_1: slot,
                arg_2: 0,
            },
            expr.position.line.try_into().unwrap(),
        );
        self.push_instruction(
            Instruction {
                op_instruction: OpInstruction::RETURN,
                arg_0: slot,
                arg_1: 1,
                arg_2: 0,
            },
            expr.position.line.try_into().unwrap(),
        );

        result_reg
    }
}
