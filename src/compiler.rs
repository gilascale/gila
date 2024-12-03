use std::collections::HashMap;

pub enum CompilationUnitStatus {
    TODO,
    DONE,
}

pub struct Compiler {
    // keep track of files and their states
    pub compilation_units: HashMap<String, CompilationUnitStatus>,
}

impl Compiler {
    pub fn new() -> Self {
        return Compiler {
            compilation_units: HashMap::new(),
        };
    }
}
