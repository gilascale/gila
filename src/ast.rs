use crate::lex::Token;

#[derive(Debug)]
pub enum Op {
    ADD,
    SUB,
    MUL,
    DIV,
}

#[derive(Debug)]
pub enum Expression {
    VARIABLE(Token),
    BIN_OP(Box<Expression>, Box<Expression>, Op),
    LITERAL_NUM(Token),
}

#[derive(Debug)]
pub enum Statement {
    RETURN(Option<Expression>),
    PROGRAM(Vec<Statement>),
    BLOCK(Vec<Statement>),
    EXPRESSION(Expression),
    // todo should these tokens be references?
    DEFINE(Token, Expression),
}
