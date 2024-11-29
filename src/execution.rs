use std::{collections::HashMap, rc::Rc};

use crate::codegen::{Chunk, Instruction, OpInstruction};

#[derive(Debug)]
pub enum RuntimeError {
    INVALID_OPERATION,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CustomObject {
    pub fields: Vec<Object>,
}

#[derive(Debug, Clone)]
pub struct FnObject {
    pub chunk: Chunk,
}

// todo should this be Rc'd?
#[derive(Debug, Clone)]
pub struct StringObject {
    pub s: Rc<String>,
}

#[derive(Debug, Clone)]
pub enum HeapObjectData {
    FN(FnObject),
    STRING(StringObject),
    DYNAMIC_OBJECT(DynamicObject),
}

#[derive(Debug, Clone)]
pub struct HeapObject {
    pub data: HeapObjectData,
    pub is_marked: bool,
}

impl HeapObject {
    pub fn print(&self) -> String {
        match &self.data {
            HeapObjectData::STRING(s) => s.s.to_string(),
            HeapObjectData::FN(f) => format!("<HeapObject:FnObject at {:p}>", self),
            HeapObjectData::DYNAMIC_OBJECT(d) => d.print(), // HeapObjectData::DYNAMIC_OBJECT(d) => {
                                                            //     format!("<HeapObject:DynamicObject at {:p}>", self)
                                                            // }
        }
    }

    pub fn add(&self, other: Object) -> Result<Object, RuntimeError> {
        Ok(Object::I64(1))
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    F64(f64),
    I64(i64),
    ATOM(Rc<String>),
    HEAP_OBJECT(Box<HeapObject>),
}

impl Object {
    pub fn create_heap_obj(heap_obj_data: HeapObjectData) -> Self {
        Object::HEAP_OBJECT(Box::new(HeapObject {
            data: heap_obj_data,
            is_marked: false,
        }))
    }

    pub fn get_type(&self) -> Object {
        match self {
            Self::I64(_) => {
                Object::create_heap_obj(HeapObjectData::DYNAMIC_OBJECT(DynamicObject::new(
                    HashMap::from([("name".to_string(), Object::ATOM(Rc::new("I64".to_string())))]),
                )))
            }
            _ => panic!(),
        }
    }

