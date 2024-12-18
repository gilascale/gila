use std::{collections::HashMap, fs};

use crate::{
    codegen::{BytecodeGenerator, Chunk, CodegenContext, SlotManager},
    execution::{ExecutionEngine, Heap, ProcessContext, SharedExecutionContext},
    lex, parse,
};

use crate::config::Config;

pub enum CompilationUnitStatus {
    TODO,
    DONE,
    ERROR,
}

pub struct CompilationContext {
    pub codegen_context: CodegenContext,
    pub process_context: ProcessContext,
}

pub struct Compiler {
    // keep track of files and their states
    pub compilation_units: HashMap<String, CompilationUnitStatus>,
}

impl Compiler {
    pub fn new() -> Self {
        return Compiler {
            compilation_units: HashMap::new(),
        };
    }

    pub fn compile_and_exec(
        &mut self,
        compilation_unit: String,
        code: String,
        config: &Config,
        shared_execution_context: &mut SharedExecutionContext, // mut codegen_context: &mut CodegenContext,
                                                               // execution_context: &mut ProcessContext,
    ) -> Option<CompilationContext> {
        if !self.compilation_units.contains_key(&compilation_unit) {
            let mut codegen_context = CodegenContext {
                current_chunk_pointer: 0,
                chunks: vec![Chunk {
                    slot_manager: SlotManager::new(),
                    debug_line_info: vec![],
                    constant_pool: vec![],
                    gc_ref_data: vec![],
                    instructions: vec![],
                    variable_map: HashMap::new(),
                    string_interns: HashMap::new(),
                }],
            };
            let mut process_context = ProcessContext {
                stack_frame_pointer: 0,
                stack_frames: vec![],
                native_fns: HashMap::new(),
            };
            let mut lexer = lex::Lexer::new();
            let mut bytecode_generator = BytecodeGenerator::new(config, &mut codegen_context);

            let mut exec_engine =
                ExecutionEngine::new(config, shared_execution_context, &mut process_context);
            let tokens = lexer.lex(code);
            let mut parser = parse::Parser {
                tokens: &tokens,
                counter: 0,
            };
            let ast = parser.parse();
            let bytecode = bytecode_generator.generate(&ast);
            let result = exec_engine.exec(compilation_unit.to_string(), bytecode, false);

            match result {
                Ok(_) => self
                    .compilation_units
                    .insert(compilation_unit.to_string(), CompilationUnitStatus::DONE),
                Err(e) => self
                    .compilation_units
                    .insert(compilation_unit.to_string(), CompilationUnitStatus::ERROR),
            };

            return Some(CompilationContext {
                codegen_context,
                process_context,
            });
        }
        None
    }
}
