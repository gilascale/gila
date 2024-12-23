use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    codegen::{BytecodeGenerator, Chunk, CodegenContext, CodegenResult, SlotManager},
    execution::{ExecutionEngine, ExecutionResult, Heap, ProcessContext, SharedExecutionContext},
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
    pub compilation_time: Duration,
    pub execution_time: Duration,
}
pub struct CompilationContext {
    pub codegen_context: CodegenContext,
    pub process_context: ProcessContext,
}

pub struct CompilerFlags {
    pub init_builtins: bool,
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
        compiler_flags: CompilerFlags,
        code: String,
        config: Config,
        codegen_context: Option<CodegenContext>,
        process_context: Option<ProcessContext>,
        shared_execution_context: Option<SharedExecutionContext>,
    ) -> CompilationResult {
        let start = Instant::now();
        let mut codegen_context = if codegen_context.is_some() {
            codegen_context.unwrap()
        } else {
            CodegenContext {
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
            }
        };
        let mut process_context = if process_context.is_some() {
            process_context.unwrap()
        } else {
            ProcessContext {
                stack_frame_pointer: 0,
                stack_frames: vec![],
                native_fns: HashMap::new(),
            }
        };
        let mut shared_execution_context = if shared_execution_context.is_some() {
            shared_execution_context.unwrap()
        } else {
            SharedExecutionContext {
                heap: Heap {
                    live_slots: vec![],
                    dead_objects: vec![],
                },
                gila_abis_dlls: vec![],
            }
        };
        let mut lexer = lex::Lexer::new();
        let mut bytecode_generator =
            BytecodeGenerator::new(config.clone(), codegen_context.clone());

        if compiler_flags.init_builtins {
            bytecode_generator.init_builtins();
        }

        let mut exec_engine =
            ExecutionEngine::new(config, shared_execution_context, process_context.clone());
        let tokens = lexer.lex(code);
        let mut parser = parse::Parser {
            tokens: &tokens,
            counter: 0,
        };
        let ast = parser.parse();
        let codegen_result = bytecode_generator.generate(&ast);

        let compilation_elapsed = start.elapsed();
        let execution_start = Instant::now();

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
        let execution_time = start.elapsed();

        return CompilationResult {
            codegen_result: codegen_result,
            execution_result: cloned_exec_result.clone(),
            compilation_time: compilation_elapsed,
            execution_time: execution_time,
        };
    }
}
