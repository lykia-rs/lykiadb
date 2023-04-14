use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum RV {
    Str(String),
    Num(f32),
    Bool(bool),
    Undefined,
    NaN,
    Nil
}

pub struct Environment {
    vars: HashMap<String, RV>
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            vars: HashMap::new()
        }
    }

    pub fn declare(&mut self, name: String, value: RV) {
        self.vars.insert(name, value);
    }

    pub fn read(&self, name: &String) -> Option<&RV> {
        self.vars.get(name)
    }
}