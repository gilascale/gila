use std::vec;

use crate::{
    ast::{ASTNode, Op, Statement},
    lex::{Position, Token, Type},
    r#type::DataType,
};

macro_rules! consume_token {
    ($self:expr, $expected:expr) => {
        if $self.tokens[$self.counter].typ != $expected {
            panic!(
                "Unexpected token: expected {:?}, found {:?}",
                $expected, $self.tokens[$self.counter].typ
            );
        }
        $self.counter += 1;
    };
}

macro_rules! get_position {
    ($self:expr) => {
        $self.tokens[$self.counter].pos.clone()
    };
}

macro_rules! get_next {
    ($self:expr) => {{
        let t = &$self.tokens[$self.counter];
        $self.counter += 1;
        t
    }};
}

macro_rules! get_current {
    ($self:expr) => {
        &$self.tokens[$self.counter];
    };
}

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
            Type::ASSERT => self.assert(),
            Type::DO => self.block(),
            Type::TEST => self.test(),
            Type::IF => self.iff(),
            Type::FOR => self.forr(),
            Type::RETURN => self.ret(),
            Type::IDENTIFIER(_) => self.identifier(),
            _ => self.expression(),
        }
    }

    fn expression(&mut self) -> ASTNode {
        let higher_precedence = self.import();
        let lhs_pos = higher_precedence.position.clone();

        if !self.end() && self.tokens[self.counter].typ == Type::ASSIGN {
            consume_token!(self, Type::ASSIGN);
            let rhs = self.expression();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::ASSIGN(Box::new(higher_precedence), Box::new(rhs)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        higher_precedence
    }

    fn import(&mut self) -> ASTNode {
        if self.tokens[self.counter].typ == Type::IMPORT {
            let lhs_pos = get_position!(self);
            consume_token!(self, Type::IMPORT);
            // todo parse module path properly
            let mut tokens: Vec<Token> = vec![];
            loop {
                let t = get_next!(self);
                tokens.push(t.clone());
                if self.tokens[self.counter].typ == Type::DOT {
                    consume_token!(self, Type::DOT);
                } else {
                    break;
                }
            }
            let first = &tokens.clone()[0];
            return ASTNode {
                statement: Statement::IMPORT(tokens),
                position: lhs_pos.join(first.pos.clone()),
            };
        }

        return self.logical_operators();
    }

    fn tryy(&mut self) -> ASTNode {
        if self.tokens[self.counter].typ == Type::EXCLAIM {
            let lhs_pos = get_position!(self);
            consume_token!(self, Type::EXCLAIM);
            // fixme should this be self.expression()
            let rhs = self.call();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::TRY(Box::new(rhs)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        self.call()
    }

    fn call(&mut self) -> ASTNode {
        let lhs_pos = get_position!(self);
        let higher_precedence = self.index();

        if !self.end() && self.tokens[self.counter].typ == Type::LPAREN {
            consume_token!(self, Type::LPAREN);
            let mut args: Vec<ASTNode> = vec![];
            let mut rhs_pos: Position;
            if self.tokens[self.counter].typ != Type::RPAREN {
                loop {
                    args.push(self.expression());
                    if self.tokens[self.counter].typ == Type::RPAREN {
                        rhs_pos = get_position!(self);
                        consume_token!(self, Type::RPAREN);
                        break;
                    }
                    consume_token!(self, Type::COMMA);
                }
            } else {
                rhs_pos = get_position!(self);
                get_next!(self);
            }
            return ASTNode {
                statement: Statement::CALL(Box::new(higher_precedence), args),
                position: lhs_pos.join(rhs_pos),
            };
        }

        higher_precedence
    }

    fn index(&mut self) -> ASTNode {
        let lhs_pos = get_position!(self);
        let higher_precedence = self.struct_access();
        if !self.end() && self.tokens[self.counter].typ == Type::LSQUARE {
            consume_token!(self, Type::LSQUARE);
            let the_index = self.struct_access();
            let rhs_pos = self.tokens[self.counter].pos.clone();
            consume_token!(self, Type::RSQUARE);
            return ASTNode {
                statement: Statement::INDEX(Box::new(higher_precedence), Box::new(the_index)),
                position: lhs_pos.join(rhs_pos),
            };
        }
        higher_precedence
    }

    fn struct_access(&mut self) -> ASTNode {
        // Parse the leftmost expression first (e.g., x in x.y.z)
        let lhs_pos = get_position!(self);
        let mut lhs = self.single();
        // Keep parsing as long as there's a DOT token followed by an identifier
        while !self.end() && self.tokens[self.counter].typ == Type::DOT {
            consume_token!(self, Type::DOT);
            let rhs_pos = get_position!(self);
            let field = get_next!(self); // Field being accessed
                                         // Create a new STRUCT_ACCESS node for this level
            lhs = ASTNode {
                statement: Statement::STRUCT_ACCESS(Box::new(lhs), field.clone()),
                position: lhs_pos.join(rhs_pos),
            };
        }
        lhs
    }

    fn single(&mut self) -> ASTNode {
        let next: &Token = &self.tokens[self.counter];
        match (next.typ) {
            Type::TRUE => {
                consume_token!(self, Type::TRUE);
                return ASTNode {
                    statement: Statement::LITERAL_BOOL(true),
                    position: next.pos.clone(),
                };
            }
            Type::FALSE => {
                consume_token!(self, Type::FALSE);
                return ASTNode {
                    statement: Statement::LITERAL_BOOL(false),
                    position: next.pos.clone(),
                };
            }
            Type::STRING_LITERAL(_) => self.string(),
            Type::ATOM(_) => self.atom(),
            Type::IDENTIFIER(_) => {
                get_next!(self);
                return ASTNode {
                    statement: Statement::VARIABLE(next.clone()),
                    position: next.pos.clone(),
                };
            }
            Type::NUMBER(_) => {
                get_next!(self);
                return ASTNode {
                    statement: Statement::LITERAL_NUM(next.clone()),
                    position: next.pos.clone(),
                };
            }
            Type::LSQUARE => {
                let lhs_pos = get_position!(self);
                consume_token!(self, Type::LSQUARE);
                let mut items: Vec<ASTNode> = vec![];
                if self.tokens[self.counter].typ != Type::RSQUARE {
                    loop {
                        items.push(self.expression());
                        if self.tokens[self.counter].typ == Type::RSQUARE {
                            break;
                        }
                        consume_token!(self, Type::COMMA);
                    }
                }
                let rhs_pos = get_position!(self);
                consume_token!(self, Type::RSQUARE);
                return ASTNode {
                    statement: Statement::SLICE(items),
                    position: lhs_pos.join(rhs_pos),
                };
            }
            Type::COLON => {
                let lhs_pos = get_position!(self);
                consume_token!(self, Type::COLON);
                let rhs_pos = get_position!(self);
                let atom = get_next!(self);
                return ASTNode {
                    statement: Statement::ATOM(atom.clone()),
                    position: lhs_pos.join(rhs_pos),
                };
            }
            Type::AMPERSAND => {
                // doing annotation
                let lhs_pos = get_position!(self);
                consume_token!(self, Type::AMPERSAND);
                let annotation = get_next!(self);
                let mut args: Vec<Token> = vec![];
                if self.tokens[self.counter].typ == Type::LPAREN {
                    consume_token!(self, Type::LPAREN);
                    if self.tokens[self.counter].typ != Type::RPAREN {
                        loop {
                            let next = get_next!(self);
                            args.push(next.clone());
                            if self.tokens[self.counter].typ == Type::RPAREN {
                                consume_token!(self, Type::RPAREN);
                                break;
                            }
                            consume_token!(self, Type::COMMA);
                        }
                    } else {
                        get_next!(self);
                    }
                }

                // FIXME should probably do this with statements...
                let expr = self.expression();
                let rhs_pos = expr.position.clone();

                return ASTNode {
                    statement: Statement::ANNOTATION(annotation.clone(), args, Box::new(expr)),
                    position: lhs_pos.join(rhs_pos),
                };
            }
            // _ => higher_precedence,
            _ => panic!("{:?}", next),
        }
    }

    fn logical_operators(&mut self) -> ASTNode {
        let higher_precedence = self.equality();

        if !self.end() && self.tokens[self.counter].typ == Type::OR {
            consume_token!(self, Type::OR);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(
                    Box::new(higher_precedence),
                    Box::new(rhs),
                    Op::LOGICAL_OR,
                ),
                position: pos,
            };
        }

        higher_precedence
    }

    fn equality(&mut self) -> ASTNode {
        let higher_precedence = self.add_sub();
        if !self.end() && self.tokens[self.counter].typ == Type::EQUALS {
            consume_token!(self, Type::EQUALS);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::EQ),
                position: pos,
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::NOT_EQUALS {
            consume_token!(self, Type::NOT_EQUALS);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::NEQ),
                position: pos,
            };
        } else if !self.end() && self.tokens[self.counter].typ == Type::GREATER_THAN {
            consume_token!(self, Type::GREATER_THAN);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::GT),
                position: pos,
            };
        }
        if !self.end() && self.tokens[self.counter].typ == Type::GREATER_EQ {
            consume_token!(self, Type::GREATER_EQ);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::GE),
                position: pos,
            };
        }
        if !self.end() && self.tokens[self.counter].typ == Type::LESS_THAN {
            consume_token!(self, Type::LESS_THAN);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::LT),
                position: pos,
            };
        }
        if !self.end() && self.tokens[self.counter].typ == Type::LESS_EQ {
            consume_token!(self, Type::LESS_EQ);
            let rhs = self.expression();
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(Box::new(higher_precedence), Box::new(rhs), Op::LE),
                position: pos,
            };
        }

        higher_precedence
    }

    fn add_sub(&mut self) -> ASTNode {
        let higher_precedence = self.mul_div();
        if !self.end() && self.tokens[self.counter].typ == Type::ADD {
            consume_token!(self, Type::ADD);
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
            consume_token!(self, Type::SUB);
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
        let higher_precedence = self.tryy();
        if !self.end() && self.tokens[self.counter].typ == Type::MUL {
            consume_token!(self, Type::MUL);
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
            consume_token!(self, Type::DIV);
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

    fn assert(&mut self) -> ASTNode {
        let assert_pos = get_position!(self);
        consume_token!(self, Type::ASSERT);
        let expr = self.expression();
        let mut name: Option<Token> = None;
        let mut rhs_pos = expr.position.clone();
        if !self.end() && self.tokens[self.counter].typ == Type::COMMA {
            consume_token!(self, Type::COMMA);
            let next = get_next!(self);
            name = Some(next.clone());
            rhs_pos = name.as_ref().unwrap().pos.clone();
        }
        return ASTNode {
            statement: Statement::ASSERT(Box::new(expr), name),
            position: assert_pos.join(rhs_pos),
        };
    }

    fn block(&mut self) -> ASTNode {
        let do_pos = get_position!(self);
        consume_token!(self, Type::DO);
        let mut stms = vec![];
        while !self.end() && self.tokens[self.counter].typ != Type::END {
            stms.push(self.statement());
        }
        let end_pos: Position;
        if stms.len() > 0 {
            end_pos = stms[stms.len() - 1].position.clone();
        } else {
            end_pos = self.tokens[self.counter - 1].pos.clone();
        }
        consume_token!(self, Type::END);
        return ASTNode {
            statement: Statement::BLOCK(stms),
            position: do_pos.join(end_pos),
        };
    }

    fn test(&mut self) -> ASTNode {
        let test_pos = get_position!(self);
        consume_token!(self, Type::TEST);
        let test_name = self.string();
        let body = self.statement();
        let body_pos = body.position.clone();
        ASTNode {
            statement: Statement::TEST(Box::new(test_name), Box::new(body)),
            position: test_pos.join(body_pos),
        }
    }

    fn iff(&mut self) -> ASTNode {
        let if_pos = get_position!(self);
        consume_token!(self, Type::IF);
        let condition = self.expression();
        let body = self.statement();
        let body_pos = body.position.clone();
        let mut else_body: Option<Box<ASTNode>> = None;
        if !self.end() && self.tokens[self.counter].typ == Type::ELSE {
            consume_token!(self, Type::ELSE);
            else_body = Some(Box::new(self.statement()));
        }
        ASTNode {
            statement: Statement::IF(Box::new(condition), Box::new(body), else_body),
            position: if_pos.join(body_pos),
        }
    }

    fn forr(&mut self) -> ASTNode {
        let for_pos = get_position!(self);
        consume_token!(self, Type::FOR);
        let var = get_next!(self);
        consume_token!(self, Type::IN);
        let range_start = get_next!(self);
        // consume the ..
        consume_token!(self, Type::DOT);
        consume_token!(self, Type::DOT);
        let range_end = get_next!(self);
        let body = self.statement();
        let body_pos = body.position.clone();
        return ASTNode {
            statement: Statement::FOR(
                var.clone(),
                range_start.clone(),
                range_end.clone(),
                Box::new(body),
            ),
            position: for_pos.join(body_pos.clone()),
        };
    }

    fn parse_range(&mut self) {}

    fn ret(&mut self) -> ASTNode {
        let pos = get_position!(self);
        consume_token!(self, Type::RETURN);
        let val = self.expression();
        let rhs_pos = val.position.clone();
        return ASTNode {
            statement: Statement::RETURN(Some(Box::new(val))),
            position: pos.clone().join(rhs_pos),
        };
    }

    fn string(&mut self) -> ASTNode {
        let s = get_next!(self);
        ASTNode {
            statement: Statement::STRING(s.clone()),
            position: s.pos.clone(),
        }
    }

    fn atom(&mut self) -> ASTNode {
        let pos = get_position!(self);
        let tok = get_next!(self);
        return ASTNode {
            statement: Statement::ATOM(tok.clone()),
            position: pos,
        };
    }

    fn identifier(&mut self) -> ASTNode {
        let identifier = get_current!(self);
        if self.end_away(1) {
            get_next!(self);
            return ASTNode {
                statement: Statement::VARIABLE(identifier.clone()),
                position: identifier.pos.clone(),
            };
        }

        if self.tokens[self.counter + 1].typ == Type::COLON {
            get_next!(self);
            let lhs_pos = identifier.pos.clone();
            // move over the :
            consume_token!(self, Type::COLON);
            let typ = self.parse_type();
            // move over the =
            consume_token!(self, Type::ASSIGN);
            let rhs = self.expression();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), Some(typ), Some(Box::new(rhs))),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // function
        if self.tokens[self.counter + 1].typ == Type::FN {
            get_next!(self);
            let lhs_pos = identifier.pos.clone();
            consume_token!(self, Type::FN);

            let mut params: Vec<ASTNode> = vec![];

            if self.tokens[self.counter].typ == Type::LPAREN {
                consume_token!(self, Type::LPAREN);
                if self.tokens[self.counter].typ != Type::RPAREN {
                    loop {
                        params.push(self.parse_decl());
                        if self.tokens[self.counter].typ == Type::RPAREN {
                            consume_token!(self, Type::RPAREN);
                            break;
                        }
                        consume_token!(self, Type::COMMA);
                    }
                } else {
                    get_next!(self);
                }
            }

            let mut return_type: Option<DataType> = None;
            if self.tokens[self.counter].typ == Type::SUB {
                consume_token!(self, Type::SUB);
                if self.tokens[self.counter].typ == Type::GREATER_THAN {
                    consume_token!(self, Type::GREATER_THAN);
                    return_type = Some(self.parse_type());
                }
            }

            let rhs = self.statement();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::NAMED_FUNCTION(
                    identifier.clone(),
                    params,
                    return_type,
                    Box::new(rhs),
                ),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // type
        // todo deal with blocks?
        if self.tokens[self.counter + 1].typ == Type::TYPE {
            get_next!(self);
            let lhs_pos = identifier.pos.clone();
            consume_token!(self, Type::TYPE);
            let mut decls: Vec<ASTNode> = vec![];
            let mut rhs_pos: Position;
            if self.tokens[self.counter].typ != Type::END {
                loop {
                    decls.push(self.parse_decl());
                    if self.tokens[self.counter].typ == Type::END {
                        rhs_pos = get_position!(self);
                        consume_token!(self, Type::END);
                        break;
                    }
                }
            } else {
                rhs_pos = get_position!(self);
                get_next!(self);
            }

            return ASTNode {
                statement: Statement::NAMED_TYPE_DECL(identifier.clone(), decls),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // fixme this should be lower precedence
        if self.tokens[self.counter + 1].typ == Type::ASSIGN {
            get_next!(self);
            let lhs_pos = identifier.pos.clone();
            consume_token!(self, Type::ASSIGN);
            let rhs = self.expression();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), None, Some(Box::new(rhs))),
                position: lhs_pos.join(rhs_pos),
            };
        }

        self.expression()
    }

    fn parse_type(&mut self) -> DataType {
        let current = get_next!(self);
        let mut t: DataType;
        match &current.typ {
            Type::DOLLAR => {
                let next = get_next!(self);
                return match &next.typ {
                    Type::IDENTIFIER(i) => DataType::GENERIC(i.clone()),
                    _ => panic!(),
                };
            }
            Type::ANY => t = DataType::ANY,
            Type::STRING => t = DataType::STRING,
            Type::BOOL => t = DataType::BOOL,
            Type::U32 => t = DataType::U32,
            Type::IDENTIFIER(i) => t = DataType::NAMED_REFERENCE(i.clone()),
            _ => panic!(),
        }
        if self.tokens[self.counter].typ == Type::LSQUARE {
            consume_token!(self, Type::LSQUARE);
            consume_token!(self, Type::RSQUARE);
            return DataType::SLICE(Box::new(t));
        }
        t
    }

    fn parse_decl(&mut self) -> ASTNode {
        let lhs_pos = get_position!(self);
        let identifier = get_next!(self);
        consume_token!(self, Type::COLON);
        let rhs_pos = self.tokens[self.counter].pos.clone();
        let typ = self.parse_type();
        return ASTNode {
            statement: Statement::DEFINE(identifier.clone(), Some(typ), None),
            position: lhs_pos.join(rhs_pos),
        };
    }

    fn end(&self) -> bool {
        self.counter == self.tokens.len()
    }

    fn end_away(&self, offset: usize) -> bool {
        self.counter + offset == self.tokens.len()
    }
}
