use std::thread::current;

use crate::{
    ast::{Expression, Op, Statement},
    lex::Token,
    lex::Type,
};

pub struct Parser<'a> {
    pub tokens: &'a std::vec::Vec<Token>,
    pub counter: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Statement {
        let mut program = vec![];

        while !self.end() {
            program.push(self.statement());
        }
        // Statement::PROGRAM(vec![Statement::EXPRESSION(Expression::BIN_OP(
        //     Box::new(Expression::LITERAL_NUM(1.0)),
        //     Box::new(Expression::LITERAL_NUM(1.0)),
        //     Op::ADD,
        // ))])
        Statement::PROGRAM(program)
    }

    fn statement(&mut self) -> Statement {
        let current: &Token = &self.tokens[self.counter];

        match current.typ {
            Type::RETURN => self.ret(),
            Type::IDENTIFIER(_) => self.identifier(),
            _ => Statement::EXPRESSION(self.expression()),
        }
    }

    fn expression(&mut self) -> Expression {
        // let higher_prece

        return self.add_sub();
    }

    fn single(&mut self) -> Expression {
        let next = &self.tokens[self.counter];
        match (next.typ) {
            Type::NUMBER(_) => {
                self.counter += 1;
                return Expression::LITERAL_NUM(next.clone());
            }
            // _ => higher_precedence,
            _ => panic!(),
        }
    }

    fn add_sub(&mut self) -> Expression {
        let higher_precedence = self.mul_div();
        if !self.end() && self.tokens[self.counter].typ == Type::ADD {
            self.counter += 1;
            let rhs = self.expression();
            return Expression::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::ADD);
        } else if !self.end() && self.tokens[self.counter].typ == Type::SUB {
            self.counter += 1;
            let rhs = self.expression();
            return Expression::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::SUB);
        }
        return higher_precedence;
    }

    fn mul_div(&mut self) -> Expression {
        let higher_precedence = self.single();
        if !self.end() && self.tokens[self.counter].typ == Type::MUL {
            self.counter += 1;
            let rhs = self.expression();
            return Expression::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::MUL);
        } else if !self.end() && self.tokens[self.counter].typ == Type::DIV {
            self.counter += 1;
            let rhs = self.expression();
            return Expression::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::DIV);
        }
        return higher_precedence;
    }

    fn block(&mut self) -> Statement {
        let mut stms = vec![];
        while !self.end() && self.tokens[self.counter].typ != Type::END {
            stms.push(self.statement());
        }
        self.counter += 1;
        return Statement::BLOCK(stms);
    }

    fn ret(&mut self) -> Statement {
        self.counter += 1;
        return Statement::RETURN(None);
    }

    fn identifier(&mut self) -> Statement {
        // todo assume define?

        let identifier = &self.tokens[self.counter];
        self.counter += 1;

        if self.end() {
            return Statement::EXPRESSION(Expression::VARIABLE(identifier.clone()));
        }

        // function
        // todo deal with blocks?
        if self.tokens[self.counter].typ == Type::FN {
            self.counter += 1;
            return Statement::NAMED_FUNCTION(identifier.clone(), Box::new(self.block()));
        }

        if self.tokens[self.counter].typ == Type::ASSIGN {
            self.counter += 1;
            return Statement::DEFINE(identifier.clone(), self.expression());
        }

        panic!()
    }

    fn end(&self) -> bool {
        self.counter == self.tokens.len()
    }
}
