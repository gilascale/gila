use core::slice;
use deepsize::DeepSizeOf;
use std::{collections::HashMap, fmt::format, fs::File, rc::Rc};

use std::os::windows::io::AsRawHandle;

use crate::{
    codegen::{Chunk, Instruction, OpInstruction},
    config::Config,
};

#[derive(Debug)]
pub enum RuntimeError {
    INVALID_OPERATION,
    OUT_OF_MEMORY,
}
#[derive(DeepSizeOf, Debug, Clone)]
pub struct DynamicObject {
    // todo perhaps this should be builtin-strings or RC'd?
    pub fields: HashMap<String, Object>,
}

impl DynamicObject {
    pub fn new(map: HashMap<String, Object>) -> Self {
        DynamicObject { fields: map }
    }

    pub fn print(&self) -> String {
        return format!("DynamicObject={:?}", self.fields);
    }
}
#[derive(DeepSizeOf, Debug, Clone)]
pub struct FnObject {
    pub chunk: Chunk,
    pub name: String,
    pub param_slots: Vec<u8>,
}

// todo should this be Rc'd?
#[derive(Debug, Clone, DeepSizeOf)]
pub struct StringObject {
    pub s: Rc<String>,
}

#[derive(Debug, Clone, DeepSizeOf)]
pub struct SliceObject {
    pub s: Vec<Object>,
}

#[derive(DeepSizeOf, Debug, Clone)]
pub enum GCRefData {
    FN(FnObject),
    STRING(StringObject),
    SLICE(SliceObject),
    DYNAMIC_OBJECT(DynamicObject),
}

impl GCRefData {
    pub fn print(&self) -> String {
        match self {
            Self::STRING(s) => s.s.to_string(),
            _ => panic!(),
        }
    }
}

#[derive(DeepSizeOf, Debug, Clone)]
pub enum Object {
    BOOL(bool),
    F64(f64),
    I64(i64),
    ATOM(Rc<String>),
    GC_REF(GCRef),
}

impl Object {
    // pub fn create_heap_obj(heap_obj_data: HeapObjectData) -> Self {
    //     Object::HEAP_OBJECT(Box::new(HeapObject {
    //         data: heap_obj_data,
    //         is_marked: false,
    //     }))
    // }

    pub fn get_type(&self) -> Object {
        match self {
            // Self::I64(_) => {
            //     Object::create_heap_obj(HeapObjectData::DYNAMIC_OBJECT(DynamicObject::new(
            //         HashMap::from([("name".to_string(), Object::ATOM(Rc::new("I64".to_string())))]),
            //     )))
            // }
            _ => panic!(),
        }
    }

    pub fn print(&self) -> std::string::String {
        match self {
            Self::BOOL(b) => b.to_string(),
            Self::F64(f) => f.to_string(),
            Self::I64(i) => i.to_string(),
            Self::ATOM(a) => format!(":{:?}", a.to_string()),
            Self::GC_REF(r) => format!("GCRef {:?}", r.index),
            // Self::HEAP_OBJECT(h) => h.print(),
        }
    }