    pub fn print(&self) -> std::string::String {
        match self {
            Self::F64(f) => f.to_string(),
            Self::I64(i) => i.to_string(),
            Self::ATOM(a) => format!(":{:?}", a.to_string()),
            Self::HEAP_OBJECT(h) => h.print(),
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
    pub instruction_pointer: usize,
    pub stack: std::vec::Vec<Object>,
    // todo this sucks
    pub fn_object: Box<FnObject>,
}

pub struct Heap {
    // linked list of objects
    pub objects: std::vec::Vec<Box<HeapObject>>,
}

impl Heap {
    pub fn new(&mut self, object: HeapObject) {
        let next = Box::new(object);
        self.objects.push(next);
    }
}

pub struct ExecutionEngine {
    pub running: bool,
    pub stack_frames: std::vec::Vec<StackFrame>,
    pub stack_frame_pointer: usize,
    // todo we need a sort of heap
    pub heap: Heap,
}

impl ExecutionEngine {
    pub fn new() -> ExecutionEngine {
        ExecutionEngine {
            stack_frame_pointer: 0,
            running: true,
            stack_frames: vec![],
            heap: Heap { objects: vec![] },
        }
    }

    pub fn exec(&mut self, bytecode: Chunk) -> Result<Object, RuntimeError> {
        self.init_startup_stack(Box::new(FnObject { chunk: bytecode }));
        self.zero_stack();
        let mut reg = 0;
        while self.running {
            // let current_instruction = self.current_instruction().clone();
            let instr = {
                let current_frame = &self.stack_frames[self.stack_frame_pointer];
                &current_frame.fn_object.chunk.instructions[current_frame.instruction_pointer]
                    .clone()
            };

            let reg_result = self.exec_instr(instr);

            if let Err(e) = reg_result {
                return Err(e);
            }

            if self.stack_frames[self.stack_frame_pointer].instruction_pointer
                == self.stack_frames[self.stack_frame_pointer]
                    .fn_object
                    .chunk
                    .instructions
                    .len()
            {
                self.running = false;
            }
        }

        // todo return reference
        return Ok(self.stack_frames[self.stack_frame_pointer].stack[reg as usize].clone());
    }

    fn exec_instr(&mut self, instr: &Instruction) -> Result<u8, RuntimeError> {
        match instr.op_instruction {
            OpInstruction::RETURN => self.exec_return(instr),
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::SUBI => self.exec_subi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            OpInstruction::CALL => self.exec_call(instr),
            OpInstruction::NEW => self.exec_new(instr),
            OpInstruction::LOAD_CONST => self.exec_load_const(instr),
            OpInstruction::IF_JMP_FALSE => self.exec_if_jmp_false(instr),
            _ => panic!("unknown instruction {:?}", instr.op_instruction),
        }
    }

    fn init_startup_stack(&mut self, fn_object: Box<FnObject>) {
        self.stack_frames.push(StackFrame {
            stack: vec![],
            fn_object: fn_object,
            instruction_pointer: 0,
        });
        self.stack_frame_pointer = 0;
    }

    fn push_stack_frame(&mut self, fn_object: Box<FnObject>) {
        self.stack_frames.push(StackFrame {
            stack: vec![],
            fn_object: fn_object,
            instruction_pointer: 0,
        });
        self.stack_frame_pointer += 1;
    }

    fn zero_stack(&mut self) {
        // setup stack
        for _ in 0..5 {
            self.stack_frames[self.stack_frame_pointer]
                .stack
                .push(Object::I64(0));
        }
    }

    fn exec_return(&mut self, ret: &Instruction) -> Result<u8, RuntimeError> {
        if self.stack_frames.len() == 1 {
            println!("stack: {:#?}", self.stack_frames);
        }

        self.stack_frames.pop();
        if self.stack_frames.len() == 0 {
            self.running = false;
        } else {
            self.stack_frame_pointer -= 1;
            self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
        }

        // fixme
        Ok(0)
    }

    fn exec_addi(&mut self, addi: &Instruction) -> Result<u8, RuntimeError> {
        self.stack_frames[self.stack_frame_pointer].stack[addi.arg_2 as usize] =
            Object::I64((addi.arg_0 + addi.arg_1).into());

        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

        Ok(addi.arg_2)
    }
    fn exec_subi(&mut self, subi: &Instruction) -> Result<u8, RuntimeError> {
        self.stack_frames[self.stack_frame_pointer].stack[subi.arg_2 as usize] =
            Object::I64((subi.arg_0 - subi.arg_1).into());

        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

        Ok(subi.arg_2)
    }

    fn exec_add(&mut self, add: &Instruction) -> Result<u8, RuntimeError> {
        let lhs = &self.stack_frames[self.stack_frame_pointer].stack[add.arg_0 as usize];
        let rhs = &self.stack_frames[self.stack_frame_pointer].stack[add.arg_1 as usize];

        let addition: Result<Object, RuntimeError> = lhs.add(rhs.clone());
        if let Ok(res) = addition {
            self.stack_frames[self.stack_frame_pointer].stack[add.arg_2 as usize] = res;
            self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

            return Ok(add.arg_2);
        } else {
            return Err(addition.err().unwrap());
        }
    }

    fn exec_call(&mut self, call: &Instruction) -> Result<u8, RuntimeError> {
        let fn_object = &self.stack_frames[self.stack_frame_pointer].stack[call.arg_0 as usize];

        let heap_object: &HeapObject = match &fn_object {
            Object::HEAP_OBJECT(h) => h,
            _ => panic!("can only call func"),
        };

        let fn_object = match &heap_object.data {
            HeapObjectData::FN(fnn) => fnn,
            _ => panic!("unknown value"),
        };

        // fixme this sucks, we shouldn't clone functions it's so expensive
        self.push_stack_frame(Box::new(fn_object.clone()));
        self.zero_stack();

        // fixme
        Ok(0)
    }

    fn exec_new(&mut self, new: &Instruction) -> Result<u8, RuntimeError> {
        // todo allocate on stack
        // for now just GC now
        self.mark_and_sweep();

        // get the type from the stack
        let type_object = &self.stack_frames[self.stack_frame_pointer].stack[new.arg_0 as usize];

        // self.heap.mark_and_sweep();
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

        // fixme
        Ok(0)
    }

    fn exec_load_const(&mut self, load_const: &Instruction) -> Result<u8, RuntimeError> {
        let const_obj = &self.stack_frames[self.stack_frame_pointer]
            .fn_object
            .chunk
            .constant_pool[load_const.arg_0 as usize];

        self.stack_frames[self.stack_frame_pointer].stack[load_const.arg_1 as usize] =
            const_obj.clone();
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

        Ok(load_const.arg_1)
    }

    fn exec_if_jmp_false(&mut self, if_jmp_else: &Instruction) -> Result<u8, RuntimeError> {
        let val = &self.stack_frames[self.stack_frame_pointer].stack[if_jmp_else.arg_0 as usize];

        if !val.truthy() {
            self.stack_frames[self.stack_frame_pointer].instruction_pointer =
                if_jmp_else.arg_1 as usize
        } else {
            self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
        }

        // fixme
        Ok(0)
    }

    fn mark_and_sweep(&mut self) {
        // https://ceronman.com/2021/07/22/my-experience-crafting-an-interpreter-with-rust/

        // // todo
        // // 1. mark every object
        // // 2. sweep

        // // lets go through the stack first
        // let current_frame = &self.stack_frames[self.stack_frame_pointer];
        // for obj in current_frame.stack.iter() {
        //     match obj {
        //         _ => continue,
        //         Object::HEAP_OBJECT(heap_object) => {
        //             // lets check if its reachable on the heap
        //             // todo probably have object ids?

        //             if self.heap.objects.is_none() {
        //                 return;
        //             }
        //             let mut next = self.heap.objects.as_ref().unwrap();
        //             while true {
        //                 break;
        //                 // if next == heap_object.data
        //             }
        //         }
        //     }
        // }
    }
}
