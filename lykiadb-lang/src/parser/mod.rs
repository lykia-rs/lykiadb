use self::program::Program;
use super::ast::expr::Operation;
use super::ast::stmt::Stmt;
use crate::ast::expr::Expr;
use crate::ast::{Span, Spanned};
use crate::tokenizer::token::{SqlKeyword, Symbol::*, Token, TokenType, TokenType::*};
use expr::ExprParser;
use lykiadb_common::error::InputError;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use sql::SqlParser;
use stmt::StmtParser;

mod expr;
mod sql;
mod stmt;

pub mod program;
pub mod resolver;

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    expr_id: usize,
    counters: FxHashMap<String, usize>,
}

impl<'a> Parser<'a> {
    pub fn create(tokens: &Vec<Token>) -> Parser {
        Parser {
            tokens,
            expr_id: 0,
            counters: FxHashMap::default(),
        }
    }

    pub fn get_count(&self, name: &str) -> usize {
        *self.counters.get(name).unwrap_or(&0)
    }

    pub fn increment_count(&mut self, name: &str) {
        let count = self.get_count(name);
        self.counters.insert(name.to_string(), count + 1);
    }

    pub fn decrement_count(&mut self, name: &str) {
        let count = self.get_count(name);
        self.counters.insert(name.to_string(), count - 1);
    }

    pub fn parse(tokens: &Vec<Token>) -> ParseResult<Program> {
        if tokens.is_empty() || tokens.first().unwrap().tok_type == Eof {
            return Err(ParseError::NoTokens);
        }
        let mut parser = Parser {
            tokens,
            expr_id: 0,
            counters: FxHashMap::default(),
        };
        let program = parser.program()?;
        Ok(Program::new(program))
    }

    fn get_expr_id(&mut self) -> usize {
        let id = self.expr_id;
        self.expr_id += 1;
        id
    }

    fn consume_expr(&mut self) -> ParseResult<Box<Expr>> {
        let mut expr = ExprParser {};
        expr.expression(self)
    }

    fn consume_call(&mut self) -> ParseResult<Box<Expr>> {
        let mut expr = SqlParser {};
        expr.sql_insert(self)
    }

    fn consume_call2(&mut self) -> ParseResult<Box<Expr>> {
        let mut expr = ExprParser {};
        expr.call(self)
    }

    fn consume_block(&mut self) -> ParseResult<Box<Stmt>> {
        let mut stmt = StmtParser {};
        stmt.block(self)
    }

    fn consume_declaration(&mut self) -> ParseResult<Box<Stmt>> {
        let mut stmt = StmtParser {};
        stmt.declaration(self)
    }

