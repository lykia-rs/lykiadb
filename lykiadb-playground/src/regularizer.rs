use lykiadb_lang::{
    ast::{expr::Expr, stmt::Stmt, visitor::VisitorMut},
    parser::NodeMetadata,
    Literal, Span,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Tree {
    name: String,
    children: Option<Vec<Tree>>,
    span: Span,
}

pub struct TreeBuilder {
    node_metadata: FxHashMap<usize, NodeMetadata>,
}

impl TreeBuilder {
    pub fn new(node_metadata: FxHashMap<usize, NodeMetadata>) -> Self {
        Self { node_metadata }
    }
    pub fn build(&mut self, stmt: &Stmt) -> Tree {
        self.visit_stmt(stmt).unwrap()
    }

    pub fn get_children(&self, id: usize) -> Option<Vec<Tree>> {
        if self.node_metadata.contains_key(&id) {
            let children: Vec<Tree> = self.node_metadata[&id]
                .tokens
                .iter()
                .map(|t| Tree {
                    name: t.lexeme.clone().unwrap_or("".to_owned()).to_string(),
                    children: None,
                    span: t.span,
                })
                .collect();
            return Some(children);
        }
        return None;
    }
}

impl<'a> VisitorMut<Tree, ()> for TreeBuilder {
    fn visit_expr(&mut self, e: &Expr) -> Result<Tree, ()> {
        let tree = match e {
            Expr::Literal {
                raw: _,
                span,
                value,
                id,
            } => {
                let mut children: Vec<Tree> = vec![];
                self.get_children(*id).map(|c| children.extend(c));
                Tree {
                    name: match value {
                        Literal::Str(_) => "String".to_string(),
                        Literal::Num(_) => "Number".to_string(),
                        Literal::Bool(_) => "Boolean".to_string(),
                        Literal::Array(exprs) => {
                            for expr in exprs {
                                children.push(self.visit_expr(expr)?);
                            }
                            "Array".to_string()
                        },
                        Literal::Object(exprs) => {
                            for (key, value) in exprs {
                                // children.push(self.visit_expr(key)?);
                                children.push(self.visit_expr(value)?);
                            }
                            "Object".to_string()
                        },
                        Literal::Null => "Null".to_string(),
                        Literal::Undefined => "Undefined".to_string(),
                        Literal::NaN => "NaN".to_string(),
                    },
                    children: Some(children),
                    span: *span,
                }
            },
            Expr::Variable { name, span, id } => {
                let mut children = vec![
                    Tree {
                        name: "identifier".to_string(),
                        children: None,
                        span: name.span,
                    },
                ];
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: "Variable".to_owned(),
                    children: Some(children),
                    span: *span,
                }
            },
            Expr::Assignment {
                dst,
                expr,
                span,
                id,
            } => {
                let mut children = vec![Tree {
                    name: "identifier".to_string(),
                    children: None,
                    span: dst.span,
                }];
                children.push(self.visit_expr(expr)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: "Assignment".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Logical {
                left,
                operation,
                right,
                span,
                id,
            } => {
                let mut children = vec![];
                children.push(self.visit_expr(left)?);
                children.push(self.visit_expr(right)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: format!("{:?}", operation),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Call {
                callee,
                args,
                span,
                id,
            } => {
                let mut children = vec![];
                children.push(self.visit_expr(callee)?);
                for arg in args {
                    children.push(self.visit_expr(arg)?);
                }
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: "Call".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Get {
                object,
                name,
                span,
                id,
            } => {
                let mut children = vec![
                    Tree {
                        name: "identifier".to_string(),
                        children: None,
                        span: name.span,
                }];
                children.push(self.visit_expr(object)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: name.name.clone(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Set {
                object,
                name,
                value,
                span,
                id,
            } => {
                let mut children = vec![
                    Tree {
                        name: "identifier".to_string(),
                        children: None,
                        span: name.span,
                }];
                children.push(self.visit_expr(object)?);
                children.push(self.visit_expr(value)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: name.name.clone(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Grouping { expr, span, id } => {
                let mut children = vec![];
                children.push(self.visit_expr(expr)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: "Grouping".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Function {
                name,
                parameters,
                body,
                span,
                id,
            } => {
                let mut children = vec![];

                if let Some(n) = name {
                    children.push(Tree {
                        name: "identifier".to_string(),
                        children: None,
                        span: n.span,
                    });
                }

                for param in parameters {
                    children.push(Tree {
                        name: "identifier".to_string(),
                        children: None,
                        span: param.span,
                    });
                }
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: "Function".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Binary {
                left,
                operation,
                right,
                span,
                id,
            } => {
                let mut children = vec![];
                children.push(self.visit_expr(left)?);
                children.push(self.visit_expr(right)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: format!("{:?}", operation),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Unary {
                operation,
                expr,
                span,
                id,
            } => {
                let mut children = vec![];
                children.push(self.visit_expr(expr)?);
                let token_children = self.get_children(*id);
                if let Some(c) = token_children {
                    children.extend(c);
                }
                Tree {
                    name: format!("{:?}", operation),
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Insert { command, span, id } => Tree {
                name: format!("{:?}", command),
                children: self.get_children(*id),
                span: *span,
            },
            Expr::Delete { command, span, id } => Tree {
                name: format!("{:?}", command),

                children: self.get_children(*id),
                span: *span,
            },
            Expr::Update { command, span, id } => Tree {
                name: format!("{:?}", command),

                children: self.get_children(*id),
                span: *span,
            },
            Expr::Select { query, span, id } => Tree {
                name: "Select".to_string(),
                children: self.get_children(*id),
                span: *span,
            },
        };
        Ok(tree)
    }

    fn visit_stmt(&mut self, s: &Stmt) -> Result<Tree, ()> {
        let tree = match s {
            Stmt::Program { body: stmts, span } => {
                let mut children = vec![];
                for stmt in stmts {
                    children.push(self.visit_stmt(stmt)?);
                }
                Tree {
                    name: "Program".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            Stmt::Expression { expr, span } => {
                let mut children = vec![];
                children.push(self.visit_expr(expr)?);
                Tree {
                    name: "Expression".to_string(),
                    children: Some(children),
                    span: *span,
                }
            }
            _ => Tree {
                name: "Unknown".to_string(),
                children: None,
                span: Span::default(),
            },
        };
        Ok(tree)
    }
}