    pub fn add(&self, other: Object) -> Result<Object, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(Object::I64(i1 + i2)),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn equals(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 == i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Self::F64(f) => return f > &0.0,
            Self::I64(i) => return i > &0,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct StackFrame {
    // the register in the previous stack to place return values
    pub return_register: u8,
    pub instruction_pointer: usize,
    pub stack: std::vec::Vec<Object>,
    // todo this sucks
    pub fn_object: Box<FnObject>,
}

#[derive(Debug, DeepSizeOf)]
pub struct Heap {
    // linked list of objects
    pub live_slots: Vec<GCRefData>,
    pub dead_objects: Vec<usize>,
}

#[derive(Debug, Clone, DeepSizeOf)]
pub struct GCRef {
    pub index: usize,
    pub marked: bool,
}

impl Heap {
    pub fn alloc(&mut self, gc_ref_dat: GCRefData, config: &Config) -> Result<GCRef, RuntimeError> {
        if self.free_space_available_bytes() >= config.max_memory {
            return Err(RuntimeError::OUT_OF_MEMORY);
        }

        // todo for now just push to end
        self.live_slots.push(gc_ref_dat);
        Ok(GCRef {
            index: self.live_slots.len() - 1,
            marked: false,
        })
    }

    pub fn deref(&mut self, gc_ref: &GCRef) -> GCRefData {
        return self.live_slots[gc_ref.index].clone();
    }

    pub fn free_space_available_bytes(&self) -> usize {
        self.live_slots.deep_size_of()
    }
}

type NativeFn = fn(&mut ExecutionContext, Vec<Object>) -> Object;

pub struct ExecutionContext {
    pub stack_frames: std::vec::Vec<StackFrame>,
    pub stack_frame_pointer: usize,
    pub heap: Heap,
    pub native_fns: HashMap<String, NativeFn>,
}

fn native_print(execution_context: &mut ExecutionContext, args: Vec<Object>) -> Object {
    let s: String = match &args[0] {
        Object::GC_REF(gc_ref) => execution_context.heap.deref(&gc_ref).print(),
        Object::I64(i) => i.to_string(),
        _ => panic!(),
    };

    println!("{}", s);
    return Object::I64(0);
}

fn native_open_windows(execution_context: &mut ExecutionContext, args: Vec<Object>) -> Object {
    if let Object::GC_REF(gc_ref) = &args[0] {
        if let GCRefData::STRING(s) = execution_context.heap.deref(&gc_ref) {
            let file = File::open(s.s.to_string());
            if let Ok(file) = file {
                let handle = file.as_raw_handle();
                return Object::I64(handle as i64);
            }
        }
    }

    return Object::I64(0);
}

pub struct ExecutionEngine<'a> {
    pub config: &'a Config,
    pub running: bool,
    pub environment: &'a mut ExecutionContext,
}

impl<'a> ExecutionEngine<'a> {
    pub fn new(config: &'a Config, environment: &'a mut ExecutionContext) -> ExecutionEngine<'a> {
        ExecutionEngine {
            config,
            running: true,
            environment,
        }
    }

    pub fn register_native_fn(&mut self, name: String, native_fn: NativeFn) {
        self.environment.native_fns.insert(name, native_fn);
    }

    pub fn exec(&mut self, bytecode: Chunk, is_repl: bool) -> Result<Object, RuntimeError> {
        self.register_native_fn("native_print".to_string(), native_print);
        self.register_native_fn("native_open_windows".to_string(), native_open_windows);

        self.running = true;

        // todo think how do do this with preludes...
        // if is_repl {
        if self.environment.stack_frames.len() == 0 {
            self.init_startup_stack(Box::new(FnObject {
                chunk: bytecode,
                name: "main".to_string(),
                param_slots: vec![],
            }));
            self.zero_stack();
            self.init_constants();
        } else {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .fn_object
                .chunk = bytecode;
        }
        // } else {
        //     self.init_startup_stack(Box::new(FnObject {
        //         chunk: bytecode,
        //         name: "main".to_string(),
        //         param_slots: vec![],
        //     }));
        //     self.zero_stack();
        //     self.init_constants();
        // }

        let mut reg = 0;
        while self.running {
            let instr = {
                let current_frame =
                    &self.environment.stack_frames[self.environment.stack_frame_pointer];
                &current_frame.fn_object.chunk.instructions[current_frame.instruction_pointer]
                    .clone()
            };

            let reg_result = self.exec_instr(instr);

            if let Err(e) = reg_result {
                return Err(e);
            }

            reg = reg_result.unwrap();

            if self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer
                == self.environment.stack_frames[self.environment.stack_frame_pointer]
                    .fn_object
                    .chunk
                    .instructions
                    .len()
            {
                self.running = false;
                self.environment.stack_frames[self.environment.stack_frame_pointer]
                    .instruction_pointer = 0;
            }
        }

        // todo return reference
        return Ok(
            self.environment.stack_frames[self.environment.stack_frame_pointer].stack[reg as usize]
                .clone(),
        );
    }

