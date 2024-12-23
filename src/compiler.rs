use std::collections::HashMap;

use crate::{
    codegen::{BytecodeGenerator, Chunk, CodegenContext, CodegenResult, SlotManager},
    execution::{ExecutionEngine, ExecutionResult, ProcessContext, SharedExecutionContext},
    lex, parse,
};

use crate::config::Config;

pub enum CompilationUnitStatus {
    TODO,
    DONE,
    ERROR,
}

#[derive(Clone)]
pub struct CompilationResult {
    pub codegen_result: CodegenResult,
    pub execution_result: ExecutionResult,
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

    // todo fix this and return the new shared execution context
    pub fn compile_and_exec(
        &mut self,
        compilation_unit: String,
        code: String,
        config: Config,
        shared_execution_context: SharedExecutionContext, // mut codegen_context: &mut CodegenContext,
                                                          // execution_context: &mut ProcessContext,
    ) -> CompilationResult {
        println!("doing compile and exec...");
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
        let mut bytecode_generator =
            BytecodeGenerator::new(config.clone(), codegen_context.clone());

        let mut exec_engine =
            ExecutionEngine::new(config, shared_execution_context, process_context.clone());
        let tokens = lexer.lex(code);
        let mut parser = parse::Parser {
            tokens: &tokens,
            counter: 0,
        };
        let ast = parser.parse();
        let codegen_result = bytecode_generator.generate(&ast);
        let execution_result = exec_engine.exec(
            compilation_unit.to_string(),
            codegen_result.codegen_context.chunks[0].clone(),
            false,
        );

        let cloned_exec_result = execution_result.clone();
        match cloned_exec_result.clone().result {
            Ok(_) => self
                .compilation_units
                .insert(compilation_unit.to_string(), CompilationUnitStatus::DONE),
            Err(e) => self
                .compilation_units
                .insert(compilation_unit.to_string(), CompilationUnitStatus::ERROR),
        };

        return CompilationResult {
            codegen_result: codegen_result,
            execution_result: cloned_exec_result.clone(),
        };
    }
}
