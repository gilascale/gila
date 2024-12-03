use crate::{
    ast::{ASTNode, Statement},
    r#type::DataType,
};

pub struct Analyser {}

impl Analyser {
    pub fn analyse(&self, ast: &ASTNode) {
        self.visit(ast);
    }

    fn visit(&self, statement: &ASTNode) -> DataType {
        match &statement.statement {
            Statement::PROGRAM(p) => self.visit_program(p),
            _ => panic!(),
        }
    }

    fn visit_program(&self, program: &Vec<ASTNode>) -> DataType {
        for item in program {
            self.visit(item);
        }
        DataType::U32
    }
}
