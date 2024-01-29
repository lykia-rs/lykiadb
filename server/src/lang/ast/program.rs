use std::sync::Arc;

use super::{
    expr::{Expr, ExprId},
    stmt::{Stmt, StmtId},
};
use serde::{ser::SerializeMap, Serialize};
use serde_json::{Map, Value};

pub struct Program {
    pub root: StmtId,
    pub arena: Arc<AstArena>,
}

impl Program {
    pub fn new(root: StmtId, arena: Arc<AstArena>) -> Program {
        Program { root, arena }
    }
}
pub struct AstArena {
    expressions: Vec<Expr>,
    statements: Vec<Stmt>,
}

impl AstArena {
    pub fn new() -> AstArena {
        AstArena {
            expressions: vec![],
            statements: vec![],
        }
    }

    pub fn alloc_expression(&mut self, expr: Expr) -> ExprId {
        self.expressions.push(expr);
        ExprId(self.expressions.len() - 1)
    }

    pub fn alloc_statement(&mut self, stmt: Stmt) -> StmtId {
        self.statements.push(stmt);
        StmtId(self.statements.len() - 1)
    }

    pub fn get_expression(&self, idx: ExprId) -> &Expr {
        &self.expressions[idx.0]
    }

    pub fn get_statement(&self, idx: StmtId) -> &Stmt {
        &self.statements[idx.0]
    }
}

const EXPR_ID_PLACEHOLDER: &'static str = "@ExprId";
const STMT_ID_PLACEHOLDER: &'static str = "@StmtId";

impl Program {
    pub fn to_json(&self) -> Value {
        self.to_json_recursive(serde_json::to_value(self.arena.get_statement(self.root)).unwrap())
    }

    fn to_json_recursive(&self, value: Value) -> Value {
        match value {
            Value::Object(map) => self.resolve_json_map(map),
            Value::Array(values) => Value::Array(
                values
                    .into_iter()
                    .map(|value| self.to_json_recursive(value))
                    .collect::<Vec<Value>>(),
            ),
            Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => value,
        }
    }

    fn resolve_json_map(&self, map: Map<String, Value>) -> Value {
        if map.contains_key(STMT_ID_PLACEHOLDER) {
            let idx = StmtId(
                map[STMT_ID_PLACEHOLDER]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            );
            self.to_json_recursive(serde_json::to_value(self.arena.get_statement(idx)).unwrap())
        } else if map.contains_key(EXPR_ID_PLACEHOLDER) {
            let idx = ExprId(
                map[EXPR_ID_PLACEHOLDER]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            );
            self.to_json_recursive(serde_json::to_value(self.arena.get_expression(idx)).unwrap())
        } else {
            serde_json::map::Map::into_iter(map)
                .map(|(key, value)| (key, self.to_json_recursive(value)))
                .collect()
        }
    }
}

impl Serialize for ExprId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(EXPR_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}

impl Serialize for StmtId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(STMT_ID_PLACEHOLDER, &self.0)?;
        map.end()
    }
}
