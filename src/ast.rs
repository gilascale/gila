use crate::{
    lex::{Position, Token},
    r#type::DataType,
};

#[derive(Debug, PartialEq)]
pub enum Op {
    ADD,
    SUB,
    MUL,
    DIV,
    EQ,
    NEQ,
    GT,
    GE,
    LT,
    LE,
}

// #[derive(Debug)]
// pub enum Expression {
//     VARIABLE(Token),
//     BIN_OP(Box<Expression>, Box<Expression>, Op),
//     LITERAL_NUM(Token),
// }

#[derive(Debug)]
pub enum Statement {
    SLICE(Vec<ASTNode>),
    CALL(Box<ASTNode>, Vec<ASTNode>),
    ATOM(Token),
    VARIABLE(Token),
    BIN_OP(Box<ASTNode>, Box<ASTNode>, Op),
    LITERAL_NUM(Token),
    STRING(Token),
    RETURN(Option<Box<ASTNode>>),
    PROGRAM(Vec<ASTNode>),
    BLOCK(Vec<ASTNode>),
    // todo should these tokens be references?
    DEFINE(Token, Option<DataType>, Option<Box<ASTNode>>),
    NAMED_FUNCTION(Token, Vec<ASTNode>, Box<ASTNode>),
    NAMED_TYPE_DECL(Token, Vec<ASTNode>),
    IF(Box<ASTNode>, Box<ASTNode>, Option<Box<ASTNode>>),
    INDEX(Box<ASTNode>, Box<ASTNode>),
    ANNOTATION(Token, Vec<Token>, Box<ASTNode>),
    STRUCT_ACCESS(Box<ASTNode>, Token),
    IMPORT(Token),
}

#[derive(Debug)]
pub struct ASTNode {
    pub statement: Statement,
    pub position: Position,
}
