use std::u8;

use crate::codegen::{Bytecode, Instruction, OpInstruction};

pub struct Object {}

pub struct ExecutionEngine {
    pub instruction_pointer: usize,
    pub running: bool,
    pub stack: std::vec::Vec<u8>,
}

impl ExecutionEngine {
    pub fn exec(&mut self, bytecode: Bytecode) -> Object {
        // setup stack
        self.stack.resize(5, 0);

        while self.running {
            let instr = &bytecode.instructions[self.instruction_pointer];
            self.exec_instr(instr);
            self.instruction_pointer += 1;
        }

        println!("stack: {:?}", self.stack);

        return Object {};
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
        self.stack[addi.arg_2 as usize] = self.stack[addi.arg_0 as usize] + addi.arg_1;
    }

    fn exec_add(&mut self, add: &Instruction) {
        self.stack[add.arg_2 as usize] =
            self.stack[add.arg_0 as usize] + self.stack[add.arg_1 as usize];
    }
}
