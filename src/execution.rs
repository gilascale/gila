use crate::compiler::Compiler;
use crate::lex::Type;
use deepsize::DeepSizeOf;
use std::hash::Hash;
use std::{collections::HashMap, fmt::format, fs::File, rc::Rc};
use std::{fs, vec};

// todo deal with multi-platform
// use std::os::windows::io::AsRawHandle;

use crate::{
    codegen::{Chunk, Instruction, OpInstruction},
    config::Config,
};

macro_rules! stack_access {
    ($self:expr, $arg:expr) => {
        &$self.environment.stack_frames[$self.environment.stack_frame_pointer].stack[$arg as usize]
    };
}

macro_rules! stack_set {
    ($self:expr, $index:expr, $value:expr) => {
        $self.environment.stack_frames[$self.environment.stack_frame_pointer].stack
            [$index as usize] = $value;
    };
}

macro_rules! increment_ip {
    ($self:expr) => {
        $self.environment.stack_frames[$self.environment.stack_frame_pointer]
            .instruction_pointer += 1;
    };
}

#[derive(Debug)]
pub enum RuntimeError {
    INVALID_OPERATION,
    INVALID_GC_REF,
    INVALID_ACCESS,
    OUT_OF_MEMORY,
    UNKNOWN_MODULE,
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
    // if this function needs to be bound at runtime
    pub requires_method_binding: bool,
    // the slot for the local variable it needs to bind to
    pub method_to_object: Option<u8>,
    pub param_slots: Vec<u8>,
    // todo maybe make a BoundedFn object?
    pub bounded_object: Option<GCRef>,
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
    TUPLE(Vec<Object>),
    FN(FnObject),
    STRING(StringObject),
    SLICE(SliceObject),
    DYNAMIC_OBJECT(DynamicObject),
}

