mod analyse;
mod ast;
mod codegen;
mod execution;
mod lex;
mod parse;

use std::{
    fs,
    io::{self, Write},
};

use codegen::BytecodeGenerator;
use execution::ExecutionEngine;

fn main() {
    let repl = true;

    if repl {
        loop {
            let lexer = lex::Lexer {};
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
            let mut bytecode_generator = BytecodeGenerator::new();
            let bytecode = bytecode_generator.generate(&ast);
            let mut exec_engine = ExecutionEngine {
                stack_frame_pointer: 0,
                running: true,
                stack_frames: vec![],
                heap: execution::Heap { objects: vec![] },
            };
            println!("bytecode={:#?}", bytecode);
            let result = exec_engine.exec(bytecode);
            match result {
                Ok(o) => println!("={}", o.print()),
                Err(e) => println!("encountered runtime exception {:?}", e),
            }
        }
    } else {
        let source = fs::read_to_string("C:/Users/jking/dev/gila/example/minimal.gila")
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

        let mut execution_engine = ExecutionEngine::new();

        let result = execution_engine.exec(bytecode);

        match result {
            Ok(o) => println!("={}", o.print()),
            Err(e) => println!("encountered runtime exception {:?}", e),
        }
    }
}
