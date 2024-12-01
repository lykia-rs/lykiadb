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
            Expr::Literal { value, .. } => {
                match value {
                    Literal::Str(s) => write!(f, "Str(\"{}\")", s),
                    Literal::Num(n) => write!(f, "Num({:?})", n),
                    Literal::Bool(b) => write!(f, "{}", b),
                    Literal::Undefined => write!(f,  "undefined"),
                    Literal::Object(o) => write!(f, "{:?}", o),
                    Literal::Array(a) => write!(f, "Array({})", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
                    Literal::NaN => write!(f, "NaN"),
                    Literal::Null => write!(f, "null"),
                }
            },
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
    pub fn walk(&self, visitor: &mut impl FnMut(&Expr) -> bool) -> bool {
        if !visitor(self) {
            return false;
        }
        match self {
            Expr::Select { .. } 
            | Expr::Insert { .. }
            | Expr::Delete { .. }
            | Expr::Update { .. }
            | Expr::Variable { .. }
            | Expr::Literal { .. }
            | Expr::Function { .. } => false,
            //
            Expr::Binary { left, right, .. }
            | Expr::Logical { left, right, .. } => { 
                let rleft = left.walk(visitor);
                let rright = right.walk(visitor);

                rleft || rright
            },
            //
            Expr::Grouping { expr, .. }
            | Expr::Unary { expr, .. }
            | Expr::Assignment { expr, .. } => expr.walk(visitor),
            //
            Expr::Call { callee, args, .. } => {
                let rcallee = callee.walk(visitor);
                let rargs = args.iter().map(|x| x.walk(visitor)).all(|x| x);

                rcallee || rargs
            },
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                let rlower = lower.walk(visitor);
                let rupper = upper.walk(visitor);
                let rsubject = subject.walk(visitor);

                rlower || rupper || rsubject
            },
            Expr::Get { object, .. } => object.walk(visitor),
            Expr::Set { object, value, .. } => {
                let robject = object.walk(visitor);
                let rvalue = value.walk(visitor);

                robject || rvalue
            },
        }
    }

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

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;

    use crate::{ast::expr::Expr, Literal, Span};

    pub fn create_simple_add_expr(id: usize, left: f64, right: f64) -> Expr {
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Num(left),
                span: Span::default(),
                id: 0,
                raw: left.to_string(),
            }),
            operation: crate::ast::expr::Operation::Add,
            right: Box::new(Expr::Literal {
                value: Literal::Num(right),
                span: Span::default(),
                id: 0,
                raw: right.to_string(),
            }),
            span: Span::default(),
            id,
        }
    }

    #[test]
    fn identical_exprs_should_be_equal_when_ids_are_different() {
        let e0 = create_simple_add_expr(0, 1.0, 2.0);

        let e1 = create_simple_add_expr(1, 1.0, 2.0);

        assert_eq!(e0, e1);

        let mut set: HashSet<Expr> = HashSet::new();

        set.insert(e0);

        assert!(set.contains(&e1));
    }

    #[test]
    fn different_exprs_with_same_ids_should_not_be_equal() {
        let e0 = create_simple_add_expr(1, 2.0, 3.0);

        let e1 = create_simple_add_expr(1, 1.0, 2.0);

        assert_ne!(e0, e1);

        let mut set: HashSet<Expr> = HashSet::new();

        set.insert(e0);

        assert!(!set.contains(&e1));
    }

    #[test]
    fn mirrored_exprs_should_not_be_equal() {
        let e0 = create_simple_add_expr(0, 2.0, 1.0);

        let e1 = create_simple_add_expr(1, 1.0, 2.0);

        assert_ne!(e0, e1);

        let mut set: HashSet<Expr> = HashSet::new();

        set.insert(e0);

        assert!(!set.contains(&e1));
    }
}
