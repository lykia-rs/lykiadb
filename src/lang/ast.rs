use std::rc::Rc;
use uuid::Uuid;
use crate::lang::token::Token;
use crate::runtime::types::RV;

pub trait Visitor<T, Q> {
    fn visit_expr(&mut self, e: &Expr) -> T;
    fn visit_stmt(&mut self, e: &Stmt) -> Result<T, Q>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlDistinct {
    All,
    Distinct
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlResultColumns {

}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlTableSubquery {

}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlJoinClause {

}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlCompoundOperator {

}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlOrderingTerm {

}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlFrom {
    TableSubquery(Vec<SqlTableSubquery>),
    JoinClause(Box<SqlJoinClause>)
}

pub enum SqlCore {

}

#[derive(Debug, Eq, PartialEq)]
struct SelectCore {
    distinct: SqlDistinct,
    result_columns: Vec<SqlResultColumns>,
    from: SqlFrom,
    r#where: Option<Box<Expr>>,
    group_by: Option<Box<Expr>>,
    having: Option<Box<Expr>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlStmt {
    Select {
        core: SelectCore,
        compound: Vec<(SqlCompoundOperator, Box<SelectCore>)>,
        order_by_clause: Option<SqlOrderingTerm>,
        limit_clause: Option<Box<Expr>>,
        offset_clause: Option<Box<Expr>>,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(Box<Expr>),
    Sql(Box<SqlStmt>),
    Function(Token, Vec<Token>, Rc<Vec<Stmt>>),
    Declaration(Token, Box<Expr>),
    Block(Vec<Stmt>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Loop(Option<Box<Expr>>, Box<Stmt>, Option<Box<Stmt>>),
    Break(Token),
    Continue(Token),
    Return(Token, Option<Box<Expr>>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Binary(Uuid, Token, Box<Expr>, Box<Expr>),
    Grouping(Uuid, Box<Expr>),
    Literal(Uuid, RV),
    Unary(Uuid, Token, Box<Expr>),
    Variable(Uuid, Token),
    Assignment(Uuid, Token, Box<Expr>),
    Logical(Uuid, Box<Expr>, Token, Box<Expr>),
    Call(Uuid, Box<Expr>, Token, Vec<Box<Expr>>),
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

    pub fn new_binary(op: Token, left: Box<Expr>, right: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Binary(Uuid::new_v4(), op, left, right))
    }
    pub fn new_grouping(expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Grouping(Uuid::new_v4(), expr))
    }
    pub fn new_literal(value: RV) -> Box<Expr> {
        Box::new(Expr::Literal(Uuid::new_v4(), value))
    }
    pub fn new_unary(op: Token, expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Unary(Uuid::new_v4(), op, expr))
    }
    pub fn new_variable(name: Token) -> Box<Expr> {
        Box::new(Expr::Variable(Uuid::new_v4(), name))
    }
    pub fn new_assignment(name: Token, value: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Assignment(Uuid::new_v4(), name, value))
    }
    pub fn new_logical(left: Box<Expr>, op: Token, right: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Logical(Uuid::new_v4(), left, op, right))
    }
    pub fn new_call(callee: Box<Expr>, paren: Token, arguments: Vec<Box<Expr>>) -> Box<Expr> {
        Box::new(Expr::Call(Uuid::new_v4(), callee, paren, arguments))
    }
}