impl GCRefData {
    pub fn print(&self, shared_execution_context: &mut SharedExecutionContext) -> String {
        match self {
            Self::STRING(s) => s.s.to_string(),
            Self::FN(fn_object) => format!("fn {}", fn_object.name),
            Self::SLICE(slice) => {
                format!(
                    "[{}]",
                    slice
                        .s
                        .iter()
                        .map(|item| item.print(shared_execution_context))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Self::DYNAMIC_OBJECT(d) => {
                format!(
                    "{{{}}}",
                    d.fields
                        .iter()
                        .map(|(key, value)| format!(
                            "{}={}",
                            key,
                            value.print(shared_execution_context)
                        ))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            _ => panic!("Cant print self {:?}", self),
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

    pub fn print(
        &self,
        shared_execution_context: &mut SharedExecutionContext,
    ) -> std::string::String {
        match self {
            Self::BOOL(b) => b.to_string(),
            Self::F64(f) => f.to_string(),
            Self::I64(i) => i.to_string(),
            Self::ATOM(a) => format!(":{:?}", a.to_string()),
            Self::GC_REF(gc_ref) => {
                let res = shared_execution_context.heap.deref(&gc_ref);
                let obj: String;
                if res.is_ok() {
                    obj = res.unwrap().print(shared_execution_context);
                } else {
                    shared_execution_context.heap.dump_heap();
                    panic!("tried to deref {}", gc_ref.index);
                }
                obj
            } // Self::HEAP_OBJECT(h) => h.print(),
        }
    }

    pub fn add(
        &self,
        shared_execution_context: &mut SharedExecutionContext,
        config: &Config,
        other: Object,
    ) -> Result<Object, RuntimeError> {
        match self {
            Self::GC_REF(gc_ref) => {
                let res = shared_execution_context.heap.deref(gc_ref);
                if res.is_err() {
                    return Err(res.err().unwrap());
                }

                match res.unwrap() {
                    GCRefData::STRING(s) => {
                        let mut dup = s.s.to_string();
                        dup.push_str(&other.print(shared_execution_context));
                        let new_str = GCRefData::STRING(StringObject { s: Rc::new(dup) });
                        let new_obj = shared_execution_context.heap.alloc(new_str, config);
                        if new_obj.is_err() {
                            return Err(new_obj.err().unwrap());
                        }
                        return Ok(Object::GC_REF(new_obj.unwrap()));
                    }
                    _ => todo!(),
                }
            }
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

    pub fn equals(
        &self,
        shared_execution_context: &mut SharedExecutionContext,
        other: Object,
    ) -> Result<bool, RuntimeError> {
        match self {
            Self::BOOL(b1) => match other {
                Self::BOOL(b2) => return Ok(*b1 == b2),
                _ => return Ok(false),
            },
            Self::ATOM(a1) => match other {
                Object::ATOM(a2) => return Ok(a1.eq(&a2)),
                _ => return Ok(false),
            },
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 == i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            Self::GC_REF(gc_ref) => {
                let res = shared_execution_context.heap.deref(gc_ref);
                if res.is_err() {
                    return Err(res.err().unwrap());
                }
                match res.unwrap() {
                    GCRefData::STRING(s) => match other {
                        Object::GC_REF(other_gc_ref) => {
                            let other_res = shared_execution_context.heap.deref(&other_gc_ref);
                            if other_res.is_err() {
                                return Err(other_res.err().unwrap());
                            }
                            match other_res.unwrap() {
                                GCRefData::STRING(other_s) => return Ok(s.s == other_s.s),
                                _ => return Ok(false),
                            }
                        }
                        _ => return Ok(false),
                    },
                    _ => todo!(),
                }
                Ok(true)
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => todo!(),
        }
    }

    pub fn not_equals(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 != i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn greater_than(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 > i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn greater_than_equals(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 >= i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn less_than(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 < i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn less_than_equals(&self, other: Object) -> Result<bool, RuntimeError> {
        match self {
            Self::I64(i1) => {
                // integer addition
                match other {
                    Object::I64(i2) => return Ok(*i1 <= i2),
                    _ => return Err(RuntimeError::INVALID_OPERATION),
                }
            }
            // Self::HEAP_OBJECT(h1) => h1.data.add(other),
            _ => return Err(RuntimeError::INVALID_OPERATION),
        }
    }

    pub fn truthy(
        &self,
        shared_execution_context: &mut SharedExecutionContext,
        execution_context: &ProcessContext,
    ) -> bool {
        match self {
            Self::BOOL(b) => return *b,
            Self::F64(f) => return f > &0.0,
            Self::I64(i) => return i > &0,
            Self::GC_REF(i) => {
                let res = shared_execution_context.heap.deref(i);
                if res.is_ok() {
                    // todo this may not be the best because if its an actual error then we need to error!
                    return true;
                }
                return false;
            }
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
        let index = self.live_slots.len();
        self.live_slots.push(gc_ref_dat);
        Ok(GCRef {
            index,
            marked: false,
        })
    }

    pub fn deref(&self, gc_ref: &GCRef) -> Result<GCRefData, RuntimeError> {
        if gc_ref.index >= self.live_slots.len() {
            self.dump_heap();
            return Err(RuntimeError::INVALID_GC_REF);
        }
        return Ok(self.live_slots[gc_ref.index].clone());
    }

    pub fn set(&mut self, gc_ref: &GCRef, value: GCRefData) -> Result<(), RuntimeError> {
        self.live_slots[gc_ref.index] = value;
        Ok(())
    }

    pub fn dump_heap(&self) {
        println!(
            "HEAP (len={}): {:#?}",
            self.live_slots.len(),
            self.live_slots
        );
    }

    pub fn free_space_available_bytes(&self) -> usize {
        self.live_slots.deep_size_of()
    }
}

type NativeFn = fn(&mut SharedExecutionContext, &mut ProcessContext, Vec<Object>) -> Object;

pub struct ProcessContext {
    pub stack_frames: std::vec::Vec<StackFrame>,
    pub stack_frame_pointer: usize,
    pub native_fns: HashMap<String, NativeFn>,
}

pub struct SharedExecutionContext {
    pub heap: Heap,
}

impl ProcessContext {
    fn dump_stack_regs(&mut self) {
        for i in 0..self.stack_frame_pointer {
            println!("frame {} ({}):", i, self.stack_frames[i].fn_object.name);
            println!("  {:#?}", self.stack_frames[i].stack);
        }
    }
}

fn native_print(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    let s: String = match &args[0] {
        Object::GC_REF(gc_ref) => {
            let res = shared_execution_context.heap.deref(&gc_ref);
            let obj: String;
            if res.is_ok() {
                obj = res.unwrap().print(shared_execution_context);
            } else {
                execution_context.dump_stack_regs();
                shared_execution_context.heap.dump_heap();
                panic!("tried to deref {}", gc_ref.index);
            }
            obj
        }
        Object::I64(i) => i.to_string(),
        Object::F64(f) => f.to_string(),
        Object::BOOL(b) => b.to_string(),
        Object::ATOM(a) => a.to_string(),
    };

    println!("{}", s);
    return Object::I64(0);
}

fn native_open_windows(
    shared_execution_context: &mut SharedExecutionContext,
    execution_context: &mut ProcessContext,
    args: Vec<Object>,
) -> Object {
    // if let Object::GC_REF(gc_ref) = &args[0] {
    //     if let GCRefData::STRING(s) = execution_context.heap.deref(&gc_ref) {
    //         let file = File::open(s.s.to_string());
    //         if let Ok(file) = file {
    //             let handle = file.as_raw_handle();
    //             return Object::I64(handle as i64);
    //         }
    //     }
    // }

    return Object::I64(0);
}

pub struct ExecutionEngine<'a> {
    pub config: &'a Config,
    pub running: bool,
    pub shared_execution_context: &'a mut SharedExecutionContext,
    pub environment: &'a mut ProcessContext,
}

impl<'a> ExecutionEngine<'a> {
    pub fn new(
        config: &'a Config,
        shared_execution_context: &'a mut SharedExecutionContext,
        environment: &'a mut ProcessContext,
    ) -> ExecutionEngine<'a> {
        ExecutionEngine {
            config,
            running: true,
            shared_execution_context,
            environment,
        }
    }

    pub fn register_native_fn(&mut self, name: String, native_fn: NativeFn) {
        self.environment.native_fns.insert(name, native_fn);
    }

    pub fn exec(
        &mut self,
        compilation_unit: String,
        bytecode: Chunk,
        is_repl: bool,
    ) -> Result<Object, RuntimeError> {
        self.register_native_fn("native_print".to_string(), native_print);
        self.register_native_fn("native_open_windows".to_string(), native_open_windows);

        self.running = true;

        // todo think how do do this with preludes...
        // if is_repl {
        if self.environment.stack_frames.len() == 0 {
            self.init_startup_stack(Box::new(FnObject {
                chunk: bytecode,
                name: "main".to_string(),
                requires_method_binding: false,
                method_to_object: None,
                param_slots: vec![],
                bounded_object: None,
            }));
            self.zero_stack();
            self.init_constants();
        } else {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .fn_object
                .chunk = bytecode;
            // todo is this right?
            // fixme this only works for repl and prelude etc, it doesn't however work when importing a module, because we
            // have code after that we need to run.
            // i think this may actually be okay, because on a module import we just need to take the top level exports!
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer = 0;
            self.init_constants();
        }

        // println!("{:#?}", self.environment.stack_frames);

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

    // todo nested gc refs!
    fn init_constants(&mut self) -> Result<(), RuntimeError> {
        // todo
        // for each gc ref data constant on the current chunk, put it in the heap and update the reference

        let stack_frame = &mut self.environment.stack_frames[self.environment.stack_frame_pointer];

        let constant_pool = &mut stack_frame.fn_object.chunk.constant_pool;
        // todo this is HORRIBLE
        let stack_gc_ref_data = &mut stack_frame.fn_object.chunk.gc_ref_data.clone();

        for i in constant_pool.iter_mut() {
            if let Object::GC_REF(gc_ref) = i {
                let mut gc_ref_data = &stack_gc_ref_data[gc_ref.index];

                match gc_ref_data {
                    GCRefData::TUPLE(t) => {
                        let mut new_vec: Vec<Object> = vec![];
                        for item in t {
                            // todo do the exact same thing here, check if its a Object::GC_REF and alloc it
                            if let Object::GC_REF(nested_gc_ref) = item {
                                let nested_gc_ref_data = &stack_gc_ref_data[nested_gc_ref.index];

                                // now lets heap allocate!
                                let alloc = self
                                    .shared_execution_context
                                    .heap
                                    .alloc(nested_gc_ref_data.clone(), &self.config);
                                match alloc {
                                    Ok(_) => {
                                        new_vec.push(Object::GC_REF(GCRef {
                                            index: alloc.unwrap().index,
                                            marked: false,
                                        }));
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                        }
                        let new_tuple = &GCRefData::TUPLE(new_vec);
                        // now lets heap allocate!
                        let alloc = self
                            .shared_execution_context
                            .heap
                            .alloc(new_tuple.clone(), &self.config);
                        match alloc {
                            Ok(_) => gc_ref.index = alloc.unwrap().index,
                            Err(e) => return Err(e),
                        }
                    }
                    _ => {
                        // now lets heap allocate!
                        let alloc = self
                            .shared_execution_context
                            .heap
                            .alloc(gc_ref_data.clone(), &self.config);
                        match alloc {
                            Ok(_) => gc_ref.index = alloc.unwrap().index,
                            Err(e) => return Err(e),
                        }
                    }
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
            OpInstruction::GREATER_THAN => self.exec_greater(instr),
            OpInstruction::GREATER_EQUAL => self.exec_greater_equals(instr),
            OpInstruction::LESS_THAN => self.exec_less_than(instr),
            OpInstruction::LESS_EQUAL => self.exec_less_equals(instr),
            OpInstruction::LOGICAL_OR => self.exec_logical_or(instr),
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::SUBI => self.exec_subi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            OpInstruction::CALL => self.exec_call(instr),
            OpInstruction::CALL_KW => self.exec_call_kw(instr),
            OpInstruction::NATIVE_CALL => self.exec_native_call(instr),
            OpInstruction::NEW => self.exec_new(instr),
            OpInstruction::LOAD_CONST => self.exec_load_const(instr),
            OpInstruction::IF_JMP_FALSE => self.exec_if_jmp_false(instr),
            OpInstruction::JMP => self.exec_jmp(instr),
            OpInstruction::BUILD_SLICE => self.exec_build_slice(instr),
            OpInstruction::BUILD_FN => self.exec_build_fn(instr),
            OpInstruction::INDEX => self.exec_index(instr),
            OpInstruction::LOAD_CLOSURE => self.exec_load_closure(instr),
            OpInstruction::STRUCT_ACCESS => self.exec_struct_access(instr),
            OpInstruction::STRUCT_SET => self.exec_struct_set(instr),
            OpInstruction::IMPORT => self.exec_import(instr),
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
        for _ in 0..100 {
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
        let lhs = stack_access!(self, equal.arg_0);
        let rhs = stack_access!(self, equal.arg_1);

        let result = lhs.equals(&mut self.shared_execution_context, rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, equal.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);
        Ok(equal.arg_2)
    }

    fn exec_nequal(&mut self, not_equal: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, not_equal.arg_0);
        let rhs = stack_access!(self, not_equal.arg_1);

        let result = lhs.not_equals(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, not_equal.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);
        Ok(not_equal.arg_2)
    }

    fn exec_greater(&mut self, greater: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, greater.arg_0);
        let rhs = stack_access!(self, greater.arg_1);

        let result = lhs.greater_than(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, greater.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);

        Ok(greater.arg_2)
    }

    fn exec_greater_equals(&mut self, greater: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, greater.arg_0);
        let rhs = stack_access!(self, greater.arg_1);

        let result = lhs.greater_than_equals(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, greater.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);

        Ok(greater.arg_2)
    }

    fn exec_less_than(&mut self, greater: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, greater.arg_0);
        let rhs = stack_access!(self, greater.arg_1);

        let result = lhs.less_than(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, greater.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);

        Ok(greater.arg_2)
    }

    fn exec_less_equals(&mut self, greater: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, greater.arg_0);
        let rhs = stack_access!(self, greater.arg_1);

        let result = lhs.less_than_equals(rhs.clone());
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        stack_set!(self, greater.arg_2, Object::BOOL(result.unwrap()));
        increment_ip!(self);

        Ok(greater.arg_2)
    }

    fn exec_logical_or(&mut self, greater: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = stack_access!(self, greater.arg_0);
        let rhs = stack_access!(self, greater.arg_1);

        let result = lhs.truthy(&mut self.shared_execution_context, &self.environment)
            || rhs.truthy(&mut self.shared_execution_context, &self.environment);
        stack_set!(self, greater.arg_2, Object::BOOL(result));
        increment_ip!(self);

        Ok(greater.arg_2)
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

        let addition: Result<Object, RuntimeError> = lhs.add(
            &mut self.shared_execution_context,
            &self.config,
            rhs.clone(),
        );
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

    fn exec_call_kw(&mut self, call: &Instruction) -> Result<u8, RuntimeError> {
        let fn_object = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [call.arg_0 as usize];
        let gc_ref_object: &GCRef = match &fn_object {
            Object::GC_REF(r) => r,
            _ => panic!("can only call func or constructor"),
        };
        let dereferenced_data = self.shared_execution_context.heap.deref(gc_ref_object);
        if dereferenced_data.is_err() {
            self.environment.dump_stack_regs();
            return Err(dereferenced_data.err().unwrap());
        }

        let mut kwarg_strings: Vec<String> = vec![];
        match &dereferenced_data.unwrap() {
            GCRefData::DYNAMIC_OBJECT(d) => {
                let kwargs_tuple = stack_access!(self, call.arg_1);

                match kwargs_tuple {
                    Object::GC_REF(kwargs_gc_ref) => {
                        let res = self.shared_execution_context.heap.deref(kwargs_gc_ref);
                        if res.is_err() {
                            return Err(res.err().unwrap());
                        }
                        match res.unwrap() {
                            GCRefData::TUPLE(t) => {
                                for item in t {
                                    match item {
                                        Object::GC_REF(item_gc_ref) => {
                                            let res = self
                                                .shared_execution_context
                                                .heap
                                                .deref(&item_gc_ref);
                                            if res.is_err() {
                                                return Err(res.err().unwrap());
                                            }
                                            match res.unwrap() {
                                                GCRefData::STRING(s) => {
                                                    kwarg_strings.push(s.s.to_string());
                                                }
                                                _ => panic!(),
                                            }
                                        }
                                        _ => panic!(),
                                    }
                                }
                            }
                            _ => panic!(),
                        }
                    }
                    _ => panic!(),
                }

                // println!("{:?}", kwarg_strings);

                let mut arg_values: Vec<Object> = vec![];
                for i in call.arg_2..call.arg_2 + kwarg_strings.len() as u8 {
                    arg_values.push(stack_access!(self, i).clone());
                }

                // println!("{:?}", arg_values);

                let destination = call.arg_2 + kwarg_strings.len() as u8;

                let mut fields: HashMap<String, Object> = HashMap::new();

                fields.insert("__prototype__".to_string(), fn_object.clone());

                // todo actually typecheck
                let mut i = 0;
                for kwarg in kwarg_strings {
                    fields.insert(kwarg, arg_values[i].clone());
                    i += 1;
                }

                let gc_ref = self.shared_execution_context.heap.alloc(
                    GCRefData::DYNAMIC_OBJECT(DynamicObject { fields }),
                    &self.config,
                );

                if gc_ref.is_err() {
                    return Err(gc_ref.err().unwrap());
                }

                let gc_ref_res = gc_ref.unwrap();

                stack_set!(self, destination, Object::GC_REF(gc_ref_res.clone()));
                increment_ip!(self);
            }
            _ => {
                panic!()
                // self.environment.stack_frames[self.environment.stack_frame_pointer]
                //     .instruction_pointer += 1;
            }
        }

        Ok(0)
    }

    fn exec_call(&mut self, call: &Instruction) -> Result<u8, RuntimeError> {
        let fn_object = &self.environment.stack_frames[self.environment.stack_frame_pointer].stack
            [call.arg_0 as usize];
        let gc_ref_object: &GCRef = match &fn_object {
            Object::GC_REF(r) => r,
            _ => panic!("can only call func or constructor"),
        };
        let dereferenced_data = self.shared_execution_context.heap.deref(gc_ref_object);
        if dereferenced_data.is_err() {
            self.environment.dump_stack_regs();
            return Err(dereferenced_data.err().unwrap());
        }

        match &dereferenced_data.unwrap() {
            GCRefData::FN(f) => {
                let destination = {
                    if call.arg_2 > 0 {
                        call.arg_1 + call.arg_2
                    } else {
                        call.arg_1.into()
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

                let mut start = 0;
                if f.bounded_object.is_some() {
                    start = 1;
                    self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                        [f.param_slots[0] as usize] =
                        Object::GC_REF(f.bounded_object.clone().unwrap());
                }

                for i in start..num_args {
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
                // todo we need to set the fields

                // let destination = {
                //     if call.arg_2 > 0 {
                //         call.arg_1 + call.arg_2
                //     } else {
                //         call.arg_1.into()
                //     }
                // };

                // let mut fields: HashMap<String, Object> = HashMap::new();

                // fields.insert("__prototype__".to_string(), fn_object.clone());

                // let mut arg_values: Vec<Object> = vec![];
                // for i in call.arg_1..call.arg_1 + call.arg_2 {
                //     arg_values.push(stack_access!(self, i).clone());
                // }
                // let mut counter = 0;
                // for key in d.fields.keys() {
                //     let typ = d.fields.get(key).unwrap();
                //     // we want to only pass in args for members, not methods
                //     match typ {
                //         Object::ATOM(a) => {
                //             fields.insert(key.to_string(), arg_values[counter].clone());
                //             counter += 1;
                //         }
                //         _ => {
                //             // dont add methods
                //             // fields.insert(key.to_string(), typ.clone());
                //         }
                //     }
                // }

                // let gc_ref = self.shared_execution_context.heap.alloc(
                //     GCRefData::DYNAMIC_OBJECT(DynamicObject { fields }),
                //     &self.config,
                // );

                // if gc_ref.is_err() {
                //     return Err(gc_ref.err().unwrap());
                // }

                // stack_set!(self, destination, Object::GC_REF(gc_ref.unwrap()));
                // increment_ip!(self);

                //
                // self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                //     [call.arg_2 as usize] = Object::GC_REF(gc_ref.unwrap());
                // self.environment.stack_frames[self.environment.stack_frame_pointer]
                //     .instruction_pointer += 1;
            }
            _ => {
                increment_ip!(self);
                // self.environment.stack_frames[self.environment.stack_frame_pointer]
                //     .instruction_pointer += 1;
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

            let name_obj = self.shared_execution_context.heap.deref(&gc_ref);
            if name_obj.is_err() {
                self.environment.dump_stack_regs();
                return Err(name_obj.err().unwrap());
            }

            if let GCRefData::STRING(s) = name_obj.unwrap() {
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

                let result = native_fn(self.shared_execution_context, self.environment, args);

                let destination = {
                    if instr.arg_2 > 0 {
                        instr.arg_1 + instr.arg_2
                    } else {
                        instr.arg_1.into()
                    }
                };

                stack_set!(self, destination, result.clone());
                // self.environment.stack_frames[self.environment.stack_frame_pointer].stack
                //     [destination as usize] = result.clone();

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

        if !val.truthy(self.shared_execution_context, &self.environment) {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer = if_jmp_else.arg_1 as usize
        } else {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;
        }

        // fixme
        Ok(0)
    }
    fn exec_jmp(&mut self, jmp: &Instruction) -> Result<u8, RuntimeError> {
        self.environment.stack_frames[self.environment.stack_frame_pointer].instruction_pointer =
            jmp.arg_0 as usize;

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

        let slice_obj = self.shared_execution_context.heap.alloc(
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

    fn exec_build_fn(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        // todo this is breaking it :(
        let fn_ref = stack_access!(self, instr.arg_0);
        if let Object::GC_REF(gc_ref) = fn_ref {
            let fn_obj = self.shared_execution_context.heap.deref(gc_ref);
            if fn_obj.is_err() {
                return Err(fn_obj.err().unwrap());
            }
            let fn_object_data = fn_obj.unwrap();
            if let GCRefData::FN(f) = fn_object_data.clone() {
                if f.requires_method_binding {
                    let obj_to_bind_to = stack_access!(self, f.method_to_object.unwrap());

                    if let Object::GC_REF(g) = obj_to_bind_to {
                        let obj = self.shared_execution_context.heap.deref(g);
                        if obj.is_err() {
                            return Err(obj.err().unwrap());
                        }
                        if let GCRefData::DYNAMIC_OBJECT(o) = obj.unwrap() {
                            // // bind the function to the object here
                            // let mut bounded_fn = f.clone();
                            // bounded_fn.bounded_object = Some(g.clone());
                            // // set the function bounded ref
                            // self.shared_execution_context
                            //     .heap
                            //     .set(gc_ref, GCRefData::FN(bounded_fn));

                            // update the object in the heap
                            let mut cloned_obj = o.clone();
                            cloned_obj.fields.insert(f.name, fn_ref.clone());

                            let res = self
                                .shared_execution_context
                                .heap
                                .set(g, GCRefData::DYNAMIC_OBJECT(cloned_obj.clone()));
                            if res.is_err() {
                                return Err(res.err().unwrap());
                            }
                        }
                    }
                }
            }
        }

        increment_ip!(self);
        Ok(0)
    }

    fn exec_index(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        // todo for now only slices can be indexed
        let obj_to_index = self.environment.stack_frames[self.environment.stack_frame_pointer]
            .stack[instr.arg_0 as usize]
            .clone();

        if let Object::GC_REF(gc_ref) = obj_to_index {
            self.environment.stack_frames[self.environment.stack_frame_pointer]
                .instruction_pointer += 1;

            let obj = self.shared_execution_context.heap.deref(&gc_ref);
            if obj.is_err() {
                self.environment.dump_stack_regs();
                return Err(obj.err().unwrap());
            }

            if let GCRefData::SLICE(s) = obj.unwrap() {
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

    fn exec_load_closure(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        let val = &self.environment.stack_frames[instr.arg_0 as usize].stack[instr.arg_1 as usize];
        stack_set!(self, instr.arg_2, val.clone());
        increment_ip!(self);
        Ok(0)
    }

    fn exec_struct_access(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        let obj = stack_access!(self, instr.arg_0);
        // fixme this is horrible nesting
        match obj {
            Object::GC_REF(obj_gc_ref) => {
                let result = self.shared_execution_context.heap.deref(obj_gc_ref);
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                match result.unwrap() {
                    GCRefData::DYNAMIC_OBJECT(o) => {
                        // now we have the object we need to get the string

                        let field = stack_access!(self, instr.arg_1);

                        match field {
                            Object::GC_REF(gc_ref) => {
                                let result = self.shared_execution_context.heap.deref(gc_ref);
                                if result.is_err() {
                                    return Err(result.err().unwrap());
                                }
                                match result.unwrap() {
                                    GCRefData::STRING(s) => {
                                        let mut next_prototype_in_chain = o.clone();
                                        let mut result: &Object;
                                        let mut found = false;
                                        loop {
                                            let res = next_prototype_in_chain
                                                .fields
                                                .get(&s.s.to_string());
                                            if res.is_some() {
                                                found = true;
                                                result = res.unwrap();
                                                break;
                                            }
                                            let prototype =
                                                next_prototype_in_chain.fields.get("__prototype__");
                                            if prototype.is_none() {
                                                return Err(RuntimeError::INVALID_ACCESS);
                                            }
                                            match prototype.unwrap() {
                                                Object::GC_REF(g) => {
                                                    let deref =
                                                        self.shared_execution_context.heap.deref(g);
                                                    if deref.is_err() {
                                                        return Err(deref.err().unwrap());
                                                    }
                                                    match deref.unwrap() {
                                                        GCRefData::DYNAMIC_OBJECT(d) => {
                                                            next_prototype_in_chain = d
                                                        }
                                                        _ => panic!(),
                                                    }
                                                }
                                                _ => panic!(),
                                            }
                                        }
                                        // go up the chain!

                                        if !found {
                                            return Err(RuntimeError::INVALID_ACCESS);
                                        }

                                        // BINDING HAPPENS HERE
                                        match result {
                                            Object::GC_REF(method_to_bind_gc_ref) => {
                                                let deref = self
                                                    .shared_execution_context
                                                    .heap
                                                    .deref(method_to_bind_gc_ref);
                                                if deref.is_err() {
                                                    return Err(deref.err().unwrap());
                                                }
                                                match deref.unwrap() {
                                                    GCRefData::FN(mut f) => {
                                                        if f.requires_method_binding {
                                                            f.bounded_object =
                                                                Some(obj_gc_ref.clone());

                                                            // todo we should probably not set the actual object? and instead return another?
                                                            // because now its forever bound?
                                                            let res = self
                                                                .shared_execution_context
                                                                .heap
                                                                .set(
                                                                    method_to_bind_gc_ref,
                                                                    GCRefData::FN(f.clone()),
                                                                );
                                                            if res.is_err() {
                                                                return Err(res.err().unwrap());
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            _ => {}
                                        }

                                        stack_set!(self, instr.arg_2, result.clone());
                                        increment_ip!(self);
                                    }
                                    _ => return Err(RuntimeError::INVALID_ACCESS),
                                }
                            }
                            _ => return Err(RuntimeError::INVALID_ACCESS),
                        }
                    }
                    _ => return Err(RuntimeError::INVALID_ACCESS),
                }
            }
            _ => return Err(RuntimeError::INVALID_ACCESS),
        }

        Ok(0)
    }

    fn exec_struct_set(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        let obj = stack_access!(self, instr.arg_0);
        let member = stack_access!(self, instr.arg_1);
        let value_to_set = stack_access!(self, instr.arg_2);

        match obj {
            Object::GC_REF(gc_ref) => {
                let val = self.shared_execution_context.heap.deref(gc_ref);
                if val.is_err() {
                    return Err(val.err().unwrap());
                }
                match val.unwrap() {
                    GCRefData::DYNAMIC_OBJECT(d) => match member {
                        Object::GC_REF(member_gc_ref) => {
                            let member_val =
                                self.shared_execution_context.heap.deref(member_gc_ref);
                            if member_val.is_err() {
                                return Err(member_val.err().unwrap());
                            }
                            match member_val.unwrap() {
                                GCRefData::STRING(s) => {
                                    let mut cloned_dynamic_obj = d.clone();
                                    cloned_dynamic_obj
                                        .fields
                                        .insert(s.s.to_string(), value_to_set.clone());

                                    let res = self
                                        .shared_execution_context
                                        .heap
                                        .set(gc_ref, GCRefData::DYNAMIC_OBJECT(cloned_dynamic_obj));
                                    if res.is_err() {
                                        return Err(res.err().unwrap());
                                    }
                                    // todo add a number here?
                                    increment_ip!(self);
                                    return Ok(0);
                                }
                                _ => todo!(),
                            }
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            _ => todo!(),
        }
    }

    fn exec_import(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        // todo
        let import_path = stack_access!(self, instr.arg_0);
        match import_path {
            Object::GC_REF(gc_ref) => {
                let data = self.shared_execution_context.heap.deref(gc_ref);
                if data.is_err() {
                    return Err(data.err().unwrap());
                }
                match data.unwrap() {
                    GCRefData::STRING(s) => {
                        // paths are the areas that we can find the module
                        let paths = vec!["./".to_string()];
                        for path in paths {
                            let full_path = path + &s.s.to_string();
                            if fs::metadata(full_path.to_string())
                                .map(|m| m.is_dir())
                                .unwrap_or(false)
                            {
                                // todo compile!

                                let mut module_objects: HashMap<String, Object> = HashMap::new();

                                for file in fs::read_dir(full_path).unwrap() {
                                    let f = file.unwrap();

                                    let module_name =
                                        f.file_name().to_string_lossy().replace(".gila", "");

                                    let normalized_path =
                                        f.path().to_string_lossy().replace("\\", "/");
                                    let code = fs::read_to_string(normalized_path.to_string())
                                        .expect("Unable to read file");

                                    // todo this is breaking
                                    let mut compiler = Compiler::new();
                                    let compilation_context = compiler.compile_and_exec(
                                        f.file_name().into_string().unwrap(),
                                        code,
                                        &self.config,
                                        &mut self.shared_execution_context,
                                    );

                                    // todo go through all exported variables

                                    if let Some(ctx) = compilation_context {
                                        let mut exported: HashMap<String, Object> = HashMap::new();
                                        for (key, val) in
                                            ctx.codegen_context.chunks[0].variable_map.clone()
                                        {
                                            if let Type::IDENTIFIER(i) = key {
                                                // lets put the variables in this module
                                                let val = ctx.process_context.stack_frames[0].stack
                                                    [val as usize]
                                                    .clone();
                                                // println!("exported val {:?}={:?}", key, val);
                                                exported.insert(i.to_string(), val);
                                                continue;
                                            }
                                            panic!();
                                        }
                                        let module_dynamic_object =
                                            DynamicObject { fields: exported };
                                        let module = self.shared_execution_context.heap.alloc(
                                            GCRefData::DYNAMIC_OBJECT(module_dynamic_object),
                                            &self.config,
                                        );
                                        if module.is_err() {
                                            return Err(module.err().unwrap());
                                        }
                                        module_objects
                                            .insert(module_name, Object::GC_REF(module.unwrap()));
                                    }
                                }

                                let module_dynamic_object = DynamicObject {
                                    fields: module_objects,
                                };
                                let module = self.shared_execution_context.heap.alloc(
                                    GCRefData::DYNAMIC_OBJECT(module_dynamic_object),
                                    &self.config,
                                );

                                if module.is_err() {
                                    return Err(module.err().unwrap());
                                }

                                stack_set!(self, instr.arg_1, Object::GC_REF(module.unwrap()));
                                increment_ip!(self);
                                return Ok(instr.arg_1);
                            }
                        }
                        return Err(RuntimeError::UNKNOWN_MODULE);
                    }
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
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