    fn init_constants(&mut self) -> Result<(), RuntimeError> {
        // todo
        // for each gc ref data constant on the current chunk, put it in the heap and update the reference

        let stack_frame = &mut self.environment.stack_frames[self.environment.stack_frame_pointer];

        let constant_pool = &mut stack_frame.fn_object.chunk.constant_pool;
        // todo this is HORRIBLE
        let gc_ref_data = &stack_frame.fn_object.chunk.gc_ref_data.clone();

        for i in constant_pool.iter_mut() {
            if let Object::GC_REF(gc_ref) = i {
                let gc_ref_data = &gc_ref_data[gc_ref.index];

                // now lets heap allocate!
                let alloc = self
                    .environment
                    .heap
                    .alloc(gc_ref_data.clone(), &self.config);
                match alloc {
                    Ok(_) => gc_ref.index = alloc.unwrap().index,
                    Err(e) => return Err(e),
                }
            }
        }

        return Ok(());
    }

    fn exec_instr(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        match instr.op_instruction {
            OpInstruction::RETURN => self.exec_return(instr),
            OpInstruction::EQUAL => self.exec_equal(instr),
            OpInstruction::NOT_EQUALS => self.exec_nequal(instr),
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::SUBI => self.exec_subi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            OpInstruction::CALL => self.exec_call(instr),
            OpInstruction::NATIVE_CALL => self.exec_native_call(instr),
            OpInstruction::NEW => self.exec_new(instr),
            OpInstruction::LOAD_CONST => self.exec_load_const(instr),
            OpInstruction::IF_JMP_FALSE => self.exec_if_jmp_false(instr),
            OpInstruction::BUILD_SLICE => self.exec_build_slice(instr),
            OpInstruction::INDEX => self.exec_index(instr),
            _ => panic!("unknown instruction {:?}", instr.op_instruction),
        }
    }

    fn init_startup_stack(&mut self, fn_object: Box<FnObject>) {
        self.environment.stack_frames.push(StackFrame {
            stack: vec![],
            fn_object: fn_object,
            instruction_pointer: 0,
            return_register: 0,
        });
        self.environment.stack_frame_pointer = 0;
    }

    fn push_stack_frame(&mut self, fn_object: Box<FnObject>, return_register: u8) {
        self.environment.stack_frames.push(StackFrame {
            stack: vec![],
            fn_object: fn_object,
            instruction_pointer: 0,
            return_register,
        });
        self.environment.stack_frame_pointer += 1;
    }

    fn zero_stack(&mut self) {
        // fixme dynamically setup stack
        for _ in 0..255 {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .stack
                .push(Object::I64(0));
        }
    }

    pub fn print_stacktrace(&mut self) {
        println!("stacktrace:");
        let mut i = 0;
        while i <= 0 {
            println!("--- {}", self.environment.stack_frames[i].fn_object.name);
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    fn exec_return(&mut self, ret: &Instruction) -> Result<u8, RuntimeError> {
        let return_register =
            self.environment.stack_frames[self.environment.stack_frame_pointer].return_register;
        let return_val = self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [ret.arg_0 as usize]
            .clone();

        self.environment.stack_frames.pop();
        if self.environment.stack_frames.len() == 0 {
            self.running = false;
        } else {
            self.environment.stack_frame_pointer -= 1;
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;

            if ret.arg_1 > 0 {
                self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                    [return_register as usize] = return_val;
            }
        }

        Ok(return_register)
    }

    fn exec_equal(&mut self, equal: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_0 as usize];
        let rhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_1 as usize];

        let result = lhs.equals(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_2 as usize] = Object::BOOL(result.unwrap());

        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;
        Ok(equal.arg_2)
    }

