use crate::ast::Statement;





pub struct CodeGenerator{

}

impl CodeGenerator{
    
    pub fn generate(&self, ast: &Statement){
        println!("generating... {:?} done!", ast);
    }

}