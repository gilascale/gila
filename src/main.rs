mod analyse;
mod ast;
mod codegen;
mod compiler;
mod config;
mod execution;
mod lex;
mod parse;
mod r#type;

use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;
use std::vec;
use std::{
    fs,
    io::{self, Write},
};

use analyse::TypeCheckError;
use codegen::{BytecodeGenerator, Chunk, CodegenContext, SlotManager};
use config::Config;
use deepsize::DeepSizeOf;
use execution::Heap;
use execution::{ExecutionEngine, SharedExecutionContext};
use execution::{Object, ProcessContext};
use lex::Lexer;

fn load_prelude<'a>(
    config: &'a Config,
    shared_execution_context: &mut SharedExecutionContext,
    codegen_context: &mut CodegenContext,
    execution_context: &'a mut ProcessContext,
) {
    // todo this should work the same way an import works basically

    let mut lexer = lex::Lexer::new();
    let mut bytecode_generator = BytecodeGenerator::new(&config, codegen_context);

    bytecode_generator.init_builtins();

    let mut exec_engine = ExecutionEngine::new(config, shared_execution_context, execution_context);
    let source = fs::read_to_string("./prelude/prelude.gila").expect("Unable to read file");
    let tokens = lexer.lex(source);
    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();
    let bytecode = bytecode_generator.generate(&ast);
    exec_engine.exec("prelude".to_string(), bytecode, false);
}

fn repl() {
    // let config = Config { max_memory: 1000 };
    // let mut shared_execution_context = SharedExecutionContext {
    //     heap: Heap {
    //         live_slots: vec![],
    //         dead_objects: vec![],
    //     },
    // };
    // let mut codegen_context = CodegenContext {
    //     current_chunk_pointer: 0,
    //     chunks: vec![Chunk {
    //         current_register: 0,
    //         debug_line_info: vec![],
    //         constant_pool: vec![],
    //         gc_ref_data: vec![],
    //         instructions: vec![],
    //         variable_map: HashMap::new(),
    //         string_interns: HashMap::new(),
    //     }],
    // };
    // let mut environment = ProcessContext {
    //     stack_frame_pointer: 0,
    //     stack_frames: vec![],
    //     native_fns: HashMap::new(),
    // };

    // load_prelude(
    //     &config,
    //     &mut shared_execution_context,
    //     &mut codegen_context,
    //     &mut environment,
    // );

    // let mut lexer = lex::Lexer::new();
    // let mut bytecode_generator = BytecodeGenerator::new(&config, &mut codegen_context);
    // let mut exec_engine =
    //     ExecutionEngine::new(&config, &mut shared_execution_context, &mut environment);

    // loop {
    //     let mut line = String::new();
    //     print!(">>");
    //     io::stdout().flush();
    //     std::io::stdin().read_line(&mut line).unwrap();
    //     let tokens = lexer.lex(line);
    //     let mut parser = parse::Parser {
    //         tokens: &tokens,
    //         counter: 0,
    //     };
    //     let ast = parser.parse();
    //     // each time this iterates, it incrementally compiles, is this what we want... probably?
    //     let bytecode = bytecode_generator.generate(&ast);
    //     // println!("generated bytecode {:#?}", bytecode);
    //     let result = exec_engine.exec("anon".to_string(), bytecode, true);
    //     match result {
    //         Ok(o) => {
    //             println!("={:?}", o);
    //         }
    //         Err(e) => {
    //             println!("encountered runtime exception {:?}", e);
    //             exec_engine.print_stacktrace();
    //         }
    //     }
    // }
}

