mod analyse;
mod ast;
mod codegen;
mod config;
mod execution;
mod lex;
mod parse;
mod r#type;

use std::time::Instant;
use std::{
    fs,
    io::{self, Write},
};

use codegen::BytecodeGenerator;
use config::Config;
use deepsize::DeepSizeOf;
use execution::ExecutionEngine;

fn main() {
    let repl = false;

    if repl {
        // loop {
        //     let lexer = lex::Lexer {};
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
        //     let mut bytecode_generator = BytecodeGenerator::new();
        //     let bytecode = bytecode_generator.generate(&ast);
        //     let mut exec_engine = ExecutionEngine {
        //         stack_frame_pointer: 0,
        //         running: true,
        //         stack_frames: vec![],
        //         heap: execution::Heap { objects: vec![] },
        //     };
        //     println!("bytecode={:#?}", bytecode);
        //     let result = exec_engine.exec(bytecode);
        //     match result {
        //         Ok(o) => println!("={} (type={})", o.print(), o.get_type().print()),
        //         Err(e) => println!("encountered runtime exception {:?}", e),
        //     }
        // }
    } else {
        let config = Config { max_memory: 1000 };

        let start = Instant::now();
        let source = fs::read_to_string("C:/Users/james/dev/gila/example/test.gila")
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
        let mut bytecode_generator = BytecodeGenerator::new();

        // analyser.analyse(&ast);
        let bytecode = bytecode_generator.generate(&ast);

        // println!("bytecode: \n{:#?}", bytecode);

        let mut execution_engine = ExecutionEngine::new(&config);

        let result = execution_engine.exec(bytecode);
        let elapsed = start.elapsed();

        match result {
            Ok(o) => {
                println!("={}", o.print());
            }
            Err(e) => println!("encountered runtime exception {:?}", e),
        }
        let denominator = 1000_000;
        println!(
            "finished in {:.9?}s & used {:.9?}MB",
            elapsed.as_secs_f64(),
            execution_engine.heap.live_slots.deep_size_of() / denominator
        );
    }
}
