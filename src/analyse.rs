use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::{ASTNode, Statement},
    lex::{Position, Token},
    r#type::DataType,
};

#[derive(Debug)]
pub enum TypeCheckError {
    TYPE_NOT_ASSIGNABLE(Position, Position, DataType, DataType),
    UNKNOWN_VARIABLE(Token),
}

struct Scope {
    vars: HashMap<Rc<String>, DataType>,
}

pub struct Analyser {
    scope_index: usize,
    scopes: Vec<Scope>,
}

impl Analyser {
    pub fn new() -> Self {
        return Analyser {
            scope_index: 0,
            scopes: vec![Scope {
                vars: HashMap::new(),
            }],
        };
    }

    pub fn analyse(&mut self, ast: &ASTNode) -> Result<(), TypeCheckError> {
        let res = self.visit(ast);
        if res.is_err() {
            return Err(res.err().unwrap());
        }
        Ok(())
    }

    fn visit(&mut self, statement: &ASTNode) -> Result<DataType, TypeCheckError> {
        match &statement.statement {
            Statement::PROGRAM(p) => self.visit_program(p),
            Statement::NAMED_FUNCTION(t, params, return_type, body) => Ok(DataType::U32),
            Statement::TEST(name, body) => Ok(DataType::U32),
            Statement::IF(cond, body, else_body) => self.visit_if(cond, body, else_body),
            Statement::FOR(var, range_start, range_end, body) => Ok(DataType::U32),
            Statement::DEFINE(t, typ, val) => self.visit_define(t, typ, val),
            Statement::ASSIGN(lhs, rhs) => Ok(DataType::U32),
            Statement::CALL(calee, args) => self.visit_call(calee, args),
            Statement::LITERAL_NUM(n) => self.visit_literal_num(n),
            Statement::STRING(s) => self.visit_string(s),
            Statement::SLICE(s) => self.visit_slice(s),
            Statement::VARIABLE(t) => self.visit_variable(t),
            Statement::NAMED_TYPE_DECL(t, decls) => self.visit_named_type_decl(&t, &decls),
            _ => panic!("Missing visit for {:?}", statement),
        }
    }

    fn visit_program(&mut self, program: &Vec<ASTNode>) -> Result<DataType, TypeCheckError> {
        for item in program {
            let res = self.visit(item);
            if res.is_err() {
                return Err(res.err().unwrap());
            }
        }
        Ok(DataType::U32)
    }

    fn visit_if(
        &mut self,
        cond: &Box<ASTNode>,
        body: &Box<ASTNode>,
        else_body: &Option<Box<ASTNode>>,
    ) -> Result<DataType, TypeCheckError> {
        Ok(DataType::U32)
    }

    fn visit_define(
        &mut self,
        token: &Token,
        typ: &Option<DataType>,
        val: &Option<Box<ASTNode>>,
    ) -> Result<DataType, TypeCheckError> {
        // lets analyse!

        let identifier = token.as_identifier();
        // existing var
        if self.scopes[self.scope_index].vars.contains_key(&identifier) {
        } else {
            if let Some(t) = typ {
                if let Some(v) = val {
                    // ensure types are same

                    let value_type = self.visit(v);

                    if value_type.is_err() {
                        return Err(value_type.err().unwrap());
                    }

                    let resolved_type = match t {
                        DataType::NAMED_REFERENCE(named_reference) => self.scopes[self.scope_index]
                            .vars
                            .get(named_reference)
                            .unwrap(),
                        _ => panic!(),
                    };

                    if resolved_type
                        .clone()
                        .assignable_from(value_type.as_ref().unwrap().clone())
                    {
                        self.scopes[self.scope_index]
                            .vars
                            .insert(token.as_identifier(), t.clone());
                    } else {
                        return Err(TypeCheckError::TYPE_NOT_ASSIGNABLE(
                            token.pos.clone(),
                            v.position.clone(),
                            t.clone(),
                            value_type.unwrap(),
                        ));
                    }
                }
            }
        }

        Ok(DataType::U32)
    }

    fn visit_call(
        &mut self,
        callee: &Box<ASTNode>,
        args: &Vec<ASTNode>,
    ) -> Result<DataType, TypeCheckError> {
        self.visit(&callee)
    }

    fn visit_literal_num(&mut self, n: &Token) -> Result<DataType, TypeCheckError> {
        Ok(DataType::U32)
    }

    fn visit_string(&mut self, s: &Token) -> Result<DataType, TypeCheckError> {
        Ok(DataType::STRING)
    }

    fn visit_slice(&mut self, s: &Vec<ASTNode>) -> Result<DataType, TypeCheckError> {
        // todo check whats in the slice
        Ok(DataType::SLICE(Box::new(DataType::U32)))
    }

    fn visit_variable(&mut self, t: &Token) -> Result<DataType, TypeCheckError> {
        if self.scopes[self.scope_index]
            .vars
            .contains_key(&t.as_identifier())
        {
            return Ok(self.scopes[self.scope_index]
                .vars
                .get(&t.as_identifier())
                .unwrap()
                .clone());
        }
        return Err(TypeCheckError::UNKNOWN_VARIABLE(t.clone()));
    }

    fn visit_named_type_decl(
        &mut self,
        t: &Token,
        decls: &Vec<ASTNode>,
    ) -> Result<DataType, TypeCheckError> {
        let mut v: Vec<DataType> = vec![];
        for decl in decls {
            let decl_type = self.visit(decl);
            if decl_type.is_err() {
                return Err(decl_type.err().unwrap());
            }
            v.push(decl_type.unwrap());
        }

        // todo insert type into scope
        self.scopes[self.scope_index]
            .vars
            .insert(t.as_identifier(), DataType::DYNAMIC_OBJECT(v));

        Ok(DataType::U32)
    }
}
