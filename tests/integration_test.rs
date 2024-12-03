use std::{collections::HashMap, fs};

use gila::{
    codegen::{BytecodeGenerator, Chunk, CodegenContext},
    config::Config,
    execution::{ExecutionContext, ExecutionEngine, Heap, Object, RuntimeError},
    lex, parse,
};

fn compile_and_execute(code: String) -> Result<Object, RuntimeError> {
    let config = Config {
        max_memory: 100_000,
    };
    let mut codegen_context = CodegenContext {
        current_chunk_pointer: 0,
        chunks: vec![Chunk {
            current_register: 0,
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
            string_interns: HashMap::new(),
        }],
    };
    let mut execution_context = ExecutionContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        native_fns: HashMap::new(),
        heap: Heap {
            live_slots: vec![],
            dead_objects: vec![],
        },
    };

    let mut lexer = lex::Lexer::new();
    let mut bytecode_generator = BytecodeGenerator::new(&config, &mut codegen_context);

    let mut exec_engine = ExecutionEngine::new(&config, &mut execution_context);
    let source = fs::read_to_string("./prelude/prelude.gila").expect("Unable to read file");
    let tokens = lexer.lex(source);
    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();
    let bytecode = bytecode_generator.generate(&ast);
    exec_engine.exec(bytecode, false)
}

#[test]
pub fn test_files() {
    let source = fs::read_to_string("./tests/gila/constructor.gila").expect("Unable to read file");

    let result = compile_and_execute(source);
    assert_eq!(result.is_ok(), true);
}
