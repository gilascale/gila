#[derive(Debug)]
pub struct Lexer{

}

impl Lexer {
    pub fn lex(&self, source: &str){
        println!("lexing {:?}... done!", source);
    }
}