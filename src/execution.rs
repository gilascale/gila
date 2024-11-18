use crate::codegen::{Chunk, Instruction, OpInstruction};

#[derive(Debug, Clone)]
pub struct FnObject {
    pub chunk: Chunk,
}

#[derive(Debug, Clone)]
pub struct StringObject {
    pub s: std::string::String,
}

#[derive(Debug, Clone)]
pub enum HeapObjectData {
    FN(FnObject),
    STRING(StringObject),
}

#[derive(Debug, Clone)]
pub struct HeapObject {
    pub data: HeapObjectData,
    pub is_marked: bool,
}

#[derive(Debug, Clone)]
pub enum Object {
    F64(f64),
    I64(i64),
    HEAP_OBJECT(Box<HeapObject>),
}

impl Object {
    pub fn print(&self) -> std::string::String {
        match self {
            Self::F64(f) => f.to_string(),
            Self::I64(i) => i.to_string(),
            Self::HEAP_OBJECT(h) => format!("<HeapObject at {:p}>", *h),
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
    pub fn exec(&mut self, bytecode: Chunk) -> Object {
        self.init_startup_stack(Box::new(FnObject { chunk: bytecode }));
        self.zero_stack();
        while self.running {
            // let current_instruction = self.current_instruction().clone();
            let instr = {
                let current_frame = &self.stack_frames[self.stack_frame_pointer];
                &current_frame.fn_object.chunk.instructions[current_frame.instruction_pointer]
                    .clone()
            };

            self.exec_instr(instr);
        }

        // println!("stack: {:#?}", self.stack_frames);
        return Object::I64(0);
    }

    fn exec_instr(&mut self, instr: &Instruction) {
        match instr.op_instruction {
            OpInstruction::RETURN => self.exec_return(instr),
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::SUBI => self.exec_subi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            OpInstruction::CALL => self.exec_call(instr),
            OpInstruction::NEW => self.exec_new(instr),
            OpInstruction::LOAD_CONST => self.exec_load_const(instr),
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

    fn exec_return(&mut self, ret: &Instruction) {
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
    }

    fn exec_addi(&mut self, addi: &Instruction) {
        self.stack_frames[self.stack_frame_pointer].stack[addi.arg_2 as usize] =
            Object::I64((addi.arg_0 + addi.arg_1).into());

        // self.stack[addi.arg_2 as usize]
        //     .i_value
        //     .replace(self.stack[addi.arg_0 as usize].i_value.unwrap() + addi.arg_1 as i64);
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
    }
    fn exec_subi(&mut self, subi: &Instruction) {
        // self.stack[addi.arg_2 as usize]
        //     .i_value
        //     .replace(self.stack[addi.arg_0 as usize].i_value.unwrap() + addi.arg_1 as i64);
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
    }

    fn exec_add(&mut self, add: &Instruction) {
        let lhs = &self.stack_frames[self.stack_frame_pointer].stack[add.arg_0 as usize];
        let rhs = &self.stack_frames[self.stack_frame_pointer].stack[add.arg_1 as usize];

        // todo check type but for now treat as in
        if let (Object::I64(i1), Object::I64(i2)) = (lhs, rhs) {
            self.stack_frames[self.stack_frame_pointer].stack[add.arg_2 as usize] =
                Object::I64(i1 + i2);
        }
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
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
        self.zero_stack();
    }

    fn exec_new(&mut self, new: &Instruction) {
        // todo allocate on stack
        // for now just GC now
        self.mark_and_sweep();

        // get the type from the stack
        let type_object = &self.stack_frames[self.stack_frame_pointer].stack[new.arg_0 as usize];

        // self.heap.mark_and_sweep();
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
    }

    fn exec_load_const(&mut self, load_const: &Instruction) {
        let const_obj = &self.stack_frames[self.stack_frame_pointer]
            .fn_object
            .chunk
            .constant_pool[load_const.arg_0 as usize];

        println!("loaded object {:?}", const_obj.print());
        self.stack_frames[self.stack_frame_pointer].stack[load_const.arg_1 as usize] =
            const_obj.clone();
        self.stack_frames[self.stack_frame_pointer].instruction_pointer += 1;
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
