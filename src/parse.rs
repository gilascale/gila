use crate::{
    ast::{ASTNode, Op, Statement},
    lex::{Position, Token, Type},
};

pub struct Parser<'a> {
    pub tokens: &'a std::vec::Vec<Token>,
    pub counter: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> ASTNode {
        let mut program: Vec<ASTNode> = vec![];

        while !self.end() {
            program.push(self.statement());
        }
        // Statement::PROGRAM(vec![Statement::EXPRESSION(Expression::BIN_OP(
        //     Box::new(Expression::LITERAL_NUM(1.0)),
        //     Box::new(Expression::LITERAL_NUM(1.0)),
        //     Op::ADD,
        // ))])
        ASTNode {
            statement: Statement::PROGRAM(program),
            position: Position { index: 0, line: 0 },
        }
    }

    fn statement(&mut self) -> ASTNode {
        let current: &Token = &self.tokens[self.counter];

        match current.typ {
            Type::RETURN => self.ret(),
            Type::IDENTIFIER(_) => self.identifier(),
            _ => self.expression(),
        }
    }

    fn expression(&mut self) -> ASTNode {
        // let higher_prece

        return self.add_sub();
    }

    fn single(&mut self) -> ASTNode {
        let next = &self.tokens[self.counter];
        match (next.typ) {
            Type::NUMBER(_) => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_NUM(next.clone()),
                    position: Position { index: 0, line: 0 },
                };
            }
            // _ => higher_precedence,
            _ => panic!(),
        }
    }

    fn add_sub(&mut self) -> ASTNode {
        let higher_precedence = self.mul_div();
        if !self.end() && self.tokens[self.counter].typ == Type::ADD {
            self.counter += 1;
            let rhs: ASTNode = self.expression();
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::ADD),
                position: Position { index: 0, line: 0 },
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::SUB {
            self.counter += 1;
            let rhs = self.expression();
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::SUB),
                position: Position { index: 0, line: 0 },
            };
        }
        return higher_precedence;
    }

    fn mul_div(&mut self) -> ASTNode {
        let higher_precedence = self.single();
        if !self.end() && self.tokens[self.counter].typ == Type::MUL {
            self.counter += 1;
            let rhs = self.expression();
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::MUL),
                position: Position { index: 0, line: 0 },
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::DIV {
            self.counter += 1;
            let rhs = self.expression();
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::DIV),
                position: Position { index: 0, line: 0 },
            };
        }
        return higher_precedence;
    }

    fn block(&mut self) -> ASTNode {
        let mut stms = vec![];
        while !self.end() && self.tokens[self.counter].typ != Type::END {
            stms.push(self.statement());
        }
        self.counter += 1;
        return ASTNode {
            statement: Statement::BLOCK(stms),
            position: Position { index: 0, line: 0 },
        };
    }

    fn ret(&mut self) -> ASTNode {
        self.counter += 1;
        return ASTNode {
            statement: Statement::RETURN(None),
            position: Position { index: 0, line: 0 },
        };
    }

    fn identifier(&mut self) -> ASTNode {
        // todo assume define?

        let identifier = &self.tokens[self.counter];
        self.counter += 1;

        if self.end() {
            return ASTNode {
                statement: Statement::VARIABLE(identifier.clone()),
                position: Position { index: 0, line: 0 },
            };
        }

        // function
        // todo deal with blocks?
        if self.tokens[self.counter].typ == Type::FN {
            self.counter += 1;
            return ASTNode {
                statement: Statement::NAMED_FUNCTION(identifier.clone(), Box::new(self.block())),
                position: Position { index: 0, line: 0 },
            };
        }

        if self.tokens[self.counter].typ == Type::ASSIGN {
            self.counter += 1;
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), Box::new(self.expression())),
                position: Position { index: 0, line: 0 },
            };
        }

        panic!()
    }

    fn end(&self) -> bool {
        self.counter == self.tokens.len()
    }
}
