use crate::lex::{Position, Token};

#[derive(Debug)]
pub enum Op {
    ADD,
    SUB,
    MUL,
    DIV,
}

// #[derive(Debug)]
// pub enum Expression {
//     VARIABLE(Token),
//     BIN_OP(Box<Expression>, Box<Expression>, Op),
//     LITERAL_NUM(Token),
// }

#[derive(Debug)]
pub enum Statement {
    CALL(Box<ASTNode>),
    ATOM(Token),
    VARIABLE(Token),
    BIN_OP(Box<ASTNode>, Box<ASTNode>, Op),
    LITERAL_NUM(Token),
    STRING(Token),
    RETURN(Option<Box<ASTNode>>),
    PROGRAM(Vec<ASTNode>),
    BLOCK(Vec<ASTNode>),
    // todo should these tokens be references?
    DEFINE(Token, Box<ASTNode>),
    NAMED_FUNCTION(Token, Box<ASTNode>),
    IF(Box<ASTNode>, Box<ASTNode>),
}

#[derive(Debug)]
pub struct ASTNode {
    pub statement: Statement,
    pub position: Position,
}
