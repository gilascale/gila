use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use crate::codegen::{Chunk, Instruction, OpInstruction};

#[derive(Debug, Clone)]
pub struct FnObject {
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct StringObject {
    pub s: std::string::String,
}

#[derive(Debug)]
pub enum HeapObjectData {
    FN(FnObject),
    STRING(StringObject),
}

#[derive(Debug)]
pub struct HeapObject {
    pub data: HeapObjectData,
    pub is_marked: bool,
}

#[derive(Debug)]
pub enum Object {
    F64(f64),
    I64(i64),
    HEAP_OBJECT(Box<HeapObject>),
}

#[derive(Debug)]
pub struct StackFrame {
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
    pub instruction_pointer: usize,
    pub running: bool,
    pub stack_frames: std::vec::Vec<StackFrame>,
    pub stack_frame_pointer: usize,
    // todo we need a sort of heap
    pub heap: Heap,
}

impl ExecutionEngine {
    pub fn exec(&mut self, bytecode: Chunk) -> Object {
        self.push_stack_frame(Box::new(FnObject { chunk: bytecode }));
        while self.running {
            // let current_instruction = self.current_instruction().clone();
            let instr = {
                let current_frame = &self.stack_frames[self.stack_frame_pointer];
                &current_frame.fn_object.chunk.instructions[self.instruction_pointer].clone()
            };

            self.exec_instr(instr);
            self.instruction_pointer += 1;
        }

        println!("stack: {:#?}", self.stack_frames[self.stack_frame_pointer]);
        return Object::I64(0);
    }

    fn exec_instr(&mut self, instr: &Instruction) {
        match instr.op_instruction {
            OpInstruction::RETURN => self.running = false,
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            OpInstruction::CALL => self.exec_call(instr),
            OpInstruction::NEW => self.exec_new(instr),
            _ => panic!("unknown instruction {:?}", instr.op_instruction),
        }
    }

    fn push_stack_frame(&mut self, fn_object: Box<FnObject>) {
        self.stack_frames.push(StackFrame {
            stack: vec![],
            fn_object: fn_object,
        });
        // setup stack
        for _ in 0..5 {
            self.stack_frames[self.stack_frame_pointer]
                .stack
                .push(Object::I64(0));
        }
    }

    fn exec_addi(&mut self, addi: &Instruction) {
        // self.stack[addi.arg_2 as usize]
        //     .i_value
        //     .replace(self.stack[addi.arg_0 as usize].i_value.unwrap() + addi.arg_1 as i64);
    }

    fn exec_add(&mut self, add: &Instruction) {
        // self.stack[add.arg_2 as usize].i_value.replace(
        //     self.stack[add.arg_0 as usize].i_value.unwrap()
        //         + self.stack[add.arg_1 as usize].i_value.unwrap(),
        // );
    }

    fn exec_call(&mut self, call: &Instruction) {
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
        self.instruction_pointer = 0;
    }

    fn exec_new(&mut self, new: &Instruction) {
        // todo allocate on stack
        // for now just GC now
        self.mark_and_sweep();

        // self.heap.mark_and_sweep();
    }

    fn mark_and_sweep(&mut self) {
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
