use crate::lang::parsing::expr::Expr;
use crate::lang::parsing::expr::Expr::{Grouping, Literal};
use crate::lang::parsing::token::{Helper, LiteralValue, Token, TokenType};
use crate::lang::parsing::token::Equality::{BangEqual, EqualEqual, Greater, GreaterEqual, Less, LessEqual};
use crate::lang::parsing::token::Operator::{Bang, Minus, Plus, Slash, Star};
use crate::lang::parsing::token::TokenType::{EOF, Equality, Operator};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

macro_rules! binary {
    ($self: ident,[$($operator:expr),*], $builder: ident) => {
        let mut current_expr: Box<Expr> = $self.$builder();
        while $self.match_next(&vec![$($operator,)*]) {
            current_expr = Box::from(Expr::Binary((*$self.peek(1)).clone(), current_expr, $self.$builder()));
        }
        return current_expr;
    }
}

impl<'a> Parser<'a> {

    pub fn parse(tokens: &Vec<Token>) -> Box<Expr> {
        let mut parser = Parser {
            tokens,
            current: 0
        };
        parser.expression()
    }

    fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Box<Expr> {
        binary!(self, [Equality(BangEqual), Equality(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> Box<Expr> {
        binary!(self, [Equality(Greater), Equality(GreaterEqual), Equality(Less), Equality(LessEqual)], term);
    }

    fn term(&mut self) -> Box<Expr> {
        binary!(self, [Operator(Plus), Operator(Minus)], factor);
    }

    fn factor(&mut self) -> Box<Expr> {
        binary!(self, [Operator(Star), Operator(Slash)], unary);
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.match_next(&vec![Operator(Minus), Operator(Bang)]) {
            return Box::from(Expr::Unary((*self.peek(1)).clone(), self.unary()));
        }
        self.primary()
    }

    fn primary(&mut self) -> Box<Expr> {
        let tok = self.peek(0);
        self.current += 1;
        match &tok.tok_type {
            TokenType::True => Box::from(Literal(LiteralValue::Bool(true))),
            TokenType::False => Box::from(Literal(LiteralValue::Bool(false))),
            TokenType::Nil => Box::from(Literal(LiteralValue::Nil)),
            TokenType::Str | TokenType::Num => Box::from(Literal(tok.literal.clone().unwrap())),
            TokenType::Helper(Helper::LeftParen) => {
                let expr = self.expression();
                self.consume(TokenType::Helper(Helper::RightParen), "Expected ')' after expression.");
                Box::from(Grouping(expr))
            }
            _ => panic!("Unexpected token: {}", tok.lexeme.clone().unwrap_or("<>".to_string()))
        }
    }

    fn consume(&mut self, expected_tok_type: TokenType, error_msg: &str) {
        if self.cmp_tok(&expected_tok_type) {
            self.advance();
            return;
        }
        panic!("{}", error_msg);
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek(1)
    }

    fn is_at_end(&self) -> bool {
        self.peek(0).tok_type == EOF
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