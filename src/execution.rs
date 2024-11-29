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
pub struct FnObject {
    pub chunk: Chunk,
}

// todo should this be Rc'd?
#[derive(Debug, Clone)]
pub struct StringObject {
    pub s: Rc<String>,
}

#[derive(Debug, Clone)]
pub enum GCRefData {
    FN(FnObject),
    STRING(StringObject),
    DYNAMIC_OBJECT(DynamicObject),
}

// impl HeapObject {
//     pub fn print(&self) -> String {
//         match &self.data {
//             HeapObjectData::STRING(s) => s.s.to_string(),
//             HeapObjectData::FN(f) => format!("<HeapObject:FnObject at {:p}>", self),
//             HeapObjectData::DYNAMIC_OBJECT(d) => d.print(), // HeapObjectData::DYNAMIC_OBJECT(d) => {
//                                                             //     format!("<HeapObject:DynamicObject at {:p}>", self)
//                                                             // }
//         }
//     }

//     pub fn add(&self, other: Object) -> Result<Object, RuntimeError> {
//         Ok(Object::I64(1))
//     }
// }

#[derive(Debug, Clone)]
pub enum Object {
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
    pub live_slots: Vec<GCRefData>,
    pub dead_objects: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct GCRef {
    pub index: usize,
    pub marked: bool,
}

impl Heap {
    pub fn new(&mut self, gc_ref_dat: GCRefData) -> GCRef {
        // todo for now just push to end
        self.live_slots.push(gc_ref_dat);
        GCRef {
            index: self.live_slots.len() - 1,
            marked: false,
        }
    }

    pub fn deref(&mut self, gc_ref: &GCRef) -> GCRefData {
        return self.live_slots[gc_ref.index].clone();
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
            heap: Heap {
                live_slots: vec![],
                dead_objects: vec![],
            },
        }
    }

    pub fn exec(&mut self, bytecode: Chunk) -> Result<Object, RuntimeError> {
        self.init_startup_stack(Box::new(FnObject { chunk: bytecode }));
        self.zero_stack();
        self.init_constants();
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

            reg = reg_result.unwrap();

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

    fn init_constants(&mut self) {
        // todo
        // for each gc ref data constant on the current chunk, put it in the heap and update the reference

        let stack_frame = &mut self.stack_frames[self.stack_frame_pointer];

        let constant_pool = &mut stack_frame.fn_object.chunk.constant_pool;
        // todo this is HORRIBLE
        let gc_ref_data = &stack_frame.fn_object.chunk.gc_ref_data.clone();

        for i in constant_pool.iter_mut() {
            if let Object::GC_REF(gc_ref) = i {
                let gc_ref_data = &gc_ref_data[gc_ref.index];

                // now lets heap allocate!
                let alloc = self.heap.new(gc_ref_data.clone());
                gc_ref.index = alloc.index;
            }
        }
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
        let gc_ref_object: &GCRef = match &fn_object {
            Object::GC_REF(r) => r,
            _ => panic!("can only call func or constructor"),
        };

        println!(
            "whoa gc ref {:?} {:?}",
            gc_ref_object, self.heap.live_slots[0]
        );

        let dereferenced_data = self.heap.deref(gc_ref_object);

        match &dereferenced_data {
            GCRefData::FN(f) => {
                // fixme this sucks, we shouldn't clone functions it's so expensive
                // fixme why is this a Box?
                self.push_stack_frame(Box::new(f.clone()));
                self.zero_stack();
                self.init_constants();
            }
            GCRefData::DYNAMIC_OBJECT(d) => {
                println!("calling new on {:?}", d);
                self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
            }
            _ => {
                self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
            }
        }

        // match &heap_object.data {
        //     HeapObjectData::FN(fnn) => {
        //         // fixme this sucks, we shouldn't clone functions it's so expensive
        //         self.push_stack_frame(Box::new(fnn.clone()));
        //         self.zero_stack();
        //          self.init_constants();
        //     }
        //     HeapObjectData::DYNAMIC_OBJECT(obj) => {
        //         println!("calling new on Vec! {:?}", obj);

        //         let fields: HashMap<String, Object> = HashMap::new();
        //         // create a new instance of this
        //         let new_object = Object::HEAP_OBJECT(Box::new(HeapObject {
        //             data: HeapObjectData::DYNAMIC_OBJECT(DynamicObject { fields }),
        //             is_marked: false,
        //         }));

        //         // todo put object on the heap

        //         self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
        //     }
        //     _ => panic!(),
        // }

        // fixme we need a way of tracking the last register used, maybe return does this?
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

        self.stack_frames[self.stack_frame_pointer].stack[load_const.arg_2 as usize] =
            const_obj.clone();
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;

        Ok(load_const.arg_2)
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
