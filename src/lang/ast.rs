use std::rc::Rc;
use crate::lang::token::Token;
use crate::runtime::types::RV;

pub trait Visitor<T, Q> {
    fn visit_expr(&mut self, e: ExprId) -> T;
    fn visit_stmt(&mut self, e: StmtId) -> Result<T, Q>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlDistinct {
    All,
    Distinct
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlProjection {
    All,
    /*AllColumnsOf{
        table: Token
    },*/
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
pub struct SelectCore {
    pub distinct: SqlDistinct,
    pub projection: Vec<SqlProjection>,
    pub from: Option<SqlFrom>,
    pub r#where: Option<SqlExpr>,
    pub group_by: Option<SqlExpr>,
    pub having: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SqlSelect {
    pub core: Box<SelectCore>,
    pub compound: Vec<(SqlCompoundOperator, Box<SelectCore>)>,
    pub order_by: Option<(SqlExpr, Option<SqlOrdering>)>,
    pub limit: Option<SqlExpr>,
    pub offset: Option<SqlExpr>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SqlExpr {
    Default(ExprId)
}

#[derive(Debug, Eq, PartialEq)]
pub enum Stmt {
    Expression(ExprId),
    Function(Token, Vec<Token>, Rc<Vec<StmtId>>),
    Declaration(Token, ExprId),
    Block(Vec<StmtId>),
    If(ExprId, StmtId, Option<StmtId>),
    Loop(Option<ExprId>, StmtId, Option<StmtId>),
    Break(Token),
    Continue(Token),
    Return(Token, Option<ExprId>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Select(Box<SqlSelect>),
    Binary(Token, ExprId, ExprId),
    Grouping(ExprId),
    Literal(RV),
    Unary(Token, ExprId),
    Variable(Token),
    Assignment(Token, ExprId),
    Logical(ExprId, Token, ExprId),
    Call(ExprId, Token, Vec<ExprId>),
}

impl Expr {
    pub fn new_binary(op: Token, left: ExprId, right: ExprId) -> Expr {
        Expr::Binary(op, left, right)
    }
    pub fn new_select(select: Box<SqlSelect>) -> Expr {
        Expr::Select(select)
    }
    pub fn new_grouping(expr: ExprId) -> Expr {
        Expr::Grouping(expr)
    }
    pub fn new_literal(value: RV) -> Expr {
        Expr::Literal(value)
    }
    pub fn new_unary(op: Token, expr: ExprId) -> Expr {
        Expr::Unary(op, expr)
    }
    pub fn new_variable(name: Token) -> Expr {
        Expr::Variable(name)
    }
    pub fn new_assignment(name: Token, value: ExprId) -> Expr {
        Expr::Assignment(name, value)
    }
    pub fn new_logical(left: ExprId, op: Token, right: ExprId) -> Expr {
        Expr::Logical(left, op, right)
    }
    pub fn new_call(callee: ExprId, paren: Token, arguments: Vec<ExprId>) -> Expr {
        Expr::Call(callee, paren, arguments)
    }
}


pub type ExprId = usize;
pub type StmtId = usize;

pub struct ParserArena {
    expressions: Vec<Expr>,
    statements: Vec<Stmt>,
}

impl ParserArena {
    pub fn new() -> ParserArena {
        ParserArena {
            expressions: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn expression(&mut self, expr: Expr) -> ExprId {
        self.expressions.push(expr);
        self.expressions.len() - 1
    }

    pub fn statement(&mut self, stmt: Stmt) -> StmtId {
        self.statements.push(stmt);
        self.statements.len() - 1
    }

    pub fn get_expression(&self, idx: ExprId) -> &Expr {
        &self.expressions[idx]
    }

    pub fn get_statement(&self, idx: StmtId) -> &Stmt {
        &self.statements[idx]
    }
}