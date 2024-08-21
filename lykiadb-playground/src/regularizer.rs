use lykiadb_lang::{
    tokenizer::token::{Token, TokenType},
    Span,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Tree {
    pub name: String,
    pub children: Option<Vec<Tree>>,
    pub span: Span,
}

pub struct TreeBuilder;

impl TreeBuilder {
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
            },
        }
    }
}
