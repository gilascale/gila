use crate::codegen::{Chunk, Instruction, OpInstruction};

pub struct FnObject {}

pub struct StringObject {}

#[derive(Debug, Clone)]
pub struct HeapObject {}

#[derive(Debug, Clone)]
pub enum Object {
    F64(f64),
    I64(i64),
    HEAP_OBJECT(Box<HeapObject>),
}

pub struct ExecutionEngine {
    pub instruction_pointer: usize,
    pub running: bool,
    pub stack: std::vec::Vec<Object>,
}

impl ExecutionEngine {
    pub fn exec(&mut self, bytecode: Chunk) -> Object {
        // setup stack
        self.stack.resize(5, Object::I64(0));

        while self.running {
            let instr = &bytecode.instructions[self.instruction_pointer];
            self.exec_instr(instr);
            self.instruction_pointer += 1;
        }

        println!("stack: {:?}", self.stack);

        return Object::I64(0);
    }

    fn exec_instr(&mut self, instr: &Instruction) {
        match instr.op_instruction {
            OpInstruction::RETURN => self.running = false,
            OpInstruction::ADDI => self.exec_addi(instr),
            OpInstruction::ADD => self.exec_add(instr),
            _ => panic!("unknown instruction {:?}", instr.op_instruction),
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
}
