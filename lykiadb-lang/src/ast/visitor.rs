use super::{expr::Expr, stmt::Stmt};

pub trait Visitor<O, E> {
    fn visit_expr(&self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&self, s: &Stmt) -> Result<O, E>;
}
pub trait VisitorMut<O, E> {
    fn visit_expr(&mut self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&mut self, s: &Stmt) -> Result<O, E>;
}

pub struct ExprVisitor<E> {
    visit_callback: Box<dyn ExprVisitorNodeCallback<(), E>>,
    is_consumed: bool,
}

pub enum ExprVisitorNode {
    In,
    Out,
}

pub trait ExprVisitorNodeCallback<T, E> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<Vec<T>, E>;
}

impl<E> ExprVisitor<E>{

    pub fn new(visit_callback: Box<dyn ExprVisitorNodeCallback<(), E>>) -> Self {
        Self {
            visit_callback,
            is_consumed: false,
        }
    }

    pub fn visit(
        self: &mut Self,
        expr: &Expr,
    ) -> Result<Vec<Expr>, E> {
        if self.is_consumed {
            return Ok(vec![]);
        }

        let result = self._traverse(expr)?;
        
        self.is_consumed = true;

        Ok(result)
    }
    
    fn _traverse(
        self: &mut Self,
        expr: &Expr,
    ) -> Result<Vec<Expr>, E> {
        self.visit_callback.visit(&expr, ExprVisitorNode::In)?;

        let r = match expr {
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Delete { .. }
            | Expr::Update { .. }
            | Expr::Literal { .. }
            | Expr::FieldPath { .. }
            | Expr::Function { .. }
            | Expr::Set { .. } => Ok(vec![]),
            //
            Expr::Variable { .. } => {
                Ok(vec![expr.clone()])
            }
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                let rleft = self._traverse(left);
                let rright = self._traverse(right);

                let mut result = vec![];
                if let Ok(v) = rleft {
                    result.extend(v);
                }
                if let Ok(v) = rright {
                    result.extend(v);
                }
                Ok(result)
            }
            //
            Expr::Grouping { expr, .. } | Expr::Unary { expr, .. } | Expr::Assignment { expr, .. } => {
                let r = self._traverse(expr);
                if let Ok(v) = r {
                    return Ok(v);
                }
                Ok(vec![])
            }
            //
            Expr::Call { callee, args, .. } => {
                let mut result: Vec<Expr> = vec![];

                result.extend(self._traverse(callee)?);

                for arg in args {
                    result.extend(self._traverse(arg)?);
                }

                Ok(result)
            }
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                let rlower = self._traverse(lower);
                let rupper = self._traverse(upper);
                let rsubject = self._traverse(subject);

                let mut result = vec![];
                if let Ok(v) = rlower {
                    result.extend(v);
                }
                if let Ok(v) = rupper {
                    result.extend(v);
                }
                if let Ok(v) = rsubject {
                    result.extend(v);
                }
                Ok(result)
            }
            Expr::Get { object, .. } => {
                let robject = self._traverse(object);
                if let Ok(v) = robject {
                    return Ok(v);
                }
                Ok(vec![])
            }
        };

        self.visit_callback.visit(&expr, ExprVisitorNode::Out)?;

        r
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{expr::Expr, Identifier, IdentifierKind, Span};
    struct CallRestrictorRule {
        in_call: u32,
    }

    impl ExprVisitorNodeCallback<(), String> for CallRestrictorRule {
        fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<Vec<()>, String> {
            if let Expr::Call { callee, .. } = expr {
                if let Expr::Variable{ name, .. } = callee.as_ref() {
                    if name.name == "avg" {
                        match visit {
                            ExprVisitorNode::In => {
                                if self.in_call > 0 && name.name == "avg" {
                                    return Err("avg() cannot be called inside another avg()".to_string());
                                }
                                self.in_call += 1;
                            }
                            ExprVisitorNode::Out => {
                                self.in_call -= 1;
                            }
                        }
                    }
                }
            }

            Ok(vec![])
        }
    }

    #[test]
    fn test_nested_avg() {
        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: Identifier::new("avg", IdentifierKind::Symbol),
                    span: Span::default(),
                    id: 0,
                }),
                args: vec![],
                span: Span::default(),
                id: 0,
            }],
            span: Span::default(),
            id: 0,
        };

        let mut visitor: ExprVisitor<String> = ExprVisitor::<String>::new(
            Box::new(CallRestrictorRule { in_call: 0 }),
        );

        assert_eq!(
            visitor.visit(&avg_call),
            Err("avg() cannot be called inside another avg()".to_string())
        );
    }

    #[test]
    fn test_deeply_nested_avg() {
        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![Expr::Call {
                callee: Box::new(Expr::Variable {
                    name: Identifier::new("something_else", IdentifierKind::Symbol),
                    span: Span::default(),
                    id: 0,
                }),
                args: vec![Expr::Call {
                    callee: Box::new(Expr::Variable {
                        name: Identifier::new("avg", IdentifierKind::Symbol),
                        span: Span::default(),
                        id: 0,
                    }),
                    args: vec![],
                    span: Span::default(),
                    id: 0,
                }],
                span: Span::default(),
                id: 0,
            }],
            span: Span::default(),
            id: 0,
        };

        let mut visitor = ExprVisitor::<String>::new(
            Box::new(CallRestrictorRule { in_call: 0 }),
        );

        assert_eq!(
            visitor.visit(&avg_call),
            Err("avg() cannot be called inside another avg()".to_string())
        );
    }

    #[test]
    fn test_sum_two_avgs() {
        let avg_call = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("sum", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![
                Expr::Call {
                    callee: Box::new(Expr::Variable {
                        name: Identifier::new("avg", IdentifierKind::Symbol),
                        span: Span::default(),
                        id: 0,
                    }),
                    args: vec![],
                    span: Span::default(),
                    id: 0,
                },
                Expr::Call {
                    callee: Box::new(Expr::Variable {
                        name: Identifier::new("avg", IdentifierKind::Symbol),
                        span: Span::default(),
                        id: 0,
                    }),
                    args: vec![],
                    span: Span::default(),
                    id: 0,
                },
            ],
            span: Span::default(),
            id: 0,
        };

        let mut visitor = ExprVisitor::<String>::new(
            Box::new(CallRestrictorRule { in_call: 0 }),
        );

        assert_eq!(visitor.visit(&avg_call).is_ok(), true);
    }
}