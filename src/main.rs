mod analyse;
mod ast;
mod codegen;
mod config;
mod execution;
mod lex;
mod parse;
mod r#type;

use std::collections::HashMap;
use std::time::Instant;
use std::{
    fs,
    io::{self, Write},
};

use codegen::{BytecodeGenerator, Chunk, CodegenContext};
use config::Config;
use deepsize::DeepSizeOf;
use execution::ExecutionContext;
use execution::ExecutionEngine;
use execution::Heap;

fn repl() {
    let config = Config { max_memory: 1000 };
    let mut codegen_context = CodegenContext {
        current_register: 0,
        current_chunk_pointer: 0,
        chunks: vec![Chunk {
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
        }],
    };
    let mut environment = ExecutionContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        heap: Heap {
            config: &config,
            live_slots: vec![],
            dead_objects: vec![],
        },
    };

    let lexer = lex::Lexer {};
    let mut bytecode_generator = BytecodeGenerator::new(&mut codegen_context);
    let mut exec_engine = ExecutionEngine::new(&config, &mut environment);

    loop {
        let mut line = String::new();
        print!(">>");
        io::stdout().flush();
        std::io::stdin().read_line(&mut line).unwrap();
        let tokens = lexer.lex(line);
        let mut parser = parse::Parser {
            tokens: &tokens,
            counter: 0,
        };
        let ast = parser.parse();
        // each time this iterates, it incrementally compiles, is this what we want... probably?
        let bytecode = bytecode_generator.generate(&ast);
        // println!("generated bytecode {:#?}", bytecode);
        let result = exec_engine.exec(bytecode, true);
        match result {
            Ok(o) => {
                println!("={}", o.print());
            }
            Err(e) => {
                println!("encountered runtime exception {:?}", e);
                exec_engine.print_stacktrace();
            }
        }
    }
}

fn exec() {
    let config = Config { max_memory: 1000 };
    let mut codegen_context = CodegenContext {
        current_register: 0,
        current_chunk_pointer: 0,
        chunks: vec![Chunk {
            debug_line_info: vec![],
            constant_pool: vec![],
            gc_ref_data: vec![],
            instructions: vec![],
            variable_map: HashMap::new(),
        }],
    };
    let mut environment = ExecutionContext {
        stack_frame_pointer: 0,
        stack_frames: vec![],
        heap: Heap {
            config: &config,
            live_slots: vec![],
            dead_objects: vec![],
        },
    };

    let start = Instant::now();
    let source = fs::read_to_string("C:/Users/jking/dev/gila/example/test.gila")
        .expect("Unable to read file");
    let lexer = lex::Lexer {};
    let tokens = lexer.lex(source);
    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();
    // println!("{:#?}", ast);

    // let analyser = analyse::Analyser {};
    let mut bytecode_generator = BytecodeGenerator::new(&mut codegen_context);

    // analyser.analyse(&ast);
    let bytecode = bytecode_generator.generate(&ast);

    println!("bytecode: \n{:#?}", bytecode);

    let mut execution_engine = ExecutionEngine::new(&config, &mut environment);

    let result = execution_engine.exec(bytecode, false);
    let elapsed = start.elapsed();

    match result {
        Ok(o) => {
            println!("={}", o.print());
        }
        Err(e) => {
            println!("encountered runtime exception {:?}", e);
            execution_engine.print_stacktrace();
        }
    }
    let denominator = 1000_000;
    println!(
        "finished in {:.9?}s & used {:.9?}MB",
        elapsed.as_secs_f64(),
        execution_engine.environment.heap.live_slots.deep_size_of() / denominator
    );
}

fn main() {
    let should_repl = false;

    if should_repl {
        repl()
    } else {
        exec();
    }
}
