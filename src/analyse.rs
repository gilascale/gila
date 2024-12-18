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
    UNKNOWN_DATA_TYPE(Rc<String>, Position),
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
            Statement::IMPORT(module) => Ok(DataType::U32),
            Statement::NAMED_FUNCTION(t, params, return_type, body) => {
                self.visit_named_fn(t, params, return_type, body)
            }
            Statement::TEST(name, body) => Ok(DataType::U32),
            Statement::IF(cond, body, else_body) => self.visit_if(cond, body, else_body),
            Statement::FOR(var, range_start, range_end, body) => Ok(DataType::U32),
            Statement::DEFINE(t, typ, val) => self.visit_define(t, typ, val),
            Statement::ASSIGN(lhs, rhs) => self.visit_assign(lhs, rhs),
            Statement::CALL(calee, args) => self.visit_call(calee, args),
            Statement::LITERAL_NUM(n) => self.visit_literal_num(n),
            Statement::STRING(s) => self.visit_string(s),
            Statement::SLICE(s) => self.visit_slice(s),
            Statement::VARIABLE(t) => self.visit_variable(t),
            Statement::NAMED_TYPE_DECL(t, decls) => self.visit_named_type_decl(&t, &decls),
            Statement::STRUCT_ACCESS(strct, member) => Ok(DataType::U32),
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

    fn resolve_data_type_to_concrete_type(
        &self,
        position: Position,
        t: DataType,
    ) -> Result<DataType, TypeCheckError> {
        match t {
            DataType::NAMED_REFERENCE(named_reference) => {
                if !self.scopes[self.scope_index]
                    .vars
                    .contains_key(&named_reference)
                {
                    return Err(TypeCheckError::UNKNOWN_DATA_TYPE(
                        named_reference.clone(),
                        position.clone(),
                    ));
                }
                Ok(self.scopes[self.scope_index]
                    .vars
                    .get(&named_reference)
                    .unwrap()
                    .clone())
            }
            _ => Ok(t),
        }
    }

    fn visit_define(
        &mut self,
        token: &Token,
        typ: &Option<DataType>,
        val: &Option<Box<ASTNode>>,
    ) -> Result<DataType, TypeCheckError> {
        let identifier = token.as_identifier();

        if self.scopes[self.scope_index].vars.contains_key(&identifier) {
            let rhs_value = val.as_ref().unwrap();
            let rhs_type = self.visit(&rhs_value);
            let lhs_type = self.scopes[self.scope_index].vars.get(&identifier).unwrap();

            if rhs_type.is_err() {
                return Err(rhs_type.err().unwrap());
            }

            let rhs_unrapped = rhs_type.unwrap();
            let res = lhs_type.clone().assignable_from(rhs_unrapped.clone());
            if !res {
                return Err(TypeCheckError::TYPE_NOT_ASSIGNABLE(
                    token.pos.clone(),
                    rhs_value.position.clone(),
                    lhs_type.clone(),
                    rhs_unrapped.clone(),
                ));
            }
            return Ok(lhs_type.clone());
        } else {
            if let Some(t) = typ {
                if let Some(v) = val {
                    // ensure types are same

                    let value_type = self.visit(v);

                    if value_type.is_err() {
                        return Err(value_type.err().unwrap());
                    }

                    let resolved_type_res =
                        self.resolve_data_type_to_concrete_type(token.pos.clone(), t.clone());

                    if resolved_type_res.is_err() {
                        return Err(resolved_type_res.err().unwrap());
                    }

                    let resolved_type = resolved_type_res.unwrap();

                    if resolved_type
                        .clone()
                        .assignable_from(value_type.as_ref().unwrap().clone())
                    {
                        self.scopes[self.scope_index]
                            .vars
                            .insert(identifier, t.clone());
                        return Ok(resolved_type.clone());
                    } else {
                        return Err(TypeCheckError::TYPE_NOT_ASSIGNABLE(
                            token.pos.clone(),
                            v.position.clone(),
                            t.clone(),
                            value_type.unwrap(),
                        ));
                    }
                } else {
                    // todo i think this is an err?
                    return self.resolve_data_type_to_concrete_type(token.pos.clone(), t.clone());
                }
            } else {
                let rhs_value = val.as_ref().unwrap();
                let value_type = self.visit(&rhs_value);

                if value_type.is_err() {
                    return Err(value_type.err().unwrap());
                }

                let v_type = value_type.unwrap();

                self.scopes[self.scope_index]
                    .vars
                    .insert(identifier, v_type.clone());

                return Ok(v_type);
            }
        }
    }

    fn visit_assign(
        &mut self,
        lhs: &Box<ASTNode>,
        rhs: &Box<ASTNode>,
    ) -> Result<DataType, TypeCheckError> {
        // todo this is never parsed, may remove!
        // let lhs_type = self.visit(&lhs);
        // let rhs_type = self.visit(&rhs);

        // if lhs_type.is_err() {
        //     return Err(lhs_type.err().unwrap());
        // }
        // if rhs_type.is_err() {
        //     return Err(rhs_type.err().unwrap());
        // }

        // let lhs_unrapped = lhs_type.unwrap();
        // let rhs_unrapped = rhs_type.unwrap();
        // let res = lhs_unrapped.clone().assignable_from(rhs_unrapped.clone());
        // if !res {
        //     return Err(TypeCheckError::TYPE_NOT_ASSIGNABLE(
        //         lhs.position.clone(),
        //         rhs.position.clone(),
        //         lhs_unrapped.clone(),
        //         rhs_unrapped.clone(),
        //     ));
        // }
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

    fn visit_named_fn(
        &mut self,
        t: &Token,
        params: &Vec<ASTNode>,
        return_type: &Option<DataType>,
        body: &Box<ASTNode>,
    ) -> Result<DataType, TypeCheckError> {
        let mut param_types: Vec<DataType> = vec![];
        for param in params {
            let res = self.visit(param);
            if res.is_err() {
                return Err(res.err().unwrap());
            }
            param_types.push(res.unwrap());
        }

        let return_type_resolved: DataType = if return_type.is_some() {
            return_type.clone().unwrap()
        } else {
            DataType::VOID
        };

        let fn_type = DataType::FN(param_types, Box::new(return_type_resolved));
        self.scopes[self.scope_index]
            .vars
            .insert(t.as_identifier(), fn_type.clone());

        Ok(fn_type)
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
