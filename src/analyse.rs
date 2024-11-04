use crate::ast::Statement;

pub struct Analyser {

}

impl Analyser {
    pub fn analyse(&self, ast: &Statement){
        println!("analysing {:?}... done!", ast);
    }
}