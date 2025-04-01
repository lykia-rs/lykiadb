use super::{expr::Expr, stmt::Stmt};

pub trait Visitor<O, E> {
    fn visit_expr(&self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&self, s: &Stmt) -> Result<O, E>;
}
pub trait VisitorMut<O, E> {
    fn visit_expr(&mut self, e: &Expr) -> Result<O, E>;
    fn visit_stmt(&mut self, s: &Stmt) -> Result<O, E>;
}

pub trait Collector<V, E> {

    fn take(&mut self, expr: &Expr) -> Result<Vec<V>, E>;

    fn collect(
        &mut self,
        expr: &Expr
    ) -> Result<Vec<V>, E> {
        let result = self.take(expr);
        if let Err(e) = result {
            return Err(e);
        }
        let subtree = match expr {
            Expr::Select { .. }
            | Expr::Insert { .. }
            | Expr::Delete { .. }
            | Expr::Update { .. }
            | Expr::Variable { .. }
            | Expr::Literal { .. }
            | Expr::FieldPath { .. }
            | Expr::Function { .. } => Ok(vec![]),
            //
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                let rleft = self.collect(left);
                let rright = self.collect(right);

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
            Expr::Grouping { expr, .. }
            | Expr::Unary { expr, .. }
            | Expr::Assignment { expr, .. } => {
                let r = self.collect(expr);
                if let Ok(v) = r {
                    return Ok(v);
                }
                Ok(vec![])
            },
            //
            Expr::Call { callee, args, .. } => {
                let rcallee = self.collect(callee);
                let rargs = args
                    .iter()
                    .map(|x| self.collect(x))
                    .fold(Ok(vec![]), |acc: Result<Vec<V>, E>, x| {
                        let mut result = acc?;
                        if let Ok(v) = x {
                            result.extend(v);
                        }
                        Ok(result)
                    });

                let mut result = vec![];
                if let Ok(v) = rcallee {
                    result.extend(v);
                }
                if let Ok(v) = rargs {
                    result.extend(v);
                }
                Ok(result)
            }
            Expr::Between {
                lower,
                upper,
                subject,
                ..
            } => {
                let rlower = self.collect(lower);
                let rupper = self.collect(upper);
                let rsubject = self.collect(subject);

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
                let robject = self.collect(object);
                if let Ok(v) = robject {
                    return Ok(v);
                }
                Ok(vec![])
            },
            Expr::Set { object, value, .. } => {
                let robject = self.collect(object);
                let rvalue = self.collect(value);

                let mut result = vec![];
                if let  Ok(v) = robject {
                    result.extend(v);
                }
                if let Ok(v) = rvalue {
                    result.extend(v);
                }
                Ok(result)
            }
        };
        subtree
    }
}

