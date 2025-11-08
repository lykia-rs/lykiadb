use derivative::Derivative;
use serde::{Deserialize, Serialize};

use super::{Identifier, Span, Spanned, expr::Expr};

#[derive(Debug, Serialize, Deserialize, Clone, Derivative)]
#[serde(tag = "@type")]
#[derivative(Eq, PartialEq, Hash)]
pub enum Stmt {
    #[serde(rename = "Stmt::Program")]
    Program {
        body: Vec<Stmt>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Expression")]
    Expression {
        expr: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Break")]
    Break {
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Continue")]
    Continue {
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Block")]
    Block {
        body: Vec<Stmt>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Declaration")]
    Declaration {
        dst: Identifier,
        expr: Box<Expr>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::If")]
    If {
        condition: Box<Expr>,
        body: Box<Stmt>,
        r#else_body: Option<Box<Stmt>>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Loop")]
    Loop {
        condition: Option<Box<Expr>>,
        body: Box<Stmt>,
        post: Option<Box<Stmt>>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
    #[serde(rename = "Stmt::Return")]
    Return {
        expr: Option<Box<Expr>>,
        #[serde(skip)]
        #[derivative(PartialEq = "ignore")]
        #[derivative(Hash = "ignore")]
        span: Span,
    },
}

impl Spanned for Stmt {
    fn get_span(&self) -> Span {
        match self {
            Stmt::Program { span, .. } => *span,
            Stmt::Expression { span, .. } => *span,
            Stmt::Break { span, .. } => *span,
            Stmt::Continue { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Declaration { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::Loop { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::ast::{
        Span,
        expr::{Expr, test_utils::create_simple_add_expr},
        stmt::Stmt,
    };

    pub fn create_simple_block_stmt(a: Expr, b: Expr) -> Stmt {
        Stmt::Block {
            body: vec![
                Stmt::Expression {
                    expr: Box::new(a),
                    span: Span::default(),
                },
                Stmt::Expression {
                    expr: Box::new(b),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        }
    }

    #[test]
    fn identical_stmts_should_be_equal() {
        let s0 = create_simple_block_stmt(
            create_simple_add_expr(0, 1.0, 2.0),
            create_simple_add_expr(1, 1.0, 2.0),
        );

        let s1 = create_simple_block_stmt(
            create_simple_add_expr(0, 1.0, 2.0),
            create_simple_add_expr(1, 1.0, 2.0),
        );

        assert_eq!(s0, s1);

        let mut set: HashSet<Stmt> = HashSet::new();

        set.insert(s0);

        assert!(set.contains(&s1));
    }

    #[test]
    fn different_stmts_should_not_be_equal_0() {
        let s0 = create_simple_block_stmt(
            create_simple_add_expr(0, 1.0, 2.0),
            create_simple_add_expr(1, 1.0, 2.0),
        );

        let s1 = create_simple_block_stmt(
            create_simple_add_expr(0, 2.0, 1.0),
            create_simple_add_expr(1, 1.0, 2.0),
        );

        assert_ne!(s0, s1);

        let mut set: HashSet<Stmt> = HashSet::new();

        set.insert(s0);

        assert!(!set.contains(&s1));
    }

    #[test]
    fn different_stmts_should_not_be_equa_1() {
        let s0 = create_simple_block_stmt(
            create_simple_add_expr(0, 1.0, 2.0),
            create_simple_add_expr(0, 10.0, 20.0),
        );

        let s1 = create_simple_block_stmt(
            create_simple_add_expr(0, 10.0, 20.0),
            create_simple_add_expr(0, 1.0, 2.0),
        );

        assert_ne!(s0, s1);

        let mut set: HashSet<Stmt> = HashSet::new();

        set.insert(s0);

        assert!(!set.contains(&s1));
    }
}
