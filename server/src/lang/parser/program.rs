use rustc_hash::FxHashMap;
use serde_json::{Map, Value};

use crate::lang::ast::{
    expr::{Expr, ExprId, EXPR_ID_PLACEHOLDER},
    sql::{SqlExpr, SQL_EXPR_ID_PLACEHOLDER},
    stmt::{Stmt, StmtId, STMT_ID_PLACEHOLDER},
    AstRef,
};

use super::AstArena;

pub struct Program {
    root: StmtId,
    arena: AstArena,
    locals: Option<FxHashMap<usize, usize>>,
}

impl Program {
    pub fn new(root: StmtId, arena: AstArena) -> Program {
        Program {
            root,
            arena,
            locals: None,
        }
    }

    pub fn set_locals(&mut self, locals: FxHashMap<usize, usize>) {
        self.locals = Some(locals);
    }

    pub fn get_distance(&self, eid: ExprId) -> Option<usize> {
        self.locals.as_ref().unwrap().get(&eid.0).copied()
    }

    pub fn get_expression(&self, idx: ExprId) -> &Expr {
        &self.arena.expressions.get(idx)
    }

    pub fn get_statement(&self, idx: StmtId) -> &Stmt {
        &self.arena.statements.get(idx)
    }

    pub fn get_root(&self) -> StmtId {
        self.root
    }

    pub fn to_json(&self) -> Value {
        self.to_json_recursive(serde_json::to_value(self.arena.statements.get(self.root)).unwrap())
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
            let idx = AstRef::<Stmt>::new(
                map[STMT_ID_PLACEHOLDER]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            );
            self.to_json_recursive(serde_json::to_value(self.arena.statements.get(idx)).unwrap())
        } else if map.contains_key(EXPR_ID_PLACEHOLDER) {
            let idx = AstRef::<Expr>::new(
                map[EXPR_ID_PLACEHOLDER]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            );
            self.to_json_recursive(serde_json::to_value(self.arena.expressions.get(idx)).unwrap())
        } else if map.contains_key(SQL_EXPR_ID_PLACEHOLDER) {
            let idx = AstRef::<SqlExpr>::new(
                map[SQL_EXPR_ID_PLACEHOLDER]
                    .as_u64()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            );
            self.to_json_recursive(
                serde_json::to_value(self.arena.sql_expressions.get(idx)).unwrap(),
            )
        } else {
            serde_json::map::Map::into_iter(map)
                .map(|(key, value)| (key, self.to_json_recursive(value)))
                .collect()
        }
    }
}
