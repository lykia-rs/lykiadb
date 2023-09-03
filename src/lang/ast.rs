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
    All,
    AllColumnsOf{
        table: Token
    },
    Complex {
        expr: SqlExpr, 
        alias: Option<Token>
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlTableSubquery {
    Simple {
        namespace: Option<Token>,
        table: Token,
        alias: Option<Token>
    },
    From {
        from: SqlFrom
    },
    Select {
        stmt: SqlSelect,
        alias: Option<Token>
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlJoinType {
    Left,
    LeftOuter,
    Right,
    Inner
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlJoinClause {
    None(SqlTableSubquery),
    Join(Vec<(SqlJoinType, SqlTableSubquery, SqlExpr)>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlCompoundOperator {
    Union,
    UnionAll,
    Intersect,
    Except
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlOrdering {
    Asc,
    Desc
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlFrom {
    TableSubquery(Vec<SqlTableSubquery>),
    JoinClause(Box<SqlJoinClause>)
}

#[derive(Debug, Eq, PartialEq)]
struct SelectCore {
    distinct: SqlDistinct,
    result_columns: Vec<SqlResultColumns>,
    from: SqlFrom,
    r#where: Option<SqlExpr>,
    group_by: Option<SqlExpr>,
    having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelect {
    core: SelectCore,
    compound: Vec<(SqlCompoundOperator, Box<SelectCore>)>,
    order_by: (SqlExpr, Option<SqlOrdering>),
    limit: Option<SqlExpr>,
    offset: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlExpr {
    Default(Expr)
}

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(Box<Expr>),
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
    Select(Uuid, Box<SqlSelect>),
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
            Expr::Select(id, _) => *id,
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
