

#[derive(Debug)]
pub enum Op{
    ADD,
    SUB,
    MUL,
    DIV
}

#[derive(Debug)]
pub enum Expression{
    BIN_OP(Box<Expression>, Box<Expression>, Op),
    LITERAL_NUM(f32)
}

#[derive(Debug)]
pub enum Statement {
    PROGRAM(Vec<Statement>),
    BLOCK(Vec<Statement>),
    EXPRESSION(Expression)
}

