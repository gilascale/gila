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
            position: Position {
                index: 0,
                line: 0,
                index_end: 0,
                line_end: 0,
            },
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
        let next: &Token = &self.tokens[self.counter];
        match (next.typ) {
            Type::NUMBER(_) => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_NUM(next.clone()),
                    position: next.pos.clone(),
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
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::ADD),
                position: pos,
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::SUB {
            self.counter += 1;
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::SUB),
                position: pos,
            };
        }
        return higher_precedence;
    }

    fn mul_div(&mut self) -> ASTNode {
        let higher_precedence = self.single();
        if !self.end() && self.tokens[self.counter].typ == Type::MUL {
            self.counter += 1;
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::MUL),
                position: pos,
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::DIV {
            self.counter += 1;
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::DIV),
                position: pos,
            };
        }
        return higher_precedence;
    }

    fn block(&mut self) -> ASTNode {
        // todo get the block start token somehow?
        let mut stms = vec![];
        while !self.end() && self.tokens[self.counter].typ != Type::END {
            stms.push(self.statement());
        }
        let start_pos = stms[0].position.clone();
        let end_pos = stms[stms.len() - 1].position.clone();
        self.counter += 1;
        return ASTNode {
            statement: Statement::BLOCK(stms),
            // todo this will error if block is empty
            position: start_pos.join(end_pos),
        };
    }

    fn ret(&mut self) -> ASTNode {
        let pos = &self.tokens[self.counter].pos;
        self.counter += 1;
        return ASTNode {
            statement: Statement::RETURN(None),
            position: pos.clone(),
        };
    }

    fn identifier(&mut self) -> ASTNode {
        // todo assume define?

        let identifier = &self.tokens[self.counter];
        self.counter += 1;

        if self.end() {
            return ASTNode {
                statement: Statement::VARIABLE(identifier.clone()),
                position: identifier.pos.clone(),
            };
        }

        // function
        // todo deal with blocks?
        if self.tokens[self.counter].typ == Type::FN {
            let lhs_pos = self.tokens[self.counter].pos.clone();
            self.counter += 1;
            let rhs = self.block();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::NAMED_FUNCTION(identifier.clone(), Box::new(rhs)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        if self.tokens[self.counter].typ == Type::ASSIGN {
            let lhs_pos = self.tokens[self.counter].pos.clone();
            self.counter += 1;
            let rhs = self.expression();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), Box::new(self.expression())),
                position: lhs_pos.join(rhs_pos),
            };
        }

        panic!()
    }

    fn end(&self) -> bool {
        self.counter == self.tokens.len()
    }
}
