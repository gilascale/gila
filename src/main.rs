mod lex;
mod ast;
mod analyse;
mod codegen;

fn main() {

    const source: &str = "print('hi')";

    let lexer = lex::Lexer{};
    lexer.lex(source);

    let ast = ast::Statement::PROGRAM(vec![
        ast::Statement::EXPRESSION(
            ast::Expression::BIN_OP(
                Box::new(ast::Expression::LITERAL_NUM(1.0)), 
                Box::new(ast::Expression::LITERAL_NUM(1.0)),
            ast::Op::ADD)
        )
    ]);

    let analyser = analyse::Analyser{};
    let code_generator = codegen::CodeGenerator{};

    analyser.analyse(&ast);
    code_generator.generate(&ast);
}
