use std::process::exit;
use crate::lang::parsing::error::parse_err;
use crate::lang::parsing::ast::{BExpr, Expr, Stmt};
use crate::lang::parsing::ast::Expr::{Assignment, Grouping, Literal, Logical, Variable};
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
        parser.program()
    }

    fn program(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration());
        }
        self.consume(Eof, "Expected EOF char at the end of file");
        statements
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_next(&vec![Var]) {
            return self.var_declaration()
        }
        self.statement()
    }

    fn statement(&mut self) -> Stmt {
        if self.match_next(&vec![If]) {
            return self.if_statement();
        }
        if self.match_next(&vec![While]) {
            return self.while_statement();
        }
        if self.match_next(&vec![For]) {
            return self.for_statement();
        }
        if self.match_next(&vec![Print]) {
            return self.print_statement();
        }
        if self.match_next(&vec![Break]) {
            return self.break_statement();
        }
        if self.match_next(&vec![Continue]) {
            return self.continue_statement();
        }
        if self.match_next(&vec![LeftBrace]) {
            return self.block();
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after if.");
        let condition = self.expression();
        self.consume(RightParen, "Expected ')' after if condition.");
        let if_branch = self.statement();

        if self.match_next(&vec![Else]) {
            let else_branch = self.statement();
            return Stmt::If(condition, Box::from(if_branch), Some(Box::from(else_branch)));
        }
        Stmt::If(condition, Box::from(if_branch), None)
    }

    fn while_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after while.");
        let condition = self.expression();
        self.consume(RightParen, "Expected ')' after while condition.");
        let inner_stmt = self.declaration();

        Stmt::Loop(Some(condition), Box::from(inner_stmt), None)
    }

    fn for_statement(&mut self) -> Stmt {
        self.consume(LeftParen, "Expected '(' after for.");

        let initializer = if self.match_next(&vec![Semicolon]) { None } else { Some(self.declaration()) };

        let condition = if self.match_next(&vec![Semicolon]) { None }
        else {
            let q = Some(self.expression());
            self.consume(Semicolon, "Expected ';' after expression.");
            q
        };

        let increment = if self.match_next(&vec![RightParen]) { None }
        else {
            let q = self.expression();
            self.consume(RightParen, "Expected ')' after body.");
            Some(Box::from(Stmt::Expression(q)))
        };

        let inner_stmt = self.declaration();

        if initializer.is_none() {
            return Stmt::Block(vec![
                Stmt::Loop(condition,
                           Box::from(inner_stmt),
                           increment
                )
            ])
        }
        Stmt::Block(vec![
            initializer.unwrap(),
            Stmt::Loop(condition,
                       Box::from(inner_stmt),
                       increment
            )
        ])
    }

    fn block(&mut self) -> Stmt {
        let mut statements: Vec<Stmt> = vec![];

        while !self.cmp_tok(&RightBrace) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        self.consume(RightBrace, "Expected '}' after block.");

        Stmt::Block(statements)
    }

    fn print_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(Semicolon, "Expected ';' after value");
        Stmt::Print(expr)
    }

    fn clock_expr(&mut self) -> BExpr {
        Box::from(Expr::Clock())
    }

    fn break_statement(&mut self) -> Stmt {
        let tok = self.peek(1);
        self.consume(Semicolon, "Expected ';' after value");
        return Stmt::Break(tok.clone());
    }

    fn continue_statement(&mut self) -> Stmt {
        let tok = self.peek(1);
        self.consume(Semicolon, "Expected ';' after value");
        return Stmt::Continue(tok.clone());
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(Semicolon, "Expected ';' after expression");
        Stmt::Expression(expr)
    }

    fn var_declaration(&mut self) -> Stmt {
        let token = self.consume(Identifier, "Expected identifier after 'var'").clone();
        let expr = match self.match_next(&vec![Equal]) {
            true => self.expression(),
            false => Box::from(Literal(LiteralValue::Nil))
        };
        self.consume(Semicolon, "Expected ';' after expression");
        Stmt::Declaration(token, expr)
    }

    fn expression(&mut self) -> BExpr {
        self.assignment()
    }

    fn assignment(&mut self) -> BExpr {
        let expr = self.or();

        if self.match_next(&vec![Equal]) {
            let equals = self.peek(1);
            let value = self.assignment();
            match *expr {
                Variable(tok) => {
                    return Box::from(Assignment(tok, value));
                },
                _ => {
                    parse_err("Invalid assignment target", equals.line);
                    exit(1);
                },
            }
        }
        expr
    }

    fn or(&mut self) -> BExpr {
        let expr = self.and();
        if self.match_next(&vec![Or]) {
            let op = self.peek(1);
            let right = self.and();
            return Box::from(Logical(expr, op.clone(), right));
        }
        expr
    }

    fn and(&mut self) -> BExpr {
        let expr = self.equality();
        if self.match_next(&vec![And]) {
            let op = self.peek(1);
            let right = self.equality();
            return Box::from(Logical(expr, op.clone(), right));
        }
        expr
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
        if self.match_next(&vec![Clock]) {
            return self.clock_expr();
        }
        self.primary()
    }

    fn primary(&mut self) -> BExpr {
        let tok = self.peek(0);
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
            Identifier => Box::from(Variable(tok.clone())),
            _ => {
                parse_err(&format!("Unexpected token '{}'", tok.lexeme.clone().unwrap()), tok.line);
                exit(1);
            },
        }
    }

    fn consume(&mut self, expected_tok_type: TokenType, error_msg: &str) -> &Token {
        if self.cmp_tok(&expected_tok_type) {
            return self.advance();
        }
        parse_err(error_msg, self.peek(0).line);
        exit(1);
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek(1)
    }

    fn is_at_end(&self) -> bool {
        self.cmp_tok(&Eof)
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