use super::{expr::Expr, stmt::Stmt};

pub trait Visitor<O, E> {
    fn visit_expr(&self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&self, s: &Stmt) -> Result<O, E>;
}
pub trait VisitorMut<O, E> {
    fn visit_expr(&mut self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&mut self, s: &Stmt) -> Result<O, E>;
}

pub struct ExprVisitor<'a, T, E> {
    visit_callback: &'a mut dyn ExprReducer<T, E>,
}

pub enum ExprVisitorNode {
    In,
    Out,
}

pub trait ExprReducer<T, E> {
    fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, E>;
    fn finalize(&mut self) -> Result<Vec<T>, E>;
}

impl<'a, T, E> ExprVisitor<'a, T, E> {
    pub fn new(visit_callback: &'a mut dyn ExprReducer<T, E>) -> Self {
        Self { visit_callback }
    }

    pub fn visit(&mut self, expr: &Expr) -> Result<Vec<T>, E> {
        self._traverse(expr)?;

        self.visit_callback.finalize()
    }

    fn _traverse(&mut self, expr: &Expr) -> Result<(), E> {
        let go = self.visit_callback.visit(expr, ExprVisitorNode::In)?;

        if !go {
            return Ok(());
        }

        match expr {
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Delete { .. }
            | Expr::Update { .. }
            | Expr::Literal { .. }
            | Expr::FieldPath { .. }
            | Expr::Function { .. }
            | Expr::Set { .. }
            | Expr::Variable { .. } => {}
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self._traverse(left)?;
                self._traverse(right)?;
            }
            //
            Expr::Grouping { expr, .. }
            | Expr::Unary { expr, .. }
            | Expr::Assignment { expr, .. } => {
                self._traverse(expr)?;
            }
            //
            Expr::Call { callee, args, .. } => {
                self._traverse(callee)?;

                for arg in args {
                    self._traverse(arg)?;
                }
            }
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                self._traverse(lower)?;
                self._traverse(upper)?;
                self._traverse(subject)?;
            }
            Expr::Get { object, .. } => {
                self._traverse(object)?;
            }
        };

        self.visit_callback.visit(expr, ExprVisitorNode::Out)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Identifier, IdentifierKind, Span, expr::Expr};
    struct DummyAggregationCollector {
        in_call: u32,
        accumulator: Vec<Expr>,
    }

    impl ExprReducer<Expr, String> for DummyAggregationCollector {
        fn visit(&mut self, expr: &Expr, visit: ExprVisitorNode) -> Result<bool, String> {
            if let Expr::Call { callee, .. } = expr {
                if let Expr::Variable { name, .. } = callee.as_ref() {
                    if name.name == "avg" {
                        match visit {
                            ExprVisitorNode::In => {
                                if self.in_call > 0 {
                                    return Err(
                                        "avg() cannot be called inside another avg()".to_string()
                                    );
                                }
                                self.in_call += 1;
                                self.accumulator.push(expr.clone());
                            }
                            ExprVisitorNode::Out => {
                                self.in_call -= 1;
                            }
                        }
                    }
                }
            }

            Ok(true)
        }

        fn finalize(&mut self) -> Result<Vec<Expr>, String> {
            Ok(self.accumulator.clone())
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

        let mut agg = DummyAggregationCollector {
            in_call: 0,
            accumulator: vec![],
        };

        let mut visitor = ExprVisitor::<Expr, String>::new(&mut agg);

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

        let mut agg = DummyAggregationCollector {
            in_call: 0,
            accumulator: vec![],
        };

        let mut visitor = ExprVisitor::<Expr, String>::new(&mut agg);

        assert_eq!(
            visitor.visit(&avg_call),
            Err("avg() cannot be called inside another avg()".to_string())
        );
    }

    #[test]
    fn test_sum_two_avgs() {
        let avg0 = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![],
            span: Span::default(),
            id: 0,
        };

        let avg1 = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("avg", IdentifierKind::Symbol),
                span: Span::default(),
                id: 1,
            }),
            args: vec![],
            span: Span::default(),
            id: 1,
        };

        let sum = Expr::Call {
            callee: Box::new(Expr::Variable {
                name: Identifier::new("sum", IdentifierKind::Symbol),
                span: Span::default(),
                id: 0,
            }),
            args: vec![avg0.clone(), avg1.clone()],
            span: Span::default(),
            id: 0,
        };

        let mut agg = DummyAggregationCollector {
            in_call: 0,
            accumulator: vec![],
        };

        let mut visitor = ExprVisitor::<Expr, String>::new(&mut agg);

        assert_eq!(visitor.visit(&sum), Ok(vec![avg0, avg1]));
    }
}
