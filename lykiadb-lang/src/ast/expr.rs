use derivative::Derivative;
use serde::{Deserialize, Serialize};

use std::{fmt::Display, sync::Arc};

use super::{
    sql::{SqlDelete, SqlInsert, SqlSelect, SqlUpdate},
    stmt::Stmt,
    AstNode, Identifier, Literal, Span, Spanned,
};

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
    #[serde(rename = "Expr::FieldPath")]
    FieldPath {
        head: Identifier,
        tail: Vec<Identifier>,
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
            Expr::Select { span, .. }
            | Expr::Insert { span, .. }
            | Expr::Delete { span, .. }
            | Expr::Update { span, .. }
            | Expr::Variable { span, .. }
            | Expr::Grouping { span, .. }
            | Expr::Literal { span, .. }
            | Expr::Function { span, .. }
            | Expr::Between { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Assignment { span, .. }
            | Expr::Logical { span, .. }
            | Expr::Call { span, .. }
            | Expr::Get { span, .. }
            | Expr::FieldPath { span, .. }
            | Expr::Set { span, .. } => *span,
        }
    }
}

impl AstNode for Expr {
    fn get_id(&self) -> usize {
        match self {
            Expr::Select { id, .. }
            | Expr::Insert { id, .. }
            | Expr::Delete { id, .. }
            | Expr::Update { id, .. }
            | Expr::Variable { id, .. }
            | Expr::Grouping { id, .. }
            | Expr::Literal { id, .. }
            | Expr::Function { id, .. }
            | Expr::Between { id, .. }
            | Expr::Binary { id, .. }
            | Expr::Unary { id, .. }
            | Expr::Assignment { id, .. }
            | Expr::Logical { id, .. }
            | Expr::Call { id, .. }
            | Expr::Get { id, .. }
            | Expr::FieldPath { id, .. }
            | Expr::Set { id, .. } => *id,
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
            Expr::Literal { value, .. } => match value {
                Literal::Str(s) => write!(f, "Str(\"{}\")", s),
                Literal::Num(n) => write!(f, "Num({:?})", n),
                Literal::Bool(b) => write!(f, "{}", b),
                Literal::Object(o) => write!(f, "Object({:?})", o),
                Literal::Array(a) => write!(
                    f,
                    "Array({})",
                    a.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Literal::Undefined => write!(f, "Undefined"),
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
                "({} {} {} And {})",
                subject,
                match kind {
                    RangeKind::Between => "Between",
                    RangeKind::NotBetween => "NotBetween",
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
            Expr::FieldPath { head, tail, .. } => {
                write!(
                    f,
                    "{}{}",
                    head,
                    tail.iter()
                        .map(|x| format!(".{}", x))
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
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
    pub fn walk<V, E>(
        &self,
        visitor: &mut impl FnMut(&Expr) -> Option<Result<V, E>>,
    ) -> Option<Result<V, E>> {
        let result = visitor(self);
        result.as_ref()?;
        if let Some(Err(_)) = result {
            return result;
        }
        match self {
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Delete { .. }
            | Expr::Update { .. }
            | Expr::Variable { .. }
            | Expr::Literal { .. }
            | Expr::FieldPath { .. }
            | Expr::Function { .. } => None,
            //
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                let rleft = left.walk(visitor);
                let rright = right.walk(visitor);

                rleft.or(rright)
            }
            //
            Expr::Grouping { expr, .. }
            | Expr::Unary { expr, .. }
            | Expr::Assignment { expr, .. } => expr.walk(visitor),
            //
            Expr::Call { callee, args, .. } => {
                let rcallee = callee.walk(visitor);
                let rargs = args
                    .iter()
                    .map(|x| x.walk(visitor))
                    .fold(None, |acc, x| acc.or(x));

                rcallee.or(rargs)
            }
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                let rlower = lower.walk(visitor);
                let rupper = upper.walk(visitor);
                let rsubject = subject.walk(visitor);

                rlower.or(rupper).or(rsubject)
            }
            Expr::Get { object, .. } => object.walk(visitor),
            Expr::Set { object, value, .. } => {
                let robject = object.walk(visitor);
                let rvalue = value.walk(visitor);

                robject.or(rvalue)
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::collections::HashSet;

    use crate::ast::expr::Expr;

    use super::*;

    #[test]
    fn test_expr_walk() {
        // Test simple expressions that don't traverse children
        {
            let simple_exprs = vec![
                Expr::Variable {
                    name: Identifier::new("x", false),
                    span: Span::default(),
                    id: 1,
                },
                Expr::Literal {
                    value: Literal::Num(42.0),
                    raw: "42".to_string(),
                    span: Span::default(),
                    id: 2,
                },
                Expr::FieldPath {
                    head: Identifier::new("user", false),
                    tail: vec![],
                    span: Span::default(),
                    id: 3,
                },
                Expr::Function {
                    name: Some(Identifier::new("test", false)),
                    parameters: vec![],
                    body: Arc::new(vec![]),
                    span: Span::default(),
                    id: 4,
                },
            ];

            for expr in simple_exprs {
                let mut visited = vec![];
                expr.walk(&mut |e| {
                    visited.push(e.get_id());
                    Some(Ok::<(), ()>(()))
                });
                assert_eq!(visited, vec![expr.get_id()]);
            }
        }

        // Test Binary and Logical expressions
        {
            let binary_expr = Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Num(1.0),
                    raw: "1".to_string(),
                    span: Span::default(),
                    id: 1,
                }),
                operation: Operation::Add,
                right: Box::new(Expr::Literal {
                    value: Literal::Num(2.0),
                    raw: "2".to_string(),
                    span: Span::default(),
                    id: 2,
                }),
                span: Span::default(),
                id: 3,
            };

            let mut visited = vec![];
            binary_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![3, 1, 2]);
        }

        // Test Grouping, Unary, and Assignment expressions
        {
            let unary_expr = Expr::Unary {
                operation: Operation::Not,
                expr: Box::new(Expr::Literal {
                    value: Literal::Bool(true),
                    raw: "true".to_string(),
                    span: Span::default(),
                    id: 1,
                }),
                span: Span::default(),
                id: 2,
            };

            let mut visited = vec![];
            unary_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![2, 1]);
        }

        // Test Call expression
        {
            let call_expr = Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: Identifier::new("test_func", false),
                    span: Span::default(),
                    id: 1,
                }),
                args: vec![
                    Expr::Literal {
                        value: Literal::Num(1.0),
                        raw: "1".to_string(),
                        span: Span::default(),
                        id: 2,
                    },
                    Expr::Literal {
                        value: Literal::Num(2.0),
                        raw: "2".to_string(),
                        span: Span::default(),
                        id: 3,
                    },
                ],
                span: Span::default(),
                id: 4,
            };

            let mut visited = vec![];
            call_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![4, 1, 2, 3]);
        }

