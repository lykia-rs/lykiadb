use super::{
    Identifier, IdentifierKind, Literal,
    expr::{Expr, Operation, RangeKind},
};
use std::sync::Arc;

pub struct AstBuilder {}

impl AstBuilder {
    pub fn call(&self, callee: Box<Expr>, args: Vec<Expr>) -> Box<Expr> {
        Box::new(Expr::Call {
            callee,
            args,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn variable(&self, name: &str) -> Box<Expr> {
        Box::new(Expr::Variable {
            name: Identifier::new(name, IdentifierKind::Symbol),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn literal_num(&self, value: f64) -> Box<Expr> {
        Box::new(Expr::Literal {
            value: Literal::Num(value),
            raw: value.to_string(),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn literal_str(&self, value: &str) -> Box<Expr> {
        Box::new(Expr::Literal {
            value: Literal::Str(Arc::new(value.to_string())),
            raw: value.to_string(),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn literal_bool(&self, value: bool) -> Box<Expr> {
        Box::new(Expr::Literal {
            value: Literal::Bool(value),
            raw: value.to_string(),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn binary(&self, left: Box<Expr>, op: Operation, right: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Binary {
            left,
            operation: op,
            right,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn unary(&self, op: Operation, expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Unary {
            operation: op,
            expr,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn grouping(&self, expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Grouping {
            expr,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn assignment(&self, dst: &str, expr: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Assignment {
            dst: Identifier::new(dst, IdentifierKind::Symbol),
            expr,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn logical(&self, left: Box<Expr>, op: Operation, right: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Logical {
            left,
            operation: op,
            right,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn get(&self, object: Box<Expr>, name: &str) -> Box<Expr> {
        Box::new(Expr::Get {
            object,
            name: Identifier::new(name, IdentifierKind::Symbol),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn field_path(&self, head: &str, tail: Vec<&str>) -> Box<Expr> {
        Box::new(Expr::FieldPath {
            head: Identifier::new(head, IdentifierKind::Symbol),
            tail: tail
                .into_iter()
                .map(|t| Identifier::new(t, IdentifierKind::Symbol))
                .collect(),
            span: Default::default(),
            id: 0,
        })
    }

    pub fn set(&self, object: Box<Expr>, name: &str, value: Box<Expr>) -> Box<Expr> {
        Box::new(Expr::Set {
            object,
            name: Identifier::new(name, IdentifierKind::Symbol),
            value,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn between(
        &self,
        subject: Box<Expr>,
        lower: Box<Expr>,
        upper: Box<Expr>,
        kind: RangeKind,
    ) -> Box<Expr> {
        Box::new(Expr::Between {
            lower,
            upper,
            subject,
            kind,
            span: Default::default(),
            id: 0,
        })
    }

    pub fn function(
        &self,
        name: Option<&str>,
        parameters: Vec<(&str, Option<Box<Expr>>)>,
        return_type: Option<Box<Expr>>,
        body: Vec<super::stmt::Stmt>,
    ) -> Box<Expr> {
        Box::new(Expr::Function {
            name: name.map(|n| Identifier::new(n, IdentifierKind::Symbol)),
            parameters: parameters
                .into_iter()
                .map(|(name, type_expr)| {
                    (
                        Identifier::new(name, IdentifierKind::Symbol),
                        type_expr.map(|expr| super::expr::TypeAnnotation {
                            type_expr: expr,
                            span: Default::default(),
                        }),
                    )
                })
                .collect(),
            return_type: return_type.map(|expr| super::expr::TypeAnnotation {
                type_expr: expr,
                span: Default::default(),
            }),
            body: Arc::new(body),
            span: Default::default(),
            id: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic_expressions() {
        let b = AstBuilder {};

        // Test variable
        let var = b.variable("x");
        assert!(matches!(*var, Expr::Variable { .. }));

        // Test binary expression without needing to box manually
        let binary = b.binary(b.literal_num(1.0), Operation::Add, b.literal_num(2.0));
        assert!(matches!(*binary, Expr::Binary { .. }));
    }
}
