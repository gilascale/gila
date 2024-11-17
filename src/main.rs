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
    // // const source: &str = "main fn 123 end";
    // let source = fs::read_to_string("C:/Users/jking/dev/gila/example/minimal.gila")
    //     .expect("Unable to read file");

    // let lexer = lex::Lexer {};
    // let tokens = lexer.lex(source);
    // // println!("tokens {:?}", tokens);

    // // let ast = ast::Statement::PROGRAM(vec![ast::Statement::EXPRESSION(ast::Expression::BIN_OP(
    // //     Box::new(ast::Expression::LITERAL_NUM(1.0)),
    // //     Box::new(ast::Expression::LITERAL_NUM(1.0)),
    // //     ast::Op::ADD,

    // // ))]);

    // println!("tokens {:?}", tokens);

    // let mut parser = parse::Parser {
    //     tokens: &tokens,
    //     counter: 0,
    // };
    // let ast = parser.parse();

    // println!("ast {:?}", ast);
    // let analyser = analyse::Analyser {};
    // let code_generator = codegen::CodeGenerator {};

    // analyser.analyse(&ast);
    // code_generator.generate(&ast);

    while true {
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
        let bytecode_generator = BytecodeGenerator {};
        let bytecode = bytecode_generator.generate(&ast);
        bytecode.print();
        let mut exec_engine = ExecutionEngine {
            instruction_pointer: 0,
            stack_frame_pointer: 0,
            running: true,
            stack_frames: vec![],
            heap: execution::Heap { objects: vec![] },
        };
        exec_engine.exec(bytecode);
    }
}
