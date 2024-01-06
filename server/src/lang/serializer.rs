use serde_json::{json, Value};

use super::parser::Program;

pub struct ProgramSerializer<'a> {
    pub program: &'a Program,
}

impl<'a> ProgramSerializer<'a> {
    pub fn new(program: &'a Program) -> ProgramSerializer<'a> {
        ProgramSerializer { program }
    }
    pub fn to_json(&self) -> Value {
        serde_json::to_value(&self.program.root).unwrap()
    }
    pub fn serialize(&self) -> String {
        serde_json::to_string_pretty(&self.program.root).unwrap()
    }
}

impl<'a> ToString for ProgramSerializer<'a> {
    fn to_string(&self) -> String {
        self.serialize().clone()
    }
}