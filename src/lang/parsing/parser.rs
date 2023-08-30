use std::rc::Rc;
use crate::{kw, sym};
use crate::lang::parsing::ast::{BExpr, Expr, Stmt};
use crate::lang::parsing::ast::Expr::{Variable};
use crate::lang::parsing::token::{LiteralValue, Token, TokenType};
use crate::lang::parsing::token::TokenType::*;
use crate::lang::parsing::token::Keyword::*;
use crate::lang::parsing::token::Symbol::*;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { line: u32, token: Token },
    MissingToken { line: u32, token: Token, description: String},
    InvalidAssignmentTarget { line: u32 }
}

type ParseResult<T> = Result<T, ParseError>;

macro_rules! binary {
    ($self: ident, [$($operator:expr),*], $builder: ident) => {
        let mut current_expr: BExpr = $self.$builder()?;
        while $self.match_next_multi(&vec![$($operator,)*]) {
            current_expr = Box::from(Expr::new_binary((*$self.peek_bw(1)).clone(), current_expr, $self.$builder()?));
        }
        return Ok(current_expr);
    }
}

// a macro for repeating match_next pattern
macro_rules! match_next {
    ($self: ident, $t: expr, $callee: ident) => {
        if $self.match_next($t) {
            return $self.$callee();
        }
    }
}

impl<'a> Parser<'a> {

    pub fn parse(tokens: &Vec<Token>) -> ParseResult<Vec<Stmt>> {
        let mut parser = Parser {
            tokens,
            current: 0
        };
        parser.program()
    }

    fn program(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.expected(Eof)?;
        Ok(statements)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        match_next!(self, kw!(Var), var_declaration);
        match_next!(self, kw!(Fun), fun_declaration);
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        match_next!(self, kw!(If), if_statement);
        match_next!(self, kw!(While), while_statement);
        match_next!(self, kw!(For), for_statement);
        match_next!(self, kw!(Loop), loop_statement);
        match_next!(self, kw!(Break), break_statement);
        match_next!(self, kw!(Continue), continue_statement);
        match_next!(self, kw!(Return), return_statement);
        match_next!(self, sym!(LeftBrace), block);
        self.expression_statement()
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let if_branch = self.statement()?;

        if self.match_next(kw!(Else)) {
            let else_branch = self.statement()?;
            return Ok(Stmt::If(condition, Box::from(if_branch), Some(Box::from(else_branch))));
        }
        Ok(Stmt::If(condition, Box::from(if_branch), None))
    }

    fn loop_statement(&mut self) -> ParseResult<Stmt> {
        let inner_stmt = self.declaration()?;
        Ok(Stmt::Loop(None, Box::from(inner_stmt), None))
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let inner_stmt = self.declaration()?;

        Ok(Stmt::Loop(Some(condition), Box::from(inner_stmt), None))
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let tok = self.peek_bw(1);
        let mut expr: Option<BExpr> = None;
        if !self.cmp_tok(&sym!(Semicolon)) {
            expr = Some(self.expression()?);
        }
        self.expected(sym!(Semicolon))?;

        Ok(Stmt::Return(tok.clone(), expr))
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.expected(sym!(LeftParen))?;

        let initializer = if self.match_next(sym!(Semicolon)) { None } else { Some(self.declaration()?) };

        let condition = if self.match_next(sym!(Semicolon)) { None }
        else {
            let wrapped = self.expression()?;
            self.expected(sym!(Semicolon))?;
            Some(wrapped)
        };

        let increment = if self.match_next(sym!(RightParen)) { None }
        else {
            let wrapped = self.expression()?;
            self.expected(sym!(RightParen))?;
            Some(Box::from(Stmt::Expression(wrapped)))
        };

        let inner_stmt = Box::from(self.declaration()?);

        if initializer.is_none() {
            return Ok(Stmt::Loop(condition,inner_stmt, increment));
        }
        Ok(Stmt::Block(vec![
            initializer.unwrap(),
            Stmt::Loop(condition, inner_stmt, increment)
        ]))
    }

    fn block(&mut self) -> ParseResult<Stmt> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.cmp_tok(&sym!(RightBrace)) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.expected(sym!(RightBrace))?;

