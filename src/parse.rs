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

// this is used so we can disambiguate certain contexts
//
// i.e. foo(1,2,3) may look like a call to 'foo' with a single tuple as the argument,
// we need to specify that we are in a call, and the arguments take precedence here.
#[derive(Clone, Copy)]
pub struct ParseContext {
    pub in_function_call: bool,
    pub in_group: bool,
}

impl ParseContext {
    pub fn new() -> Self {
        return ParseContext {
            in_function_call: false,
            in_group: false,
        };
    }
}

pub struct Parser<'a> {
    pub tokens: &'a std::vec::Vec<Token>,
    pub counter: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> ASTNode {
        let mut program: Vec<ASTNode> = vec![];

        let parse_context = ParseContext::new();
        while !self.end() {
            program.push(self.statement(parse_context));
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

    fn statement(&mut self, parse_context: ParseContext) -> ASTNode {
        let current: &Token = &self.tokens[self.counter];

        match current.typ {
            Type::MATCH => self.matchh(parse_context),
            Type::ASSERT => self.assert(parse_context),
            Type::DO => self.block(parse_context),
            Type::TEST => self.test(parse_context),
            Type::IF => self.iff(parse_context),
            Type::FOR => self.forr(parse_context),
            Type::RETURN => self.ret(parse_context),
            Type::IDENTIFIER(_) => self.identifier(parse_context),
            _ => self.expression(parse_context),
        }
    }

    fn expression(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.import(parse_context);
        let lhs_pos = higher_precedence.position.clone();

        if !self.end() && self.tokens[self.counter].typ == Type::ASSIGN {
            consume_token!(self, Type::ASSIGN);
            let rhs = self.expression(parse_context);
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::ASSIGN(Box::new(higher_precedence), Box::new(rhs)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        higher_precedence
    }

    fn import(&mut self, parse_context: ParseContext) -> ASTNode {
        if self.tokens[self.counter].typ == Type::IMPORT {
            let lhs_pos = get_position!(self);
            consume_token!(self, Type::IMPORT);
            // todo parse module path properly
            let mut tokens: Vec<Token> = vec![];
            loop {
                let t = get_next!(self);
                tokens.push(t.clone());
                if !self.end() && self.tokens[self.counter].typ == Type::DOT {
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

        return self.parse_range(parse_context);
    }

    fn tryy(&mut self, parse_context: ParseContext) -> ASTNode {
        if self.tokens[self.counter].typ == Type::EXCLAIM {
            let lhs_pos = get_position!(self);
            consume_token!(self, Type::EXCLAIM);
            // fixme should this be self.expression()
            let rhs = self.call(parse_context);
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::TRY(Box::new(rhs)),
                position: lhs_pos.join(rhs_pos),
            };
        }

        self.call(parse_context)
    }

    fn call(&mut self, parse_context: ParseContext) -> ASTNode {
        let lhs_pos = get_position!(self);
        let higher_precedence = self.index(parse_context);

        if !self.end() && self.tokens[self.counter].typ == Type::LPAREN {
            consume_token!(self, Type::LPAREN);
            let mut args: Vec<ASTNode> = vec![];
            let mut rhs_pos: Position;
            if self.tokens[self.counter].typ != Type::RPAREN {
                let mut new_parse_context = parse_context.clone();
                new_parse_context.in_function_call = true;
                loop {
                    args.push(self.expression(new_parse_context));
                    if self.tokens[self.counter].typ == Type::RPAREN {
                        rhs_pos = get_position!(self);
                        consume_token!(self, Type::RPAREN);
                        break;
                    }
                    // todo we need to deal with function call contexts here
                    // we need to pass a context object down!
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

    fn index(&mut self, parse_context: ParseContext) -> ASTNode {
        let lhs_pos = get_position!(self);
        let higher_precedence = self.struct_access(parse_context);
        if !self.end() && self.tokens[self.counter].typ == Type::LSQUARE {
            consume_token!(self, Type::LSQUARE);
            let the_index = self.struct_access(parse_context);
            let rhs_pos = self.tokens[self.counter].pos.clone();
            consume_token!(self, Type::RSQUARE);
            return ASTNode {
                statement: Statement::INDEX(Box::new(higher_precedence), Box::new(the_index)),
                position: lhs_pos.join(rhs_pos),
            };
        }
        higher_precedence
    }

    fn struct_access(&mut self, parse_context: ParseContext) -> ASTNode {
        // Parse the leftmost expression first (e.g., x in x.y.z)
        let lhs_pos = get_position!(self);
        let mut lhs = self.tuple(parse_context);
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

    fn tuple(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_expression = self.single(parse_context);
        if !self.end()
            && self.tokens[self.counter].typ == Type::COMMA
            && ((parse_context.in_function_call && parse_context.in_group)
                || (!parse_context.in_function_call))
        {
            let lhs_pos = higher_expression.position;
            let mut exprs: Vec<ASTNode> = vec![higher_expression];

            consume_token!(self, Type::COMMA);
            while !self.end() {
                exprs.push(self.single(parse_context));
                if self.end() || self.tokens[self.counter].typ != Type::COMMA {
                    break;
                }
                consume_token!(self, Type::COMMA);
            }

            return ASTNode {
                statement: Statement::TUPLE(exprs),
                position: lhs_pos,
            };
        }
        higher_expression
    }

    fn single(&mut self, parse_context: ParseContext) -> ASTNode {
        let next: &Token = &self.tokens[self.counter];
        match next.typ {
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
            Type::STRING_LITERAL(_) => self.string(parse_context),
            Type::ATOM(_) => self.atom(parse_context),
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
            Type::LPAREN => {
                let lhs_pos = get_position!(self);
                consume_token!(self, Type::LPAREN);
                let mut expr = self.expression(parse_context);
                let rhs_pos = get_position!(self);
                consume_token!(self, Type::RPAREN);
                expr.position = lhs_pos.join(rhs_pos);
                return expr;
            }
            Type::LSQUARE => {
                let lhs_pos = get_position!(self);
                consume_token!(self, Type::LSQUARE);
                let mut items: Vec<ASTNode> = vec![];
                if self.tokens[self.counter].typ != Type::RSQUARE {
                    loop {
                        items.push(self.single(parse_context));
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
                let expr = self.expression(parse_context);
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

    fn logical_operators(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.equality(parse_context);

        if !self.end() && self.tokens[self.counter].typ == Type::OR {
            consume_token!(self, Type::OR);
            let rhs = self.logical_operators(parse_context);
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

    fn equality(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.add_sub(parse_context);
        if !self.end() && self.tokens[self.counter].typ == Type::EQUALS {
            consume_token!(self, Type::EQUALS);
            let rhs = self.equality(parse_context);
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
            let rhs = self.equality(parse_context);
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
            let rhs = self.equality(parse_context);
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
            let rhs = self.equality(parse_context);
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
            let rhs = self.equality(parse_context);
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
            let rhs = self.expression(parse_context);
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

    fn add_sub(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.mul_div(parse_context);
        if !self.end() && self.tokens[self.counter].typ == Type::ADD {
            consume_token!(self, Type::ADD);
            let rhs: ASTNode = self.add_sub(parse_context);
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
            let rhs = self.add_sub(parse_context);
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

    fn mul_div(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.bitwise(parse_context);
        if !self.end() && self.tokens[self.counter].typ == Type::MUL {
            consume_token!(self, Type::MUL);
            let rhs = self.mul_div(parse_context);
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
            let rhs = self.mul_div(parse_context);
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

    fn bitwise(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.tryy(parse_context);
        if !self.end() && self.tokens[self.counter].typ == Type::BITWISE_OR {
            consume_token!(self, Type::BITWISE_OR);
            let rhs = self.bitwise(parse_context);
            let pos = higher_precedence
                .position
                .clone()
                .join(rhs.position.clone());
            return ASTNode {
                statement: Statement::BIN_OP(
                    Box::new(higher_precedence),
                    Box::new(rhs),
                    Op::BITWISE_OR,
                ),
                position: pos,
            };
        }
        return higher_precedence;
    }

    fn matchh(&mut self, parse_context: ParseContext) -> ASTNode {
        let matchh_pos = get_position!(self);
        consume_token!(self, Type::MATCH);
        let match_value = self.expression(parse_context);
        consume_token!(self, Type::DO);

        // the cases here
        let t = get_next!(self);
        consume_token!(self, Type::ASSIGN);
        consume_token!(self, Type::GREATER_THAN);
        let expr = self.statement(parse_context);
        let pattern = Statement::MATCH_CASE(t.clone(), Box::new(expr));
        consume_token!(self, Type::END);
        ASTNode {
            statement: Statement::MATCH(
                Box::new(match_value),
                vec![ASTNode {
                    statement: pattern,
                    position: matchh_pos.clone(),
                }],
            ),
            position: matchh_pos,
        }
    }

    fn assert(&mut self, parse_context: ParseContext) -> ASTNode {
        let assert_pos = get_position!(self);
        consume_token!(self, Type::ASSERT);
        let expr = self.expression(parse_context);
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

    fn block(&mut self, parse_context: ParseContext) -> ASTNode {
        let do_pos = get_position!(self);
        consume_token!(self, Type::DO);
        let mut stms = vec![];
        while !self.end() && self.tokens[self.counter].typ != Type::END {
            stms.push(self.statement(parse_context));
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

    fn test(&mut self, parse_context: ParseContext) -> ASTNode {
        let test_pos = get_position!(self);
        consume_token!(self, Type::TEST);
        let test_name = self.string(parse_context);
        let body = self.statement(parse_context);
        let body_pos = body.position.clone();
        ASTNode {
            statement: Statement::TEST(Box::new(test_name), Box::new(body)),
            position: test_pos.join(body_pos),
        }
    }

    fn iff(&mut self, parse_context: ParseContext) -> ASTNode {
        let if_pos = get_position!(self);
        consume_token!(self, Type::IF);
        let condition = self.expression(parse_context);
        let body = self.statement(parse_context);
        let body_pos = body.position.clone();
        let mut else_body: Option<Box<ASTNode>> = None;
        if !self.end() && self.tokens[self.counter].typ == Type::ELSE {
            consume_token!(self, Type::ELSE);
            else_body = Some(Box::new(self.statement(parse_context)));
        }
        ASTNode {
            statement: Statement::IF(Box::new(condition), Box::new(body), else_body),
            position: if_pos.join(body_pos),
        }
    }

    fn forr(&mut self, parse_context: ParseContext) -> ASTNode {
        let for_pos = get_position!(self);
        consume_token!(self, Type::FOR);
        let var = get_next!(self);
        consume_token!(self, Type::IN);
        let iter_obj = self.parse_range(parse_context);
        let body = self.statement(parse_context);
        let body_pos = body.position.clone();
        return ASTNode {
            statement: Statement::FOR(var.clone(), Box::new(iter_obj), Box::new(body)),
            position: for_pos.join(body_pos.clone()),
        };
    }

    fn parse_range(&mut self, parse_context: ParseContext) -> ASTNode {
        let higher_precedence = self.logical_operators(parse_context);
        let first_pos = higher_precedence.position;
        if !self.end() && self.tokens[self.counter].typ == Type::DOT_DOT {
            consume_token!(self, Type::DOT_DOT);
            let second = self.expression(parse_context);
            let second_pos = second.position;
            return ASTNode {
                statement: Statement::RANGE(Box::new(higher_precedence), Box::new(second)),
                position: first_pos.join(second_pos),
            };
        }
        higher_precedence
    }

    fn ret(&mut self, parse_context: ParseContext) -> ASTNode {
        let pos = get_position!(self);
        consume_token!(self, Type::RETURN);
        let val = self.expression(parse_context);
        let rhs_pos = val.position.clone();
        return ASTNode {
            statement: Statement::RETURN(Some(Box::new(val))),
            position: pos.clone().join(rhs_pos),
        };
    }

    fn string(&mut self, parse_context: ParseContext) -> ASTNode {
        let s = get_next!(self);
        ASTNode {
            statement: Statement::STRING(s.clone()),
            position: s.pos.clone(),
        }
    }

    fn atom(&mut self, parse_context: ParseContext) -> ASTNode {
        let pos = get_position!(self);
        let tok = get_next!(self);
        return ASTNode {
            statement: Statement::ATOM(tok.clone()),
            position: pos,
        };
    }

    fn identifier(&mut self, parse_context: ParseContext) -> ASTNode {
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
            let typ = self.parse_type(parse_context);
            // move over the =
            consume_token!(self, Type::ASSIGN);
            let rhs = self.expression(parse_context);
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
                        params.push(self.parse_decl(parse_context));
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
                    return_type = Some(self.parse_type(parse_context));
                }
            }

            let rhs = self.statement(parse_context);
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
                    decls.push(self.parse_decl(parse_context));
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
            let rhs = self.expression(parse_context);
            let rhs_pos = rhs.position.clone();
            return ASTNode {
                statement: Statement::DEFINE(identifier.clone(), None, Some(Box::new(rhs))),
                position: lhs_pos.join(rhs_pos),
            };
        }

        self.expression(parse_context)
    }

    fn parse_type(&mut self, parse_context: ParseContext) -> DataType {
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
            Type::I32 => t = DataType::I32,
            Type::I64 => t = DataType::I64,
            Type::F32 => t = DataType::F32,
            Type::F64 => t = DataType::F64,
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

    fn parse_decl(&mut self, parse_context: ParseContext) -> ASTNode {
        let lhs_pos = get_position!(self);
        let identifier = get_next!(self);
        consume_token!(self, Type::COLON);
        let rhs_pos = self.tokens[self.counter].pos.clone();
        let typ = self.parse_type(parse_context);
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
