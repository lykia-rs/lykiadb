use std::process::exit;
use crate::lang::parsing::error::parse_err;
use crate::lang::parsing::ast::{BExpr, Expr, Stmt};
use crate::lang::parsing::ast::Expr::{Grouping, Literal};
use crate::lang::parsing::token::{LiteralValue, Token, TokenType};
use crate::lang::parsing::token::TokenType::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

macro_rules! binary {
    ($self: ident,[$($operator:expr),*], $builder: ident) => {
        let mut current_expr: BExpr = $self.$builder();
        while $self.match_next(&vec![$($operator,)*]) {
            current_expr = Box::from(Expr::Binary((*$self.peek(1)).clone(), current_expr, $self.$builder()));
        }
        return current_expr;
    }
}

impl<'a> Parser<'a> {

    pub fn parse(tokens: &Vec<Token>) -> Vec<Stmt> {
        let mut parser = Parser {
            tokens,
            current: 0
        };
        // println!("DebugExp: {:?}", tokens);
        parser.program()
    }

    fn program(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement());
        }
        self.consume(EOF, "Expected EOF char at the end of file");
        statements
    }

    fn statement(&mut self) -> Stmt {
        if self.match_next(&vec![Print]) {
            return self.print_statement();
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(Semicolon, "Expected ; after value");
        Stmt::Print(expr)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(Semicolon, "Expected ; after expression");
        Stmt::Expression(expr)
    }

    fn expression(&mut self) -> BExpr {
        self.equality()
    }

    fn equality(&mut self) -> BExpr {
        binary!(self, [BangEqual, EqualEqual], comparison);
    }

    fn comparison(&mut self) -> BExpr {
        binary!(self, [Greater, GreaterEqual, Less, LessEqual], term);
    }

    fn term(&mut self) -> BExpr {
        binary!(self, [Plus, Minus], factor);
    }

    fn factor(&mut self) -> BExpr {
        binary!(self, [Star, Slash], unary);
    }

    fn unary(&mut self) -> BExpr {
        if self.match_next(&vec![Minus, Bang]) {
            return Box::from(Expr::Unary((*self.peek(1)).clone(), self.unary()));
        }
        self.primary()
    }

    fn primary(&mut self) -> BExpr {
        let tok = self.peek(0);
        // println!("Debug: {:?}", tok);
        self.current += 1;
        match &tok.tok_type {
            True => Box::from(Literal(LiteralValue::Bool(true))),
            False => Box::from(Literal(LiteralValue::Bool(false))),
            Nil => Box::from(Literal(LiteralValue::Nil)),
            Str | Num => Box::from(Literal(tok.literal.clone().unwrap())),
            LeftParen => {
                let expr = self.expression();
                self.consume(RightParen, "Expected ')' after expression");
                Box::from(Grouping(expr))
            },
            _ => {
                parse_err(&format!("Unexpected token: '{}'", tok.lexeme.clone().unwrap_or("<>".to_string())), tok.line);
                exit(1);
            }
        }
    }

    fn consume(&mut self, expected_tok_type: TokenType, error_msg: &str) {
        if self.cmp_tok(&expected_tok_type) {
            self.advance();
            return;
        }
        parse_err(&error_msg.to_string(), self.peek(0).line);
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek(1)
    }

    fn is_at_end(&self) -> bool {
        self.cmp_tok(&EOF)
    }

    fn peek(&self, offset: usize) -> &'a Token {
        &self.tokens[self.current - offset]
    }

    fn cmp_tok(&self, t: &TokenType) -> bool {
        let current = self.peek(0);
        current.tok_type == *t
    }

    fn match_next(&mut self, types: &Vec<TokenType>) -> bool {
        for t in types {
            if self.cmp_tok(t) {
                self.advance();
                return true;
            }
        }
        false
    }
}