        Ok(Stmt::Block(statements))
    }

    fn break_statement(&mut self) -> ParseResult<Stmt> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(Stmt::Break(tok.clone()))
    }

    fn continue_statement(&mut self) -> ParseResult<Stmt> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(Stmt::Continue(tok.clone()))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.expected(sym!(Semicolon))?;
        Ok(Stmt::Expression(expr))
    }

    fn fun_declaration(&mut self) -> ParseResult<Stmt> {
        let token = self.expected(Identifier { dollar: false })?.clone();
        self.expected(sym!(LeftParen))?;
        let mut parameters: Vec<Token> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            let p = self.expected(Identifier { dollar: true })?;
            parameters.push(p.clone());
            while self.match_next(sym!(Comma)) {
                let q = self.expected(Identifier { dollar: true })?;
                parameters.push(q.clone());
            }
        }
        self.expected(sym!(RightParen))?;
        self.expected(sym!(LeftBrace))?;
        let block = self.block()?;

        let body = match block {
            Stmt::Block(stmts) => stmts,
            _ => vec![]
        };

        Ok(Stmt::Function(token, parameters, Rc::new(body)))
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let token = self.expected(Identifier { dollar: true })?.clone();
        let expr = match self.match_next(sym!(Equal)) {
            true => self.expression()?,
            false => Box::from(Expr::new_literal(LiteralValue::Nil))
        };
        self.expected(sym!(Semicolon))?;
        Ok(Stmt::Declaration(token, expr))
    }

    fn expression(&mut self) -> ParseResult<BExpr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<BExpr> {
        let expr = self.or()?;

        if self.match_next(sym!(Equal)) {
            let equals = self.peek_bw(1);
            let value = self.assignment()?;
            match *expr {
                Variable(_, tok) => {
                    return Ok(Box::from(Expr::new_assignment(tok, value)));
                },
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget { line: equals.line });
                },
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<BExpr> {
        let expr = self.and()?;
        if self.match_next(kw!(Or)) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(Box::from(Expr::new_logical(expr, op.clone(), right)));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<BExpr> {
        let expr = self.equality()?;
        if self.match_next(kw!(And)) {
            let op = self.peek_bw(1);
            let right = self.equality()?;
            return Ok(Box::from(Expr::new_logical(expr, op.clone(), right)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<BExpr> {
        binary!(self, [sym!(BangEqual), sym!(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> ParseResult<BExpr> {
        binary!(self, [sym!(Greater), sym!(GreaterEqual), sym!(Less), sym!(LessEqual)], term);
    }

    fn term(&mut self) -> ParseResult<BExpr> {
        binary!(self, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self) -> ParseResult<BExpr> {
        binary!(self, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self) -> ParseResult<BExpr> {
        if self.match_next_multi(&vec![sym!(Minus), sym!(Bang)]) {
            return Ok(Box::from(Expr::new_unary((*self.peek_bw(1)).clone(), self.unary()?)));
        }
        self.call()
    }

    fn finish_call(&mut self, callee: BExpr) -> ParseResult<BExpr> {
        let mut arguments: Vec<BExpr> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            arguments.push(self.expression()?);
            while self.match_next(sym!(Comma)) {
                arguments.push(self.expression()?);
            }
        }
        let paren = self.expected(sym!(RightParen))?;

        Ok(Box::from(Expr::new_call(callee, paren.clone(), arguments)))
    }

    fn call(&mut self) -> ParseResult<BExpr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_next(sym!(LeftParen)) {
                expr = self.finish_call(expr)?;
            }
            else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<BExpr> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            True => Ok(Box::from(Expr::new_literal(LiteralValue::Bool(true)))),
            False => Ok(Box::from(Expr::new_literal(LiteralValue::Bool(false)))),
            Nil => Ok(Box::from(Expr::new_literal(LiteralValue::Nil))),
            Str | Num => Ok(Box::from(Expr::new_literal(tok.literal.clone().unwrap()))),
            Identifier { dollar: _ } => Ok(Box::from(Expr::new_variable(tok.clone()))),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(sym!(RightParen))?;
                Ok(Box::from(Expr::new_grouping(expr)))
            },
            _ => {
                Err(ParseError::UnexpectedToken { line: tok.line, token: tok.clone() })
            },
        }
    }

    fn expected(&mut self, expected_tok_type: TokenType) -> ParseResult<&Token> {
        if self.cmp_tok(&expected_tok_type) {
            return Ok(self.advance());
        };
        let prev_token = self.peek_bw(1);
        Err(ParseError::MissingToken {
            line: prev_token.line,
            token: prev_token.clone(),
            description: format!("Expected '{:?}' after {:?}", expected_tok_type, self.peek_bw(1))
        })
    }

    fn advance(&mut self) -> &'a Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.peek_bw(1)
    }

    fn is_at_end(&self) -> bool {
        self.cmp_tok(&Eof)
    }

    fn peek_bw(&self, offset: usize) -> &'a Token {
        &self.tokens[self.current - offset]
    }

    fn cmp_tok(&self, t: &TokenType) -> bool {
        let current = self.peek_bw(0);
        current.tok_type == *t
    }

    fn match_next(&mut self, t: TokenType) -> bool {
        if self.cmp_tok(&t) {
            self.advance();
            return true;
        }
        false
    }

    fn match_next_multi(&mut self, types: &Vec<TokenType>) -> bool {
        for t in types {
            if self.cmp_tok(t) {
                self.advance();
                return true;
            }
        }
        false
    }
}