fn print_typecheck_error(source: String, typecheck_err: TypeCheckError) {
    println!("Typecheck Error:\n");
    let split_source = source.lines().collect::<Vec<&str>>();

    match typecheck_err {
        TypeCheckError::TYPE_NOT_ASSIGNABLE(lhs, rhs, lhs_type, rhs_type) => {
            println!("{}", split_source[lhs.line as usize]);
            let left_squiggle = "^".repeat((lhs.index_end - lhs.index) as usize);
            let right_squiggle = "^".repeat((rhs.index_end - rhs.index) as usize);
            let offset = rhs.index - lhs.index + 1;
            println!(
                "{}{}{}{}",
                " ".repeat(lhs.index as usize),
                left_squiggle,
                " ".repeat(offset as usize),
                right_squiggle
            );
            println!("{:?} not assignable to {:?}.\n", rhs_type, lhs_type)
        }
        TypeCheckError::UNKNOWN_VARIABLE(t) => {
            println!("{}", split_source[t.pos.line as usize]);
            let left_squiggle = "^".repeat((t.pos.index_end - t.pos.index) as usize);
            println!("{}{}", " ".repeat(t.pos.index as usize), left_squiggle);
            println!("unknown variable {:?}.\n", t.typ);
        }
        TypeCheckError::UNKNOWN_DATA_TYPE(data_type, pos) => {
            println!("{}", split_source[pos.line as usize]);
            let left_squiggle = "^".repeat((pos.index_end - pos.index) as usize);
            println!("{}{}", " ".repeat(pos.index as usize), left_squiggle);
            println!("unknown data type {:?}.\n", data_type);
        }
        TypeCheckError::MISSING_ARGUMENT => println!("missing argument"),
    }
}

fn exec(file_to_exec: String) {
    let start = Instant::now();

    fs::create_dir_all("./gila-build");

    let config = Config {
        max_memory: 100_000,
    };
    let mut shared_execution_context = SharedExecutionContext {
        heap: Heap {
            live_slots: vec![],
            dead_objects: vec![],
        },
    };
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
    let mut environment = ProcessContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        native_fns: HashMap::new(),
    };

    load_prelude(
        &config,
        &mut shared_execution_context,
        &mut codegen_context,
        &mut environment,
    );

    let source = fs::read_to_string(file_to_exec.to_string()).expect("Unable to read file");
    let mut lexer = lex::Lexer::new();
    let tokens = lexer.lex(source.clone());
    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();
    // println!("ast {:#?}", ast);
    let mut file = File::create("./gila-build/parsed.gilaast");
    file.unwrap().write_all(format!("{:#?}", ast).as_bytes());

    let mut analyser = analyse::Analyser::new();
    let typecheck_res = analyser.analyse(&ast);
    if typecheck_res.is_err() {
        print_typecheck_error(source.clone(), typecheck_res.err().unwrap());
        return;
    }

    let mut bytecode_generator = BytecodeGenerator::new(&config, &mut codegen_context);

    let bytecode = bytecode_generator.generate(&ast);
    // println!("{:#?}", bytecode);

    let mut file = File::create("./gila-build/bytecode.giladbg");
    file.unwrap()
        .write_all(bytecode.dump_to_file_format(&source).as_bytes());

    let mut execution_engine =
        ExecutionEngine::new(&config, &mut shared_execution_context, &mut environment);

    let result = execution_engine.exec(file_to_exec.to_string(), bytecode, false);
    let elapsed = start.elapsed();

    match result {
        Ok(o) => {}
        Err(e) => {
            println!("encountered runtime exception {:?}", e);
            execution_engine.print_stacktrace();
        }
    }
    let denominator = 1000_000;
    println!(
        "finished in {:.9?}s & used {:.9?}MB",
        elapsed.as_secs_f64(),
        execution_engine
            .shared_execution_context
            .heap
            .live_slots
            .deep_size_of()
            / denominator
    );
}

fn do_test(file_to_test: String) {}

enum Mode {
    FILE(String),
    REPL,
    TEST(String),
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_to_exec: String = args[3].to_string();
    let mode = Mode::FILE(file_to_exec);
    // let mode = Mode::REPL;

    match mode {
        Mode::FILE(path) => exec(path),
        Mode::REPL => repl(),
        Mode::TEST(path) => do_test(path),
    }
}
