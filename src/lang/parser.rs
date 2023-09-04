
use std::rc::Rc;
use crate::lang::ast::{SqlCompoundOperator, SelectCore, SqlDistinct};

use crate::{kw, sym, skw};
use crate::lang::ast::{Expr, Stmt};
use crate::lang::ast::Expr::Variable;
use crate::lang::token::{Token, TokenType};
use crate::lang::token::TokenType::*;
use crate::lang::token::Keyword;
use crate::lang::token::Keyword::*;
use crate::lang::token::SqlKeyword::*;
use crate::lang::token::Symbol::*;
use crate::runtime::types::RV;

use super::ast::{SqlSelect, SqlProjection, SqlFrom, SqlExpr, SqlTableSubquery};

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
        let mut current_expr: Box<Expr> = $self.$builder()?;
        while $self.match_next_multi(&vec![$($operator,)*]) {
            current_expr = Expr::new_binary((*$self.peek_bw(1)).clone(), current_expr, $self.$builder()?);
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
        let mut expr: Option<Box<Expr>> = None;
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
            false => Expr::new_literal(RV::Null)
        };
        self.expected(sym!(Semicolon))?;
        Ok(Stmt::Declaration(token, expr))
    }

    fn expression(&mut self) -> ParseResult<Box<Expr>> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.or()?;

        if self.match_next(sym!(Equal)) {
            let equals = self.peek_bw(1);
            let value = self.assignment()?;
            match *expr {
                Variable(_, tok) => {
                    return Ok(Expr::new_assignment(tok, value));
                },
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget { line: equals.line });
                },
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.and()?;
        if self.match_next(kw!(Keyword::Or)) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(Expr::new_logical(expr, op.clone(), right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.select()?;
        if self.match_next(kw!(Keyword::And)) {
            let op = self.peek_bw(1);
            let right = self.select()?;
            return Ok(Expr::new_logical(expr, op.clone(), right));
        }
        Ok(expr)
    }

    fn select(&mut self) -> ParseResult<Box<Expr>> {
        if !self.cmp_tok(&skw!(Select)) {
            return self.equality();
        }
        let core = self.select_core()?;
        let mut compounds: Vec<(SqlCompoundOperator, Box<SelectCore>)> = vec![];
        while self.match_next_multi(&vec![ skw!(Union), skw!(Intersect), skw!(Except) ]) {
            let op = self.peek_bw(1);
            let mut compound_op = SqlCompoundOperator::Union;
            if &op.tok_type == &skw!(Union) && !self.cmp_tok(&skw!(All)) {
                compound_op = SqlCompoundOperator::UnionAll;
            }
            else {
                compound_op = match op.tok_type {
                    SqlKeyword(Union) => SqlCompoundOperator::Union,
                    SqlKeyword(Intersect) => SqlCompoundOperator::Intersect,
                    SqlKeyword(Except) => SqlCompoundOperator::Except,
                    _ => return Err(ParseError::UnexpectedToken { line: op.line, token: op.clone() })
                }
            }
            let secondary_core = self.select_core()?;
            compounds.push((compound_op, secondary_core))
        }
        Ok(Expr::new_select(Box::new(SqlSelect {
            core,
            compound: compounds,
            order_by: None,
            limit: None,
            offset: None,
        })))
    }

    fn select_core(&mut self) -> ParseResult<Box<SelectCore>> {
        self.expected(skw!(Select))?;
        let mut distinct = SqlDistinct::All;
        if self.match_next(skw!(Distinct)) {
            distinct = SqlDistinct::Distinct;
        }
        else if self.match_next(skw!(All)) {
            distinct = SqlDistinct::All;
        }
        let projection = self.sql_projection();
        let from = self.sql_from()?;
        Ok(Box::new(SelectCore {
            distinct,
            projection,
            from,
            r#where: None,
            group_by: None,
            having: None,
        }))
    }

    fn sql_projection(&mut self) -> Vec<SqlProjection> {
        let mut projections: Vec<SqlProjection> = vec![];
        loop {
            if self.match_next(sym!(Star)) {
                projections.push(SqlProjection::All);
            }
            else {
                let expr = self.expression().unwrap();
                let mut alias: Option<Token> = None;
                if self.match_next(skw!(As)) {
                    let token = self.expected(Identifier { dollar: false });
                    alias = Some(token.unwrap().clone());
                }
                else if self.match_next(Identifier { dollar: false }) {
                    let token = self.peek_bw(1);
                    alias = Some(token.clone());
                }
                projections.push(SqlProjection::Complex { expr: SqlExpr::Default(*expr), alias });
            }
            if !self.match_next(sym!(Comma)) {
                break;
            }
        }
        projections
    }   

    fn sql_from(&mut self) -> ParseResult<Option<SqlFrom>> {
        if self.match_next(skw!(From)) {
            let token = self.expected(Identifier { dollar: false });
            return Ok(Some(SqlFrom::TableSubquery(vec![SqlTableSubquery::Simple { namespace: None, table: token.unwrap().clone(), alias: None }])));
        }
        Ok(None)
    }

    fn equality(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(BangEqual), sym!(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(Greater), sym!(GreaterEqual), sym!(Less), sym!(LessEqual)], term);
    }

    fn term(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self) -> ParseResult<Box<Expr>> {
        if self.match_next_multi(&vec![sym!(Minus), sym!(Bang)]) {
            return Ok(Expr::new_unary((*self.peek_bw(1)).clone(), self.unary()?));
        }
        self.call()
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> ParseResult<Box<Expr>> {
        let mut arguments: Vec<Box<Expr>> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            arguments.push(self.expression()?);
            while self.match_next(sym!(Comma)) {
                arguments.push(self.expression()?);
            }
        }
        let paren = self.expected(sym!(RightParen))?;

        Ok(Expr::new_call(callee, paren.clone(), arguments))
    }

    fn call(&mut self) -> ParseResult<Box<Expr>> {
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

    fn primary(&mut self) -> ParseResult<Box<Expr>> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            True => Ok(Expr::new_literal(RV::Bool(true))),
            False => Ok(Expr::new_literal(RV::Bool(false))),
            TokenType::Null => Ok(Expr::new_literal(RV::Null)),
            Str | Num => Ok(Expr::new_literal(tok.literal.clone().unwrap())),
            Identifier { dollar: _ } => Ok(Expr::new_variable(tok.clone())),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(sym!(RightParen))?;
                Ok(Expr::new_grouping(expr))
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


#[cfg(test)]
mod test {

    use crate::lang::{ast::{Expr, Stmt}, scanner::Scanner, token::Token};

    use crate::lexm;

    use super::*;

    fn get_tokens(source: &str) -> Vec<Token> {
        return Scanner::scan(source).unwrap();
    }

    fn compare_parsed_to_expected(source: &str, expected: Vec<Stmt>) {
        let tokens = get_tokens(source);
        let parsed = Parser::parse(&tokens).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_literal_expression() {
        compare_parsed_to_expected("1;", vec![
            Stmt::Expression(
                Expr::new_literal(RV::Num(1.0))
            )
        ]);
    }

    #[test]
    fn test_parse_unary_expression() {
        compare_parsed_to_expected("-1;", vec![
            Stmt::Expression(
                Expr::new_unary(
                    Token {tok_type: sym!(Minus), lexeme: lexm!("-"), literal: None, line: 0},
                    Expr::new_literal(RV::Num(1.0)
                )
            )
        )]);
    }
}