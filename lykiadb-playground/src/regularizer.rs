use lykiadb_lang::{
    ast::{expr::Expr, stmt::Stmt, visitor::VisitorMut}, parser::NodeMetadata, tokenizer::token::{Token, TokenType}, Literal, Span
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Tree {
    pub name: String,
    pub children: Option<Vec<Tree>>,
    pub span: Span,
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

    pub fn token_to_tree(token: Token) -> Tree {
        match token.tok_type {
            TokenType::Identifier { .. } => Tree {
                name: "Identifier".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Keyword { .. } => Tree {
                name: "Keyword".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::SqlKeyword { .. } => Tree {
                name: "SqlKeyword".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Str { .. } => Tree {
                name: "String".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Num { .. } => Tree {
                name: "Number".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::True { .. } | TokenType::False { .. } => Tree {
                name: "Boolean".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Null { .. } => Tree {
                name: "Null".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Undefined { .. } => Tree {
                name: "Undefined".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Symbol { .. } => Tree {
                name: "Symbol".to_string(),
                children: None,
                span: token.span,
            },
            TokenType::Eof { .. } => Tree {
                name: "Eof".to_string(),
                children: None,
                span: token.span,
            }
        }
    }

    pub fn get_children(&self, id: usize) -> Option<Vec<Tree>> {
        if self.node_metadata.contains_key(&id) {
            let children: Vec<Tree> = self.node_metadata[&id]
                .tokens
                .iter()
                .map(|t| Self::token_to_tree(t.clone()))
                .collect();
            return Some(children);
        }
        return None;
    }

    pub fn get_subtree(&self, name: &str, span: &Span, id: usize, mut children: Vec<Tree>) -> Tree {
        let token_children = self.get_children(id);
        if let Some(c) = token_children {
            children.extend(c);
        }
        Tree {
            name: name.to_owned(),
            children: Some(children),
            span: *span,
        }
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
                        }
                        Literal::Object(exprs) => {
                            for (key, value) in exprs {
                                // children.push(self.visit_expr(key)?);
                                children.push(self.visit_expr(value)?);
                            }
                            "Object".to_string()
                        }
                        Literal::Null => "Null".to_string(),
                        Literal::Undefined => "Undefined".to_string(),
                        Literal::NaN => "NaN".to_string(),
                    },
                    children: Some(children),
                    span: *span,
                }
            }
            Expr::Variable { name, span, id } => {
                self.get_subtree("Variable", span, *id, vec![Tree {
                    name: "Identifier".to_string(),
                    children: None,
                    span: name.span,
                }])
            }
            Expr::Assignment {
                dst,
                expr,
                span,
                id,
            } => {
                let children = vec![Tree {
                    name: "Identifier".to_string(),
                    children: None,
                    span: dst.span,
                }, self.visit_expr(expr)?];
                self.get_subtree("Assignment", span, *id, children)
            }
            Expr::Logical {
                left,
                operation,
                right,
                span,
                id,
            } => {
                let children = vec![
                    self.visit_expr(left)?,
                    self.visit_expr(right)?
                ];
                self.get_subtree(&format!("Logical/{:?}", operation), span, *id, children)
            }
            Expr::Call {
                callee,
                args,
                span,
                id,
            } => {
                let mut children = vec![self.visit_expr(callee)?];
                for arg in args {
                    children.push(self.visit_expr(arg)?);
                }
                self.get_subtree("Call", span, *id, children)
            }
            Expr::Get {
                object,
                name,
                span,
                id,
            } => {
                let children = vec![Tree {
                    name: "Identifier".to_string(),
                    children: None,
                    span: name.span,
                }, self.visit_expr(object)?];
                self.get_subtree("Get", span, *id, children)
            }
            Expr::Set {
                object,
                name,
                value,
                span,
                id,
            } => {
                let children = vec![Tree {
                    name: "Identifier".to_string(),
                    children: None,
                    span: name.span,
                }, self.visit_expr(object)?, self.visit_expr(value)?];
                self.get_subtree("Set", span, *id, children)
            }
            Expr::Grouping { expr, span, id } => {
                let children = vec![self.visit_expr(expr)?];
                self.get_subtree("Grouping", span, *id, children)
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
                        name: "Identifier".to_string(),
                        children: None,
                        span: n.span,
                    });
                }

                for param in parameters {
                    children.push(Tree {
                        name: "Identifier".to_string(),
                        children: None,
                        span: param.span,
                    });
                }

                self.get_subtree("Function", span, *id, children)
            }
            Expr::Binary {
                left,
                operation,
                right,
                span,
                id,
            } => {
                let children = vec![
                    self.visit_expr(left)?,
                    self.visit_expr(right)?
                ];
                self.get_subtree(&format!("Binary/{:?}", operation), span, *id, children)
            }
            Expr::Unary {
                operation,
                expr,
                span,
                id,
            } => {
                let children = vec![self.visit_expr(expr)?];
                self.get_subtree(&format!("Unary/{:?}", operation), span, *id, children)
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
