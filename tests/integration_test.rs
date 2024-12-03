use std::{collections::HashMap, fs};

use gila::{
    codegen::{BytecodeGenerator, Chunk, CodegenContext},
    config::Config,
    execution::{ProcessContext, ExecutionEngine, Heap, Object, RuntimeError},
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
    let mut execution_context = ProcessContext {
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

    let mut exec_engine: ExecutionEngine<'_> =
        ExecutionEngine::new(&config, &mut execution_context);
    let tokens = lexer.lex(code);
    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();
    let bytecode = bytecode_generator.generate(&ast);
    exec_engine.exec(bytecode, false)
}

macro_rules! dynamic_test {
    ($test_name:ident, $file_path:expr) => {
        #[test]
        fn $test_name() {
            use std::fs;
            let source = fs::read_to_string($file_path).expect("Unable to read file");
            let result = compile_and_execute(source);
            assert_eq!(result.is_ok(), true);
        }
    };
}

dynamic_test!(test_constructor, "./tests/gila/constructor.gila");
dynamic_test!(addition, "./tests/gila/addition.gila");
dynamic_test!(logical_operators, "./tests/gila/logical_operators.gila");