    pub fn program(&mut self) -> ParseResult<Box<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_at_end() {
            statements.push(*self.consume_declaration()?);
        }
        self.expect(&Eof)?;
        Ok(Box::new(Stmt::Program {
            body: statements.clone(),
            span: self.get_merged_span(&(statements[0]), &(statements[statements.len() - 1])),
        }))
    }

    /*
    fn expect_type_annotation(&mut self) -> ParseResult<TypeAnnotation> {
        let start_tok = self.peek_bw(0);
        let expr = self.consume_expr()?;

        let last_span = expr.get_span();
        Ok(TypeAnnotation {
            type_expr: expr,
            span: self.get_merged_span(&start_tok.span, &last_span),
        })
    }
    */

    fn expect(&mut self, expected_tok_type: &TokenType) -> ParseResult<&Token> {
        if self.cmp_tok(expected_tok_type) {
            return Ok(self.advance());
        };
        let prev_token = self.peek_bw(1);
        Err(ParseError::MissingToken {
            token: prev_token.clone(),
            expected: expected_tok_type.clone(),
        })
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.increment_count("current");
        }
        self.peek_bw(1)
    }

    fn is_at_end(&self) -> bool {
        self.cmp_tok(&Eof)
    }

    fn peek_bw(&self, offset: usize) -> &'a Token {
        &self.tokens[self.get_count("current") - offset]
    }

    fn peek_fw(&self, offset: usize) -> &'a Token {
        &self.tokens[self.get_count("current") + offset]
    }

    fn cmp_tok(&self, t: &TokenType) -> bool {
        let current = self.peek_bw(0);
        current.tok_type == *t
    }

    fn match_next(&mut self, t: &TokenType) -> bool {
        if self.cmp_tok(t) {
            self.advance();
            return true;
        }
        false
    }

    fn peek_next_all_of(&mut self, tokens: &[TokenType]) -> bool {
        for (i, t) in tokens.iter().enumerate() {
            if self.peek_fw(i).tok_type != *t {
                return false;
            }
        }
        true
    }

    fn match_next_all_of(&mut self, tokens: &[TokenType]) -> bool {
        for (i, t) in tokens.iter().enumerate() {
            if self.peek_fw(i).tok_type != *t {
                return false;
            }
        }
        for _ in 0..tokens.len() {
            self.advance();
        }
        true
    }

    fn match_next_one_of(&mut self, tokens: &[TokenType]) -> bool {
        for t in tokens {
            if self.cmp_tok(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn tok_type_to_op(&self, tok_t: TokenType) -> Operation {
        match tok_t {
            TokenType::Symbol(sym) => match sym {
                Plus => Operation::Add,
                Minus => Operation::Subtract,
                Star => Operation::Multiply,
                Slash => Operation::Divide,
                EqualEqual => Operation::IsEqual,
                BangEqual => Operation::IsNotEqual,
                Greater => Operation::Greater,
                GreaterEqual => Operation::GreaterEqual,
                Less => Operation::Less,
                LessEqual => Operation::LessEqual,
                Bang => Operation::Not,
                LogicalAnd => Operation::And,
                LogicalOr => Operation::Or,
                Equal => {
                    if self.get_count("in_select_depth") > 0 {
                        Operation::IsEqual
                    } else {
                        unreachable!()
                    }
                }
                _ => unreachable!(),
            },
            TokenType::SqlKeyword(skw) => match skw {
                SqlKeyword::And => Operation::And,
                SqlKeyword::Or => Operation::Or,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn get_merged_span(&self, left: &impl Spanned, right: &impl Spanned) -> Span {
        let left_span = &left.get_span();
        let right_span = &right.get_span();
        left_span.merge(right_span)
    }
}

#[derive(thiserror::Error, PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    #[error("Unexpected token {token:?}")]
    UnexpectedToken { token: Token },
    #[error("Missing token. Expected {expected:?} after {token:?}")]
    MissingToken { token: Token, expected: TokenType },
    #[error("Invalid assignment target {left:?}")]
    InvalidAssignmentTarget { left: Token },
    #[error("Missing identifier")]
    MissingIdentifier { token : Token },
    #[error("Empty token literal")]
    EmptyTokenLiteral { token: Token },
    #[error("Empty token lexeme")]
    EmptyTokenLexeme { token: Token },
    #[error("No tokens to parse")]
    NoTokens,
}

impl From<ParseError> for InputError {
    fn from(value: ParseError) -> Self {
        let (hint, sp) = match &value {
            ParseError::UnexpectedToken { token } => (
                "Check the syntax and ensure tokens are in the correct order",
                token.span,
            ),
            ParseError::MissingToken { token, .. } => (
                "Add the required token or check for syntax errors",
                token.span,
            ),
            ParseError::InvalidAssignmentTarget { left } => (
                "Ensure the left side of assignment is a valid variable or identifier",
                left.span,
            ),
            ParseError::MissingIdentifier { token } => (
                "Provide a valid identifier",
                token.span,
            ),
            ParseError::EmptyTokenLiteral { token } => (
                "Provide a valid literal value",
                token.span,
            ),
            ParseError::EmptyTokenLexeme { token } => (
                "Provide a valid lexeme",
                token.span,
            ),
            ParseError::NoTokens => ("Provide valid input to parse", Span::default()),
        };

        InputError::new(&value.to_string(), hint, Some(sp.into()))
    }
}
