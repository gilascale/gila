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
            Type::IF => self.iff(),
            Type::RETURN => self.ret(),
            // Type::IDENTIFIER(_) => self.identifier(),
            _ => self.expression(),
        }
    }

    fn expression(&mut self) -> ASTNode {
        // let higher_prece

        return self.add_sub();
    }

    fn call(&mut self) -> ASTNode {
        let higher_precedence = self.single();

        if !self.end() && self.tokens[self.counter].typ == Type::LPAREN {
            self.counter += 1;
            let lhs_pos = higher_precedence.position.clone();
            let rhs_pos = self.tokens[self.counter].pos.clone();
            self.counter += 1;

            return ASTNode {
                statement: Statement::CALL(Box::new(higher_precedence)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        higher_precedence
    }

    fn single(&mut self) -> ASTNode {
        let next: &Token = &self.tokens[self.counter];
        match (next.typ) {
            Type::IDENTIFIER(_) => self.identifier(),
            Type::NUMBER(_) => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_NUM(next.clone()),
                    position: next.pos.clone(),
                };
            }
            // _ => higher_precedence,
            _ => panic!("{:?}", next),
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
        let higher_precedence = self.call();
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
        let start_pos: Position;
        let end_pos: Position;
        if stms.len() > 0 {
            start_pos = stms[0].position.clone();
            end_pos = stms[stms.len() - 1].position.clone();
        } else {
            // todo this is a hack so if the block is empty we don't crash
            start_pos = self.tokens[self.counter - 1].pos.clone();
            end_pos = self.tokens[self.counter - 1].pos.clone();
        }
        self.counter += 1;
        return ASTNode {
            statement: Statement::BLOCK(stms),
            // todo this will error if block is empty
            position: start_pos.join(end_pos),
        };
    }

    fn iff(&mut self) -> ASTNode {
        let if_pos = self.tokens[self.counter].pos.clone();
        self.counter += 1;

        let condition = self.expression();
        println!("parsed if condition {:?}", condition);
        let body = self.statement();
        println!("parsed if body {:?}", body);
        let body_pos = body.position.clone();

        // consume end
        self.counter += 1;

        ASTNode {
            statement: Statement::IF(Box::new(condition), Box::new(body)),
            position: if_pos.join(body_pos),
        }
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

        // fixme this should be lower precedence
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

        ASTNode {
            statement: Statement::VARIABLE(identifier.clone()),
            position: identifier.pos.clone(),
        }
    }

    fn end(&self) -> bool {
        self.counter == self.tokens.len()
    }
}