        // Test Between expression
        {
            let between_expr = Expr::Between {
                lower: Box::new(Expr::Literal {
                    value: Literal::Num(1.0),
                    raw: "1".to_string(),
                    span: Span::default(),
                    id: 1,
                }),
                upper: Box::new(Expr::Literal {
                    value: Literal::Num(10.0),
                    raw: "10".to_string(),
                    span: Span::default(),
                    id: 2,
                }),
                subject: Box::new(Expr::Variable {
                    name: Identifier::new("x", false),
                    span: Span::default(),
                    id: 3,
                }),
                kind: RangeKind::Between,
                span: Span::default(),
                id: 4,
            };

            let mut visited = vec![];
            between_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![4, 1, 2, 3]);
        }

        // Test Get and Set expressions
        {
            let get_expr = Expr::Get {
                object: Box::new(Expr::Variable {
                    name: Identifier::new("obj", false),
                    span: Span::default(),
                    id: 1,
                }),
                name: Identifier::new("prop", false),
                span: Span::default(),
                id: 2,
            };

            let mut visited = vec![];
            get_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![2, 1]);

            let set_expr = Expr::Set {
                object: Box::new(Expr::Variable {
                    name: Identifier::new("obj", false),
                    span: Span::default(),
                    id: 1,
                }),
                name: Identifier::new("prop", false),
                value: Box::new(Expr::Literal {
                    value: Literal::Num(42.0),
                    raw: "42".to_string(),
                    span: Span::default(),
                    id: 2,
                }),
                span: Span::default(),
                id: 3,
            };

            let mut visited = vec![];
            set_expr.walk(&mut |e| {
                visited.push(e.get_id());
                Some(Ok::<(), ()>(()))
            });
            assert_eq!(visited, vec![3, 1, 2]);
        }
    }

    #[test]
    fn test_expr_get_id() {
        // Test Variable
        let var_expr = Expr::Variable {
            name: Identifier::new("test_var", false),
            span: Span::default(),
            id: 2,
        };
        assert_eq!(var_expr.get_id(), 2);

        // Test Grouping
        let group_expr = Expr::Grouping {
            expr: Box::new(Expr::Literal {
                value: Literal::Num(42.0),
                raw: "42".to_string(),
                span: Span::default(),
                id: 3,
            }),
            span: Span::default(),
            id: 4,
        };
        assert_eq!(group_expr.get_id(), 4);

        // Test Between
        let between_expr = Expr::Between {
            lower: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 5,
            }),
            upper: Box::new(Expr::Literal {
                value: Literal::Num(10.0),
                raw: "10".to_string(),
                span: Span::default(),
                id: 6,
            }),
            subject: Box::new(Expr::Variable {
                name: Identifier::new("x", false),
                span: Span::default(),
                id: 7,
            }),
            kind: RangeKind::Between,
            span: Span::default(),
            id: 8,
        };
        assert_eq!(between_expr.get_id(), 8);

        // Test Binary
        let binary_expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 9,
            }),
            operation: Operation::Add,
            right: Box::new(Expr::Literal {
                value: Literal::Num(2.0),
                raw: "2".to_string(),
                span: Span::default(),
                id: 10,
            }),
            span: Span::default(),
            id: 11,
        };
        assert_eq!(binary_expr.get_id(), 11);

        // Test Logical
        let logical_expr = Expr::Logical {
            left: Box::new(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: Span::default(),
                id: 12,
            }),
            operation: Operation::And,
            right: Box::new(Expr::Literal {
                value: Literal::Bool(false),
                raw: "false".to_string(),
                span: Span::default(),
                id: 13,
            }),
            span: Span::default(),
            id: 14,
        };
        assert_eq!(logical_expr.get_id(), 14);

        // Test Call
        let call_expr = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("test_func", false),
                span: Span::default(),
                id: 15,
            }),
            args: vec![],
            span: Span::default(),
            id: 16,
        };
        assert_eq!(call_expr.get_id(), 16);

        // Test FieldPath
        let field_path_expr = Expr::FieldPath {
            head: Identifier::new("user", false),
            tail: vec![],
            span: Span::default(),
            id: 17,
        };
        assert_eq!(field_path_expr.get_id(), 17);
    }

    #[test]
    fn test_expr_get_span() {
        let test_span = Span::default();

        // Test Variable
        let var_expr = Expr::Variable {
            name: Identifier::new("test_var", false),
            span: test_span,
            id: 5,
        };
        assert_eq!(var_expr.get_span(), test_span);

        // Test Grouping
        let group_expr = Expr::Grouping {
            expr: Box::new(Expr::Literal {
                value: Literal::Num(42.0),
                raw: "42".to_string(),
                span: Span::default(),
                id: 6,
            }),
            span: test_span,
            id: 7,
        };
        assert_eq!(group_expr.get_span(), test_span);

        // Test Literal
        let literal_expr = Expr::Literal {
            value: Literal::Num(42.0),
            raw: "42".to_string(),
            span: test_span,
            id: 8,
        };
        assert_eq!(literal_expr.get_span(), test_span);

        // Test Function
        let func_expr = Expr::Function {
            name: Some(Identifier::new("test_func", false)),
            parameters: vec![],
            body: Arc::new(vec![]),
            span: test_span,
            id: 9,
        };
        assert_eq!(func_expr.get_span(), test_span);

        // Test Between
        let between_expr = Expr::Between {
            lower: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 10,
            }),
            upper: Box::new(Expr::Literal {
                value: Literal::Num(10.0),
                raw: "10".to_string(),
                span: Span::default(),
                id: 11,
            }),
            subject: Box::new(Expr::Variable {
                name: Identifier::new("x", false),
                span: Span::default(),
                id: 12,
            }),
            kind: RangeKind::Between,
            span: test_span,
            id: 13,
        };
        assert_eq!(between_expr.get_span(), test_span);

        // Test Binary
        let binary_expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 14,
            }),
            operation: Operation::Add,
            right: Box::new(Expr::Literal {
                value: Literal::Num(2.0),
                raw: "2".to_string(),
                span: Span::default(),
                id: 15,
            }),
            span: test_span,
            id: 16,
        };
        assert_eq!(binary_expr.get_span(), test_span);

        // Test Unary
        let unary_expr = Expr::Unary {
            operation: Operation::Not,
            expr: Box::new(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: Span::default(),
                id: 17,
            }),
            span: test_span,
            id: 18,
        };
        assert_eq!(unary_expr.get_span(), test_span);

        // Test Assignment
        let assignment_expr = Expr::Assignment {
            dst: Identifier::new("x", false),
            expr: Box::new(Expr::Literal {
                value: Literal::Num(42.0),
                raw: "42".to_string(),
                span: Span::default(),
                id: 19,
            }),
            span: test_span,
            id: 20,
        };
        assert_eq!(assignment_expr.get_span(), test_span);

        // Test Logical
        let logical_expr = Expr::Logical {
            left: Box::new(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: Span::default(),
                id: 21,
            }),
            operation: Operation::And,
            right: Box::new(Expr::Literal {
                value: Literal::Bool(false),
                raw: "false".to_string(),
                span: Span::default(),
                id: 22,
            }),
            span: test_span,
            id: 23,
        };
        assert_eq!(logical_expr.get_span(), test_span);

        // Test Call
        let call_expr = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("test_func", false),
                span: Span::default(),
                id: 24,
            }),
            args: vec![],
            span: test_span,
            id: 25,
        };
        assert_eq!(call_expr.get_span(), test_span);

        // Test Get
        let get_expr = Expr::Get {
            object: Box::new(Expr::Variable {
                name: Identifier::new("obj", false),
                span: Span::default(),
                id: 26,
            }),
            name: Identifier::new("prop", false),
            span: test_span,
            id: 27,
        };
        assert_eq!(get_expr.get_span(), test_span);

        // Test FieldPath
        let field_path_expr = Expr::FieldPath {
            head: Identifier::new("user", false),
            tail: vec![Identifier::new("name", false)],
            span: test_span,
            id: 28,
        };
        assert_eq!(field_path_expr.get_span(), test_span);

        // Test Set
        let set_expr = Expr::Set {
            object: Box::new(Expr::Variable {
                name: Identifier::new("obj", false),
                span: Span::default(),
                id: 29,
            }),
            name: Identifier::new("prop", false),
            value: Box::new(Expr::Literal {
                value: Literal::Num(42.0),
                raw: "42".to_string(),
                span: Span::default(),
                id: 30,
            }),
            span: test_span,
            id: 31,
        };
        assert_eq!(set_expr.get_span(), test_span);
    }

    #[test]
    fn test_expr_display() {
        // Test Variable display
        let var_expr = Expr::Variable {
            name: Identifier::new("test_var", false),
            span: Span::default(),
            id: 1,
        };
        assert_eq!(var_expr.to_string(), "test_var");

        // Test Grouping display
        let group_expr = Expr::Grouping {
            expr: Box::new(Expr::Literal {
                value: Literal::Num(42.0),
                raw: "42".to_string(),
                span: Span::default(),
                id: 2,
            }),
            span: Span::default(),
            id: 3,
        };
        assert_eq!(group_expr.to_string(), "(Num(42.0))");

        // Test Binary display
        let binary_expr = Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 4,
            }),
            operation: Operation::Add,
            right: Box::new(Expr::Literal {
                value: Literal::Num(2.0),
                raw: "2".to_string(),
                span: Span::default(),
                id: 5,
            }),
            span: Span::default(),
            id: 6,
        };
        assert_eq!(binary_expr.to_string(), "(Num(1.0) Add Num(2.0))");
    }

    #[test]
    fn test_field_path_display() {
        let field_path = Expr::FieldPath {
            head: Identifier::new("user", false),
            tail: vec![
                Identifier::new("address", false),
                Identifier::new("city", false),
            ],
            span: Span::default(),
            id: 1,
        };
        assert_eq!(field_path.to_string(), "user.address.city");
    }

    #[test]
    fn test_function_display() {
        let func = Expr::Function {
            name: Some(Identifier::new("test_func", false)),
            parameters: vec![Identifier::new("a", false), Identifier::new("b", false)],
            body: Arc::new(vec![]),
            span: Span::default(),
            id: 1,
        };
        assert_eq!(func.to_string(), "fn test_func(a, b)");
    }

    #[test]
    fn test_between_display() {
        let between = Expr::Between {
            lower: Box::new(Expr::Literal {
                value: Literal::Num(1.0),
                raw: "1".to_string(),
                span: Span::default(),
                id: 1,
            }),
            upper: Box::new(Expr::Literal {
                value: Literal::Num(10.0),
                raw: "10".to_string(),
                span: Span::default(),
                id: 2,
            }),
            subject: Box::new(Expr::Variable {
                name: Identifier::new("x", false),
                span: Span::default(),
                id: 3,
            }),
            kind: RangeKind::Between,
            span: Span::default(),
            id: 4,
        };
        assert_eq!(between.to_string(), "(x Between Num(1.0) And Num(10.0))");
    }

    #[test]
    fn test_call_display() {
        let call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("test_func", false),
                span: Span::default(),
                id: 1,
            }),
            args: vec![
                Expr::Literal {
                    value: Literal::Num(1.0),
                    raw: "1".to_string(),
                    span: Span::default(),
                    id: 2,
                },
                Expr::Literal {
                    value: Literal::Num(2.0),
                    raw: "2".to_string(),
                    span: Span::default(),
                    id: 3,
                },
            ],
            span: Span::default(),
            id: 4,
        };
        assert_eq!(call.to_string(), "test_func(Num(1.0), Num(2.0))");
    }

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
