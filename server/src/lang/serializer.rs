use serde::{Serialize, ser::SerializeMap};
use serde_json::{Value, Map};

use super::{parser::Program, ast::{stmt::StmtId, expr::ExprId}};

pub struct ProgramSerializer<'a> {
    pub program: &'a Program,
}

impl<'a> ProgramSerializer<'a> {
    pub fn new(program: &'a Program) -> ProgramSerializer<'a> {
        ProgramSerializer { program }
    }
    pub fn to_json(&self) -> Value {
        self.to_json_recursive(serde_json::to_value(self.program.arena.get_statement(self.program.root)).unwrap())
    }

    fn to_json_recursive(&self, value: Value) -> Value {
        match value {
            Value::Object(map) => self.resolve_json_map(map),
            Value::Array(values) => {
                Value::Array(values.into_iter().map(|value| {
                    self.to_json_recursive(value)
                }).collect::<Vec<Value>>())
            }
            Value::String(_) |
            Value::Number(_) |
            Value::Bool(_) |
            Value::Null => value,
        }
    }

    fn resolve_json_map(&self, map: Map<String, Value> ) -> Value {
        if map.contains_key("@StmtId") {
            let idx = StmtId(map["@StmtId"].as_u64().unwrap().try_into().unwrap());
            self.to_json_recursive(
                serde_json::to_value(self.program.arena.get_statement(idx)).unwrap()
            )
        }
        else if map.contains_key("@ExprId") {
            let idx = ExprId(map["@ExprId"].as_u64().unwrap().try_into().unwrap());
            self.to_json_recursive(
                serde_json::to_value(self.program.arena.get_expression(idx)).unwrap()
            )
        }
        else {
            serde_json::map::Map::into_iter(map).map(|(key, value)| {
                (key, self.to_json_recursive(value))
            }).collect()
        }
    }
}

impl Serialize for ExprId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("@ExprId", &self.0)?;
        map.end()
    }
}

impl Serialize for StmtId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("@StmtId", &self.0)?;
        map.end()
    }
}
