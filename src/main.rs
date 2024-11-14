mod analyse;
mod ast;
mod codegen;
mod lex;
mod parse;

use std::fs;

fn main() {
    // const source: &str = "main fn 123 end";
    let source = fs::read_to_string("C:/Users/jking/dev/gila/example/minimal.gila")
        .expect("Unable to read file");

    let lexer = lex::Lexer {};
    let tokens = lexer.lex(source.to_string());
    // println!("tokens {:?}", tokens);

    // let ast = ast::Statement::PROGRAM(vec![ast::Statement::EXPRESSION(ast::Expression::BIN_OP(
    //     Box::new(ast::Expression::LITERAL_NUM(1.0)),
    //     Box::new(ast::Expression::LITERAL_NUM(1.0)),
    //     ast::Op::ADD,

    // ))]);

    println!("tokens {:?}", tokens);

    let mut parser = parse::Parser {
        tokens: &tokens,
        counter: 0,
    };
    let ast = parser.parse();

    println!("ast {:?}", ast);
    // let analyser = analyse::Analyser {};
    // let code_generator = codegen::CodeGenerator {};

    // analyser.analyse(&ast);
    // code_generator.generate(&ast);
}
