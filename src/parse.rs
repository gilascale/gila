use crate::{
    ast::{ASTNode, Op, Statement},
    lex::{Position, Token, Type},
    r#type::DataType,
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
            // todo
            self.counter += 1;
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
            let lhs = self.tokens[self.counter].pos.clone();
            self.counter += 1;
            // todo parse module path properly
            let t = &self.tokens[self.counter];
            self.counter += 1;
            return ASTNode {
                statement: Statement::IMPORT(t.clone()),
                position: lhs.join(t.pos.clone()),
            };
        }

        return self.logical_operators();
    }

    fn tryy(&mut self) -> ASTNode {
        if self.tokens[self.counter].typ == Type::EXCLAIM {
            let lhs_pos = self.tokens[self.counter].pos.clone();
            self.counter += 1;
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
        let higher_precedence = self.index();

        if !self.end() && self.tokens[self.counter].typ == Type::LPAREN {
            self.counter += 1;
            let lhs_pos = higher_precedence.position.clone();

            let mut args: Vec<ASTNode> = vec![];

            let mut rhs_pos: Position;
            if self.tokens[self.counter].typ != Type::RPAREN {
                loop {
                    args.push(self.expression());
                    if self.tokens[self.counter].typ == Type::RPAREN {
                        rhs_pos = self.tokens[self.counter].pos.clone();
                        self.counter += 1;
                        break;
                    }
                    // skip ,
                    self.counter += 1;
                }
            } else {
                rhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
            }

            return ASTNode {
                statement: Statement::CALL(Box::new(higher_precedence), args),
                position: lhs_pos.join(rhs_pos),
            };
        }

        higher_precedence
    }

    fn index(&mut self) -> ASTNode {
        let higher_precedence = self.struct_access();
        if !self.end() && self.tokens[self.counter].typ == Type::LSQUARE {
            let lhs_pos = higher_precedence.position.clone();
            // consume [
            self.counter += 1;

            let the_index = self.struct_access();

            let rhs_pos = self.tokens[self.counter].pos.clone();
            // consume ]
            self.counter += 1;

            return ASTNode {
                statement: Statement::INDEX(Box::new(higher_precedence), Box::new(the_index)),
                position: lhs_pos.join(rhs_pos),
            };
        }
        higher_precedence
    }

    // fixme we are not parsing this correctly
    fn struct_access(&mut self) -> ASTNode {
        // Parse the leftmost expression first (e.g., x in x.y.z)
        let mut lhs = self.single();

        // Keep parsing as long as there's a DOT token followed by an identifier
        while !self.end() && self.tokens[self.counter].typ == Type::DOT {
            let lhs_pos = lhs.position.clone(); // Position of the left-hand side
            self.counter += 1; // Consume the DOT token

            // Ensure there is an identifier after the DOT
            // if self.end() || self.tokens[self.counter].typ != Type::IDENTIFIER(_) {
            //     panic!("Expected identifier after '.'");
            // }

            let field = self.tokens[self.counter].clone(); // Field being accessed
            let rhs_pos = field.pos.clone(); // Position of the field
            self.counter += 1; // Consume the identifier token

            // Create a new STRUCT_ACCESS node for this level
            lhs = ASTNode {
                statement: Statement::STRUCT_ACCESS(Box::new(lhs), field),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // Return the fully parsed access chain
        lhs
    }

    fn single(&mut self) -> ASTNode {
        let next: &Token = &self.tokens[self.counter];
        match (next.typ) {
            Type::TRUE => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_BOOL(true),
                    position: next.pos.clone(),
                };
            }
            Type::FALSE => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_BOOL(false),
                    position: next.pos.clone(),
                };
            }
            Type::STRING(_) => self.string(),
            Type::ATOM(_) => self.atom(),
            Type::IDENTIFIER(_) => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::VARIABLE(next.clone()),
                    position: next.pos.clone(),
                };
            }
            Type::NUMBER(_) => {
                self.counter += 1;
                return ASTNode {
                    statement: Statement::LITERAL_NUM(next.clone()),
                    position: next.pos.clone(),
                };
            }
            Type::LSQUARE => {
                let lhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
                let mut items: Vec<ASTNode> = vec![];
                if self.tokens[self.counter].typ != Type::RSQUARE {
                    loop {
                        items.push(self.expression());
                        if self.tokens[self.counter].typ == Type::RSQUARE {
                            break;
                        }
                        // loop over the ,
                        self.counter += 1;
                    }
                }
                let rhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
                return ASTNode {
                    statement: Statement::SLICE(items),
                    position: lhs_pos.join(rhs_pos),
                };
            }
            Type::COLON => {
                let lhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
                let atom = self.tokens[self.counter].clone();
                let rhs_pos = atom.pos.clone();
                self.counter += 1;
                return ASTNode {
                    statement: Statement::ATOM(atom),
                    position: lhs_pos.join(rhs_pos),
                };
            }
            Type::AMPERSAND => {
                // doing annotation
                let lhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
                let annotation = self.tokens[self.counter].clone();
                self.counter += 1;

                let mut args: Vec<Token> = vec![];
                if self.tokens[self.counter].typ == Type::LPAREN {
                    self.counter += 1;
                    if self.tokens[self.counter].typ != Type::RPAREN {
                        loop {
                            args.push(self.tokens[self.counter].clone());
                            self.counter += 1;

                            if self.tokens[self.counter].typ == Type::RPAREN {
                                self.counter += 1;
                                break;
                            }
                            // consume ,
                            self.counter += 1;
                        }
                    } else {
                        self.counter += 1;
                    }
                }

                // FIXME should probably do this with statements...
                let expr = self.expression();
                let rhs_pos = expr.position.clone();

                return ASTNode {
                    statement: Statement::ANNOTATION(annotation, args, Box::new(expr)),
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
            self.counter += 1;
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
            self.counter += 1;
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
            self.counter += 1;
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
            self.counter += 1;
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
            self.counter += 1;
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
            self.counter += 1;
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
            self.counter += 1;
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
        let higher_precedence = self.tryy();
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
        // consume the END
        self.counter += 1;
        return ASTNode {
            statement: Statement::BLOCK(stms),
            // todo this will error if block is empty
            position: start_pos.join(end_pos),
        };
    }

    fn test(&mut self) -> ASTNode {
        let test_pos = self.tokens[self.counter].pos.clone();
        self.counter += 1;

        let test_name = self.string();
        let body = self.statement();
        let body_pos = body.position.clone();

        // consume end
        // fixme this needs to be done properly
        // because right now we can't do else if
        let end_pos = self.tokens[self.counter].pos.clone();
        self.counter += 1;

        ASTNode {
            statement: Statement::TEST(Box::new(test_name), Box::new(body)),
            position: test_pos.join(end_pos),
        }
    }

    fn iff(&mut self) -> ASTNode {
        let if_pos = self.tokens[self.counter].pos.clone();
        self.counter += 1;

        let condition = self.expression();
        let body = self.statement();
        let body_pos = body.position.clone();

        let mut else_body: Option<Box<ASTNode>> = None;
        if self.tokens[self.counter].typ == Type::ELSE {
            self.counter += 1;
            else_body = Some(Box::new(self.statement()));
        }
        // consume end
        // fixme this needs to be done properly
        // because right now we can't do else if
        self.counter += 1;

        ASTNode {
            statement: Statement::IF(Box::new(condition), Box::new(body), else_body),
            position: if_pos.join(body_pos),
        }
    }

    fn forr(&mut self) -> ASTNode {
        let if_pos = self.tokens[self.counter].pos.clone();
        self.counter += 1;

        let condition = self.expression();
        let body = self.statement();
        let body_pos = body.position.clone();

        let mut else_body: Option<Box<ASTNode>> = None;
        if self.tokens[self.counter].typ == Type::ELSE {
            self.counter += 1;
            else_body = Some(Box::new(self.statement()));
        }
        // consume end
        // fixme this needs to be done properly
        // because right now we can't do else if
        self.counter += 1;

        ASTNode {
            statement: Statement::IF(Box::new(condition), Box::new(body), else_body),
            position: if_pos.join(body_pos),
        }
    }

    // fn lett(&mut self) -> ASTNode {
    //     let let_pos = self.tokens[self.counter].pos.clone();
    //     self.counter += 1;

    //     let identifier = &self.tokens[self.counter];

    //     // consume identifier and =
    //     self.counter += 1;
    //     self.counter += 1;

    //     let value = self.expression();
    //     let value_pos = value.position.clone();

    //     // fixme add type
    //     ASTNode {
    //         statement: Statement::DEFINE(identifier.clone(), Box::new(value)),
    //         position: let_pos.join(value_pos),
    //     }
    // }

    fn ret(&mut self) -> ASTNode {
        let pos = &self.tokens[self.counter].pos;
        self.counter += 1;
        let val = self.expression();
        let rhs_pos = val.position.clone();
        return ASTNode {
            statement: Statement::RETURN(Some(Box::new(val))),
            position: pos.clone().join(rhs_pos),
        };
    }

    fn string(&mut self) -> ASTNode {
        let s = &self.tokens[self.counter];
        self.counter += 1;
        ASTNode {
            statement: Statement::STRING(s.clone()),
            position: s.pos.clone(),
        }
    }

    fn atom(&mut self) -> ASTNode {
        let tok = &self.tokens[self.counter];
        let pos = tok.pos.clone();
        self.counter += 1;
        return ASTNode {
            statement: Statement::ATOM(tok.clone()),
            position: pos,
        };
    }

    fn identifier(&mut self) -> ASTNode {
        // todo assume define?

        let identifier = &self.tokens[self.counter];
        // self.counter += 1;

        if self.end_away(1) {
            self.counter += 1;
            return ASTNode {
                statement: Statement::VARIABLE(identifier.clone()),
                position: identifier.pos.clone(),
            };
        }

        if self.tokens[self.counter + 1].typ == Type::COLON {
            self.counter += 1;
            let lhs_pos = identifier.pos.clone();
            // move over the :
            self.counter += 1;
            let typ = self.parse_type();
            // move over the =
            self.counter += 1;
            let rhs = self.expression();
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), Some(typ), Some(Box::new(rhs))),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // function
        // todo deal with blocks?
        if self.tokens[self.counter + 1].typ == Type::FN {
            self.counter += 1;
            let lhs_pos = identifier.pos.clone();
            // move over the fn
            self.counter += 1;

            let mut params: Vec<ASTNode> = vec![];

            if self.tokens[self.counter].typ == Type::LPAREN {
                self.counter += 1;
                if self.tokens[self.counter].typ != Type::RPAREN {
                    loop {
                        // fixme do proper decl
                        params.push(self.parse_decl());
                        // params.push(ASTNode {
                        //     statement: Statement::VARIABLE(self.tokens[self.counter].clone()),
                        //     position: self.tokens[self.counter].pos.clone(),
                        // });
                        // self.counter += 1;

                        if self.tokens[self.counter].typ == Type::RPAREN {
                            self.counter += 1;
                            break;
                        }
                        // skip ,
                        self.counter += 1;
                    }
                } else {
                    self.counter += 1;
                }
            }

            let mut return_type: Option<DataType> = None;
            if self.tokens[self.counter].typ == Type::SUB {
                self.counter += 1;
                if self.tokens[self.counter].typ == Type::GREATER_THAN {
                    self.counter += 1;
                    //consume type
                    return_type = Some(self.parse_type());
                }
            }

            let rhs = self.block();
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
            self.counter += 1;
            let lhs_pos = identifier.pos.clone();
            // move over the type
            self.counter += 1;

            let mut decls: Vec<ASTNode> = vec![];

            let mut rhs_pos: Position;
            if self.tokens[self.counter].typ != Type::END {
                loop {
                    // consume types

                    decls.push(self.parse_decl());

                    if self.tokens[self.counter].typ == Type::END {
                        rhs_pos = self.tokens[self.counter].pos.clone();
                        self.counter += 1;
                        break;
                    }
                }
            } else {
                rhs_pos = self.tokens[self.counter].pos.clone();
                self.counter += 1;
            }

            return ASTNode {
                statement: Statement::NAMED_TYPE_DECL(identifier.clone(), decls),
                position: lhs_pos.join(rhs_pos),
            };
        }

        // fixme this should be lower precedence
        if self.tokens[self.counter + 1].typ == Type::ASSIGN {
            self.counter += 1;
            let lhs_pos = identifier.pos.clone();
            // move over the =
            self.counter += 1;
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
        let current = &self.tokens[self.counter];
        self.counter += 1;
        let mut t: DataType;
        match &current.typ {
            Type::U32 => t = DataType::U32,
            Type::IDENTIFIER(i) => t = DataType::DYNAMIC_OBJECT(i.clone()),
            _ => panic!("umm what is {:?}", current.typ),
        }
        if self.tokens[self.counter].typ == Type::LSQUARE {
            // todo do this properly - consume the []
            self.counter += 2;
            return DataType::SLICE(Box::new(t));
        }
        t
    }

    fn parse_decl(&mut self) -> ASTNode {
        let identifier = &self.tokens[self.counter];
        let lhs_pos = identifier.pos.clone();
        // consume identifier
        self.counter += 1;
        // consume :
        self.counter += 1;
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