    fn exec_nequal(&mut self, equal: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_0 as usize];
        let rhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_1 as usize];

        let result = lhs.equals(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [equal.arg_2 as usize] = Object::BOOL(!result.unwrap());
        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;
        Ok(equal.arg_2)
    }

    fn exec_addi(&mut self, addi: &Instruction) -> Result<u8, RuntimeError> {
        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [addi.arg_2 as usize] = Object::I64((addi.arg_0 + addi.arg_1).into());

        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;

        Ok(addi.arg_2)
    }
    fn exec_subi(&mut self, subi: &Instruction) -> Result<u8, RuntimeError> {
        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [subi.arg_2 as usize] = Object::I64((subi.arg_0 - subi.arg_1).into());

        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;

        Ok(subi.arg_2)
    }

    fn exec_add(&mut self, add: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [add.arg_0 as usize];
        let rhs = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [add.arg_1 as usize];

        let addition: Result<Object, RuntimeError> = lhs.add(rhs.clone());
        if let Ok(res) = addition {
            self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                [add.arg_2 as usize] = res;
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;

            return Ok(add.arg_2);
        } else {
            return Err(addition.err().unwrap());
        }
    }

    fn exec_call(&mut self, call: &Instruction) -> Result<u8, RuntimeError> {
        let fn_object = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [call.arg_0 as usize];
        let gc_ref_object: &GCRef = match &fn_object {
            Object::GC_REF(r) => r,
            _ => panic!("can only call func or constructor"),
        };

        let dereferenced_data = self.environment.heap.deref(gc_ref_object);

        match &dereferenced_data {
            GCRefData::FN(f) => {
                let destination = {
                    if call.arg_2 > 0 {
                        call.arg_1 + call.arg_2
                    } else {
                        (call.arg_1 + 1).into()
                    }
                };
                // fixme this sucks, we shouldn't clone functions it's so expensive
                // fixme why is this a Box?
                self.push_stack_frame(Box::new(f.clone()), destination);
                self.zero_stack();
                self.init_constants();

                // pass the args by value
                let starting_reg = call.arg_1;
                let num_args = call.arg_2;

                for i in 0..num_args {
                    let arg_register = starting_reg as usize + i as usize;
                    let arg = &self.environment.stack_frames
                        [self.environment.stack_frame_pointer - 1]
                        .stack[arg_register];
                    self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                        [f.param_slots[i as usize] as usize] = arg.clone();
                }

                return Ok(call.arg_1 + call.arg_2);
            }
            GCRefData::DYNAMIC_OBJECT(d) => {
                let fields: HashMap<String, Object> = HashMap::new();
                let gc_ref = self.environment.heap.alloc(
                    GCRefData::DYNAMIC_OBJECT(DynamicObject { fields }),
                    &self.config,
                );

                if gc_ref.is_err() {
                    return Err(gc_ref.err().unwrap());
                }

                self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                    [call.arg_2 as usize] = Object::GC_REF(gc_ref.unwrap());
                self.environment.stack_frames[self.environment.stack_frame_pointer]
                    .instruction_pointer += 1;
            }
            _ => {
                self.environment.stack_frames[self.environment.stack_frame_pointer]
                    .instruction_pointer += 1;
            }
        }

        // fixme we need a way of tracking the last register used, maybe return does this?
        Ok(0)
    }

    fn exec_native_call(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        let name = self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [instr.arg_0 as usize]
            .clone();
        if let Object::GC_REF(gc_ref) = name {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;

            let name_obj = self.environment.heap.deref(&gc_ref);

            if let GCRefData::STRING(s) = name_obj {
                let ss = s.s.to_string();
                let native_fn = &self.environment.native_fns[&ss];

                let mut args: Vec<Object> = vec![];
                for i in instr.arg_1..instr.arg_1 + instr.arg_2 {
                    args.push(
                        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                            [i as usize]
                            .clone(),
                    );
                }

                let result = native_fn(self.environment, args);

                let destination = {
                    if instr.arg_2 > 0 {
                        instr.arg_1 as usize + instr.arg_2 as usize
                    } else {
                        (instr.arg_1 + 1).into()
                    }
                };

                self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                    [destination] = result.clone();

                return Ok(instr.arg_2 + instr.arg_2);
            }
        }
        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;
        return Err(RuntimeError::INVALID_OPERATION);
    }

    fn exec_new(&mut self, new: &Instruction) -> Result<u8, RuntimeError> {
        // todo allocate on stack
        // for now just GC now
        self.mark_and_sweep();

        // get the type from the stack
        let type_object = &self.environment.stack_frames[self.environment.stack_frame_pointer]
            .stack[new.arg_0 as usize];

        // self.environment.heap.mark_and_sweep();
        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;

        // fixme
        Ok(0)
    }

    fn exec_load_const(&mut self, load_const: &Instruction) -> Result<u8, RuntimeError> {
        let const_obj = &self.environment.stack_frames[self.environment.stack_frame_pointer]
            .fn_object
            .chunk
            .constant_pool[load_const.arg_0 as usize];

        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [load_const.arg_2 as usize] = const_obj.clone();
        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;

        Ok(load_const.arg_2)
    }

    fn exec_if_jmp_false(&mut self, if_jmp_else: &Instruction) -> Result<u8, RuntimeError> {
        let val = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [if_jmp_else.arg_0 as usize];

        if !val.truthy() {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer = if_jmp_else.arg_1 as usize
        } else {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;
        }

        // fixme
        Ok(0)
    }

    fn exec_build_slice(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        let mut slice_objects: Vec<Object> = vec![];
        for i in 0..instr.arg_1 {
            slice_objects.push(
                self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                    [instr.arg_0 as usize + i as usize]
                    .clone(),
            );
        }

        let slice_obj = self.environment.heap.alloc(
            GCRefData::SLICE(SliceObject { s: slice_objects }),
            &self.config,
        );

        if slice_obj.is_err() {
            // fixme correct error
            return Err(RuntimeError::OUT_OF_MEMORY);
        }

        self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [instr.arg_2 as usize] = Object::GC_REF(slice_obj.unwrap());

        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer +=
            1;
        Ok(instr.arg_2)
    }

    fn exec_index(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        // todo for now only slices can be indexed
        let obj_to_index = self.environment.stack_frames[self.environment.stack_frame_pointer]
            .stack[instr.arg_0 as usize]
            .clone();

        if let Object::GC_REF(gc_ref) = obj_to_index {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;

            let obj = self.environment.heap.deref(&gc_ref);
            println!("indexing {:?}", obj);

            if let GCRefData::SLICE(s) = obj {
                // now lets get the index
                let index_obj = &self.environment.stack_frames
                    [self.environment.stack_frame_pointer]
                    .stack[instr.arg_1 as usize];
                if let Object::I64(i) = index_obj {
                    let index_val: i64 = *i;
                    self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                        [instr.arg_2 as usize] = s.s[index_val as usize].clone();

                    return Ok(instr.arg_2);
                }
            }
        }

        Err(RuntimeError::INVALID_OPERATION)
    }

    fn mark_and_sweep(&mut self) {
        // https://ceronman.com/2021/07/22/my-experience-crafting-an-interpreter-with-rust/

        // // todo
        // // 1. mark every object
        // // 2. sweep

        // // lets go through the stack first
        // let current_frame = &self.environment.stack_frames[self.environment.stack_frame_pointer];
        // for obj in current_frame.stack.iter() {
        //     match obj {
        //         _ => continue,
        //         Object::HEAP_OBJECT(heap_object) => {
        //             // lets check if its reachable on the heap
        //             // todo probably have object ids?

        //             if self.environment.heap.objects.is_none() {
        //                 return;
        //             }
        //             let mut next = self.environment.heap.objects.as_ref().unwrap();
        //             while true {
        //                 break;
        //                 // if next == heap_object.data
        //             }
        //         }
        //     }
        // }
    }
}
