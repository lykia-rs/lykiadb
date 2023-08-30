use std::hash::{Hash, Hasher};
use std::rc::Rc;
use crate::lang::parsing::token::{LiteralValue, Token};
pub type BExpr = Box<Expr>;
pub type BStmt = Box<Stmt>;
use uuid::Uuid;

pub trait Visitor<T, Q> {
    fn visit_expr(&mut self, e: &Expr) -> T;
    fn visit_stmt(&mut self, e: &Stmt) -> Result<T, Q>;
}

#[derive(Debug)]
pub enum Expr {
    Binary(Uuid, Token, BExpr, BExpr),
    Grouping(Uuid, BExpr),
    Literal(Uuid, LiteralValue),
    Unary(Uuid, Token, BExpr),
    Variable(Uuid, Token),
    Assignment(Uuid, Token, BExpr),
    Logical(Uuid, BExpr, Token, BExpr),
    Call(Uuid, BExpr, Token, Vec<BExpr>),
}

impl Expr {

    pub fn id(&self) -> Uuid {
        match self {
            Expr::Binary(id, _, _, _) => *id,
            Expr::Grouping(id, _) => *id,
            Expr::Literal(id, _) => *id,
            Expr::Unary(id, _, _) => *id,
            Expr::Variable(id, _) => *id,
            Expr::Assignment(id, _, _) => *id,
            Expr::Logical(id, _, _, _) => *id,
            Expr::Call(id, _, _, _) => *id,
        }
    }

    pub fn new_binary(op: Token, left: BExpr, right: BExpr) -> Expr {
        Expr::Binary(Uuid::new_v4(), op, left, right)
    }
    pub fn new_grouping(expr: BExpr) -> Expr {
        Expr::Grouping(Uuid::new_v4(), expr)
    }
    pub fn new_literal(value: LiteralValue) -> Expr {
        Expr::Literal(Uuid::new_v4(), value)
    }
    pub fn new_unary(op: Token, expr: BExpr) -> Expr {
        Expr::Unary(Uuid::new_v4(), op, expr)
    }
    pub fn new_variable(name: Token) -> Expr {
        Expr::Variable(Uuid::new_v4(), name)
    }
    pub fn new_assignment(name: Token, value: BExpr) -> Expr {
        Expr::Assignment(Uuid::new_v4(), name, value)
    }
    pub fn new_logical(left: BExpr, op: Token, right: BExpr) -> Expr {
        Expr::Logical(Uuid::new_v4(), left, op, right)
    }
    pub fn new_call(callee: BExpr, paren: Token, arguments: Vec<BExpr>) -> Expr {
        Expr::Call(Uuid::new_v4(), callee, paren, arguments)
    }
}

pub enum Stmt {
    Expression(BExpr),
    Function(Token, Vec<Token>, Rc<Vec<Stmt>>),
    Declaration(Token, BExpr),
    Block(Vec<Stmt>),
    If(BExpr, BStmt, Option<BStmt>),
    Loop(Option<BExpr>, BStmt, Option<BStmt>),
    Break(Token),
    Continue(Token),
    Return(Token, Option<BExpr>)
}
