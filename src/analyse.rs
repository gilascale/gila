use std::collections::HashMap;

use crate::{
    ast::{ASTNode, Statement},
    lex::Token,
    r#type::DataType,
};

#[derive(Debug)]
pub enum TypeCheckError {
    TYPE_NOT_ASSIGNABLE,
}

struct Scope {
    vars: HashMap<Token, DataType>,
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

    pub fn analyse(&mut self, ast: &ASTNode) {
        let result = self.visit(ast);
        match result {
            Ok(_) => {}
            Err(e) => println!("Typechecking failed {:?}", e),
        }
    }

    fn visit(&mut self, statement: &ASTNode) -> Result<DataType, TypeCheckError> {
        match &statement.statement {
            Statement::PROGRAM(p) => self.visit_program(p),
            Statement::DEFINE(t, typ, val) => self.visit_define(t, typ, val),
            Statement::CALL(calee, args) => self.visit_call(calee, args),
            Statement::LITERAL_NUM(n) => self.visit_literal_num(n),
            Statement::STRING(s) => self.visit_string(s),
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

    fn visit_define(
        &mut self,
        token: &Token,
        typ: &Option<DataType>,
        val: &Option<Box<ASTNode>>,
    ) -> Result<DataType, TypeCheckError> {
        // lets analyse!

        // existing var
        if self.scopes[self.scope_index].vars.contains_key(token) {
        } else {
            if let Some(t) = typ {
                if let Some(v) = val {
                    // ensure types are same

                    let value_type = self.visit(v);

                    if value_type.is_err() {
                        return Err(value_type.err().unwrap());
                    }

                    if t.clone().assignable_from(value_type.unwrap()) {
                        self.scopes[self.scope_index]
                            .vars
                            .insert(token.clone(), t.clone());
                    } else {
                        return Err(TypeCheckError::TYPE_NOT_ASSIGNABLE);
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
        Ok(DataType::U32)
    }

    fn visit_literal_num(&mut self, n: &Token) -> Result<DataType, TypeCheckError> {
        Ok(DataType::U32)
    }
    fn visit_string(&mut self, s: &Token) -> Result<DataType, TypeCheckError> {
        Ok(DataType::STRING)
    }
}
