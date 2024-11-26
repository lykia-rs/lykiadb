use derivative::Derivative;
use serde::{Deserialize, Serialize};

use std::{fmt::Display, sync::Arc};

use crate::{Identifier, Span, Spanned};

use super::{
    sql::{SqlDelete, SqlInsert, SqlSelect, SqlUpdate},
    stmt::Stmt,
    AstNode,
};

use crate::Literal;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(tag = "@type")]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    IsEqual,
    IsNotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Not,
    Is,
    IsNot,
    In,
    NotIn,
    Like,
    NotLike,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
#[serde(tag = "@type")]
pub enum RangeKind {
    Between,
    NotBetween,
}

#[derive(Debug, Serialize, Deserialize, Clone, Derivative)]
#[serde(tag = "@type")]
#[derivative(Eq, PartialEq, Hash)]
pub enum Expr {
    #[serde(rename = "Expr::Select")]
    Select {
        query: SqlSelect,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Insert")]
    Insert {
        command: SqlInsert,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Update")]
    Update {
        command: SqlUpdate,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Delete")]
    Delete {
        command: SqlDelete,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Variable")]
    Variable {
        name: Identifier,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Grouping")]
    Grouping {
        expr: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Literal")]
    Literal {
        value: Literal,
        raw: String,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Function")]
    Function {
        name: Option<Identifier>,
        parameters: Vec<Identifier>,
        body: Arc<Vec<Stmt>>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Between")]
    Between {
        lower: Box<Expr>,
        upper: Box<Expr>,
        subject: Box<Expr>,
        kind: RangeKind,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Binary")]
    Binary {
        left: Box<Expr>,
        operation: Operation,
        right: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Unary")]
    Unary {
        operation: Operation,
        expr: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Assignment")]
    Assignment {
        dst: Identifier,
        expr: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Logical")]
    Logical {
        left: Box<Expr>,
        operation: Operation,
        right: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Call")]
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Get")]
    Get {
        object: Box<Expr>,
        name: Identifier,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
    #[serde(rename = "Expr::Set")]
    Set {
        object: Box<Expr>,
        name: Identifier,
        value: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        id: usize,
    },
}

impl Spanned for Expr {
    fn get_span(&self) -> Span {
        match self {
            Expr::Select {
                query: _,
                span,
                id: _,
            }
            | Expr::Insert {
                command: _,
                span,
                id: _,
            }
            | Expr::Delete {
                command: _,
                span,
                id: _,
            }
            | Expr::Update {
                command: _,
                span,
                id: _,
            }
            | Expr::Variable {
                name: _,
                span,
                id: _,
            }
            | Expr::Grouping {
                expr: _,
                span,
                id: _,
            }
            | Expr::Literal {
                value: _,
                raw: _,
                span,
                id: _,
            }
            | Expr::Function {
                name: _,
                parameters: _,
                body: _,
                span,
                id: _,
            }
            | Expr::Between {
                lower: _,
                upper: _,
                subject: _,
                kind: _,
                span,
                id: _,
            }
            | Expr::Binary {
                left: _,
                operation: _,
                right: _,
                span,
                id: _,
            }
            | Expr::Unary {
                operation: _,
                expr: _,
                span,
                id: _,
            }
            | Expr::Assignment {
                dst: _,
                expr: _,
                span,
                id: _,
            }
            | Expr::Logical {
                left: _,
                operation: _,
                right: _,
                span,
                id: _,
            }
            | Expr::Call {
                callee: _,
                args: _,
                span,
                id: _,
            }
            | Expr::Get {
                object: _,
                name: _,
                span,
                id: _,
            }
            | Expr::Set {
                object: _,
                name: _,
                value: _,
                span,
                id: _,
            } => *span,
        }
    }
}

impl AstNode for Expr {
    fn get_id(&self) -> usize {
        match self {
            Expr::Select {
                query: _,
                span: _,
                id,
            }
            | Expr::Insert {
                command: _,
                span: _,
                id,
            }
            | Expr::Delete {
                command: _,
                span: _,
                id,
            }
            | Expr::Update {
                command: _,
                span: _,
                id,
            }
            | Expr::Variable {
                name: _,
                span: _,
                id,
            }
            | Expr::Grouping {
                expr: _,
                span: _,
                id,
            }
            | Expr::Literal {
                value: _,
                raw: _,
                span: _,
                id,
            }
            | Expr::Function {
                name: _,
                parameters: _,
                body: _,
                span: _,
                id,
            }
            | Expr::Between {
                lower: _,
                upper: _,
                subject: _,
                kind: _,
                span: _,
                id,
            }
            | Expr::Binary {
                left: _,
                operation: _,
                right: _,
                span: _,
                id,
            }
            | Expr::Unary {
                operation: _,
                expr: _,
                span: _,
                id,
            }
            | Expr::Assignment {
                dst: _,
                expr: _,
                span: _,
                id,
            }
            | Expr::Logical {
                left: _,
                operation: _,
                right: _,
                span: _,
                id,
            }
            | Expr::Call {
                callee: _,
                args: _,
                span: _,
                id,
            }
            | Expr::Get {
                object: _,
                name: _,
                span: _,
                id,
            }
            | Expr::Set {
                object: _,
                name: _,
                value: _,
                span: _,
                id,
            } => *id,
        }
    }
}

impl Display for Expr {
    // A basic display function for Expr
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Select { .. } => write!(f, "<SqlSelect>"),
            Expr::Insert { .. } => write!(f, "<SqlInsert>"),
            Expr::Update { .. } => write!(f, "<SqlUpdate>"),
            Expr::Delete { .. } => write!(f, "<SqlDelete>"),
            Expr::Variable { name, .. } => write!(f, "{}", name),
            Expr::Grouping { expr, .. } => write!(f, "({})", expr),
            Expr::Literal { value, .. } => write!(f, "{:?}", value),
            Expr::Function {
                name, parameters, ..
            } => {
                write!(
                    f,
                    "fn {}({})",
                    name.as_ref().unwrap(),
                    parameters
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Between {
                lower,
                upper,
                subject,
                kind,
                ..
            } => write!(
                f,
                "{} {} {} AND {}",
                subject,
                match kind {
                    RangeKind::Between => "BETWEEN",
                    RangeKind::NotBetween => "NOT BETWEEN",
                },
                lower,
                upper
            ),
            Expr::Binary {
                left,
                operation,
                right,
                ..
            } => {
                write!(f, "({} {:?} {})", left, operation, right)
            }
            Expr::Unary {
                operation, expr, ..
            } => write!(f, "{:?}{}", operation, expr),
            Expr::Assignment { dst, expr, .. } => write!(f, "{} = {}", dst, expr),
            Expr::Logical {
                left,
                operation,
                right,
                ..
            } => {
                write!(f, "{} {:?} {}", left, operation, right)
            }
            Expr::Call { callee, args, .. } => {
                write!(
                    f,
                    "{}({})",
                    callee,
                    args.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Get { object, name, .. } => write!(f, "{}.{}", object, name),
            Expr::Set {
                object,
                name,
                value,
                ..
            } => write!(f, "{}.{} = {}", object, name, value),
        }
    }
}

impl Expr {
    pub fn collect(&self, predicate: &impl Fn(&Expr) -> bool, collected: &mut Vec<Expr>) {
        if predicate(self) {
            collected.push(self.clone());
            return;
        }
        match &self {
            Expr::Grouping { expr, .. } => expr.collect(predicate, collected),
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                lower.collect(predicate, collected);
                upper.collect(predicate, collected);
                subject.collect(predicate, collected);
            }
            Expr::Binary { left, right, .. } => {
                left.collect(predicate, collected);
                right.collect(predicate, collected);
            }
            Expr::Unary { expr, .. } => expr.collect(predicate, collected),
            Expr::Logical { left, right, .. } => {
                left.collect(predicate, collected);
                right.collect(predicate, collected);
            }
            Expr::Call { args, .. } => {
                args.iter().for_each(|x| x.collect(predicate, collected));
            }
            _ => (),
        }
    }
}
