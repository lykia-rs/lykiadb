use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Undefined,
    NaN,
    Nil
}

pub struct EnvironmentStack {
    envs: Vec<HashMap<String, RV>>
}

impl EnvironmentStack {
    pub fn new() -> EnvironmentStack {
        EnvironmentStack {
            envs: vec![HashMap::new()]
        }
    }

    pub fn push(&mut self) {
        self.envs.insert(0, HashMap::new());
    }

    pub fn pop(&mut self) {
        self.envs.remove(0);
    }

    pub fn declare(&mut self, name: String, value: RV) {
        let env = self.envs.first_mut().unwrap();
        env.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: RV) -> Result<bool, String> {
        for x in self.envs.iter_mut() {
            if x.contains_key(&name) {
                x.insert(name, value);
                return Ok(true);
            }
        }
        Err(format!("Assignment to an undefined variable '{}'", &name))
    }

    pub fn read(&mut self, name: &String) -> Result<&RV, String> {
        for x in self.envs.iter_mut() {
            if x.contains_key(name) {
                return Ok(x.get(name).unwrap());
            }
        }
        Err(format!("Variable '{}' was not found.", &name))
    }
}