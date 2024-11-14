mod analyse;
mod ast;
mod codegen;
mod lex;

fn main() {
    const source: &str = "main = pass";

    let lexer = lex::Lexer {};
    let tokens = lexer.lex(source);
    println!("tokens {:?}", tokens);

    let ast = ast::Statement::PROGRAM(vec![ast::Statement::EXPRESSION(ast::Expression::BIN_OP(
        Box::new(ast::Expression::LITERAL_NUM(1.0)),
        Box::new(ast::Expression::LITERAL_NUM(1.0)),
        ast::Op::ADD,
    ))]);

    let analyser = analyse::Analyser {};
    let code_generator = codegen::CodeGenerator {};

    analyser.analyse(&ast);
    code_generator.generate(&ast);
}
