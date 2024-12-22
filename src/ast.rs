use crate::{
    lex::{Position, Token},
    r#type::DataType,
};

#[derive(Debug, PartialEq)]
pub enum Op {
    BITWISE_OR,
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
    LOGICAL_OR,
}

// #[derive(Debug)]
// pub enum Expression {
//     VARIABLE(Token),
//     BIN_OP(Box<Expression>, Box<Expression>, Op),
//     LITERAL_NUM(Token),
// }

#[derive(Debug)]
pub enum Statement {
    ASSERT(Box<ASTNode>, Option<Token>),
    NAMED_ARG(Token, Box<ASTNode>),
    TRY(Box<ASTNode>),
    SLICE(Vec<ASTNode>),
    CALL(Box<ASTNode>, Vec<ASTNode>),
    ATOM(Token),
    VARIABLE(Token),
    BIN_OP(Box<ASTNode>, Box<ASTNode>, Op),
    LITERAL_BOOL(bool),
    LITERAL_NUM(Token),
    STRING(Token),
    RETURN(Option<Box<ASTNode>>),
    PROGRAM(Vec<ASTNode>),
    BLOCK(Vec<ASTNode>),
    MATCH(Box<ASTNode>, Vec<ASTNode>),
    MATCH_CASE(Token, Box<ASTNode>),
    // todo should these tokens be references?
    DEFINE(Token, Option<DataType>, Option<Box<ASTNode>>),
    ASSIGN(Box<ASTNode>, Box<ASTNode>),
    NAMED_FUNCTION(Token, Vec<ASTNode>, Option<DataType>, Box<ASTNode>),
    NAMED_TYPE_DECL(Token, Vec<ASTNode>),
    TEST(Box<ASTNode>, Box<ASTNode>),
    IF(Box<ASTNode>, Box<ASTNode>, Option<Box<ASTNode>>),
    FOR(Token, Token, Token, Box<ASTNode>),
    INDEX(Box<ASTNode>, Box<ASTNode>),
    ANNOTATION(Token, Vec<Token>, Box<ASTNode>),
    STRUCT_ACCESS(Box<ASTNode>, Token),
    IMPORT(Vec<Token>),
}

#[derive(Debug)]
pub struct ASTNode {
    pub statement: Statement,
    pub position: Position,
}
