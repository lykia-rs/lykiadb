use std::process::exit;
use crate::lang::parsing::error::parse_err;
use crate::lang::parsing::expr::{Ast, Expr};
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
        let mut current_expr: Ast = $self.$builder();
        while $self.match_next(&vec![$($operator,)*]) {
            current_expr = Box::from(Expr::Binary((*$self.peek(1)).clone(), current_expr, $self.$builder()));
        }
        return current_expr;
    }
}

impl<'a> Parser<'a> {

    pub fn parse(tokens: &Vec<Token>) -> Ast {
        let mut parser = Parser {
            tokens,
            current: 0
        };
        println!("DebugExp: {:?}", tokens);
        parser.expression()
    }

    fn expression(&mut self) -> Ast {
        self.equality()
    }

    fn equality(&mut self) -> Ast {
        binary!(self, [Equality(BangEqual), Equality(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> Ast {
        binary!(self, [Equality(Greater), Equality(GreaterEqual), Equality(Less), Equality(LessEqual)], term);
    }

    fn term(&mut self) -> Ast {
        binary!(self, [Operator(Plus), Operator(Minus)], factor);
    }

    fn factor(&mut self) -> Ast {
        binary!(self, [Operator(Star), Operator(Slash)], unary);
    }

    fn unary(&mut self) -> Ast {
        if self.match_next(&vec![Operator(Minus), Operator(Bang)]) {
            return Box::from(Expr::Unary((*self.peek(1)).clone(), self.unary()));
        }
        self.primary()
    }

    fn primary(&mut self) -> Ast {
        let tok = self.peek(0);
        println!("Debug: {:?}", tok);
        self.current += 1;
        match &tok.tok_type {
            TokenType::True => Box::from(Literal(LiteralValue::Bool(true))),
            TokenType::False => Box::from(Literal(LiteralValue::Bool(false))),
            TokenType::Nil => Box::from(Literal(LiteralValue::Nil)),
            TokenType::Str | TokenType::Num => Box::from(Literal(tok.literal.clone().unwrap())),
            TokenType::Helper(Helper::LeftParen) => {
                let expr = self.expression();
                self.consume(TokenType::Helper(Helper::RightParen), "Expected ')' after expression");
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
        parse_err(&format!("{}", error_msg), self.peek(0).line);
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