use std::rc::Rc;

use crate::lang::token::Keyword;
use crate::lang::token::Keyword::*;
use crate::lang::token::SqlKeyword::*;
use crate::lang::token::Symbol::*;
use crate::lang::token::TokenType::*;
use crate::lang::token::{Token, TokenType};
use crate::runtime::types::RV;
use crate::{kw, skw, sym};

use super::ast::expr::Expr;
use super::ast::expr::ExprId;
use super::ast::sql::SelectCore;
use super::ast::sql::SqlCompoundOperator;
use super::ast::sql::SqlDistinct;
use super::ast::sql::SqlExpr;
use super::ast::sql::SqlFrom;
use super::ast::sql::SqlProjection;
use super::ast::sql::SqlSelect;
use super::ast::sql::SqlTableSubquery;
use super::ast::stmt::Stmt;
use super::ast::stmt::StmtId;
use super::ast::ParserArena;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    arena: ParserArena,
}

pub struct Parsed {
    pub statements: Vec<StmtId>,
    pub arena: Rc<ParserArena>,
}

impl Parsed {
    pub fn new(statements: Vec<StmtId>, arena: Rc<ParserArena>) -> Parsed {
        Parsed { statements, arena }
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken { token: Token },
    MissingToken { token: Token, expected: TokenType },
    InvalidAssignmentTarget { left: Token },
}

type ParseResult<T> = Result<T, ParseError>;

macro_rules! binary {
    ($self: ident, [$($operator:expr),*], $builder: ident) => {
        let mut current_expr: ExprId = $self.$builder()?;
        while $self.match_next_multi(&vec![$($operator,)*]) {
            let token = (*$self.peek_bw(1)).clone();
            let left = current_expr;
            let right = $self.$builder()?;
            current_expr = $self.arena.expression(Expr::Binary { left, token, right });
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
    };
}

macro_rules! optional_with_expected {
    ($self: ident, $optional: expr, $expected: expr) => {
        if $self.match_next($optional) {
            let token = $self.expected($expected);
            Some(token.unwrap().clone())
        } else if $self.match_next($expected) {
            let token = $self.peek_bw(1);
            Some(token.clone())
        } else {
            None
        }
    };
}

impl<'a> Parser<'a> {
    pub fn parse(tokens: &Vec<Token>) -> ParseResult<Parsed> {
        let arena = ParserArena::new();
        let mut parser = Parser {
            tokens,
            current: 0,
            arena,
        };
        let statements = parser.program()?;
        Ok(Parsed::new(statements, Rc::new(parser.arena)))
    }

    fn program(&mut self) -> ParseResult<Vec<StmtId>> {
        let mut statements: Vec<StmtId> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.expected(Eof)?;
        Ok(statements)
    }

    fn declaration(&mut self) -> ParseResult<StmtId> {
        match_next!(self, kw!(Var), var_declaration);
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<StmtId> {
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

    fn if_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let if_branch = self.statement()?;

        if self.match_next(kw!(Else)) {
            let else_branch = self.statement()?;
            return Ok(self
                .arena
                .statement(Stmt::If(condition, if_branch, Some(else_branch))));
        }
        Ok(self.arena.statement(Stmt::If(condition, if_branch, None)))
    }

    fn loop_statement(&mut self) -> ParseResult<StmtId> {
        let inner_stmt = self.declaration()?;
        Ok(self.arena.statement(Stmt::Loop(None, inner_stmt, None)))
    }

    fn while_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let inner_stmt = self.declaration()?;

        Ok(self
            .arena
            .statement(Stmt::Loop(Some(condition), inner_stmt, None)))
    }

    fn return_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        let mut expr: Option<ExprId> = None;
        if !self.cmp_tok(&sym!(Semicolon)) {
            expr = Some(self.expression()?);
        }
        self.expected(sym!(Semicolon))?;

        Ok(self.arena.statement(Stmt::Return(tok.clone(), expr)))
    }

    fn for_statement(&mut self) -> ParseResult<StmtId> {
        self.expected(sym!(LeftParen))?;

        let initializer = if self.match_next(sym!(Semicolon)) {
            None
        } else {
            Some(self.declaration()?)
        };

        let condition = if self.match_next(sym!(Semicolon)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(sym!(Semicolon))?;
            Some(wrapped)
        };

        let increment = if self.match_next(sym!(RightParen)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(sym!(RightParen))?;
            Some(self.arena.statement(Stmt::Expression(wrapped)))
        };

        let inner_stmt = self.declaration()?;

        if initializer.is_none() {
            return Ok(self
                .arena
                .statement(Stmt::Loop(condition, inner_stmt, increment)));
        }
        let loop_stmt = self
            .arena
            .statement(Stmt::Loop(condition, inner_stmt, increment));
        Ok(self
            .arena
            .statement(Stmt::Block(vec![initializer.unwrap(), loop_stmt])))
    }

    fn block(&mut self) -> ParseResult<StmtId> {
        let mut statements: Vec<StmtId> = vec![];

        while !self.cmp_tok(&sym!(RightBrace)) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.expected(sym!(RightBrace))?;

        Ok(self.arena.statement(Stmt::Block(statements)))
    }

    fn break_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Break(tok.clone())))
    }

    fn continue_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Continue(tok.clone())))
    }

    fn expression_statement(&mut self) -> ParseResult<StmtId> {
        let expr = self.expression()?;
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Expression(expr)))
    }

    fn var_declaration(&mut self) -> ParseResult<StmtId> {
        let token = self.expected(Identifier { dollar: true })?.clone();
        let expr = match self.match_next(sym!(Equal)) {
            true => self.expression()?,
            false => self.arena.expression(Expr::Literal(RV::Null)),
        };
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Declaration(token, expr)))
    }

    fn fun_declaration(&mut self) -> ParseResult<ExprId> {
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
        let bidx = self.block()?;

        let block = self.arena.get_statement(bidx);

        let body: Vec<StmtId> = match block {
            Stmt::Block(stmts) => stmts.clone(),
            _ => vec![],
        };

        Ok(self
            .arena
            .expression(Expr::Function(token, parameters, Rc::new(body))))
    }

    fn expression(&mut self) -> ParseResult<ExprId> {
        match_next!(self, kw!(Fun), fun_declaration);
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<ExprId> {
        let expr = self.or()?;

        if self.match_next(sym!(Equal)) {
            let value = self.assignment()?;
            match self.arena.get_expression(expr) {
                Expr::Variable(tok) => {
                    return Ok(self.arena.expression(Expr::Assignment {
                        var_tok: tok.clone(),
                        expr: value,
                    }));
                }
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget {
                        left: self.peek_bw(3).clone(),
                        // message: format!("Invalid assignment target `{}`", equals.span.lexeme),
                    });
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<ExprId> {
        let expr = self.and()?;
        if self.match_next(kw!(Keyword::Or)) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(self.arena.expression(Expr::Logical {
                left: expr,
                token: op.clone(),
                right,
            }));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<ExprId> {
        let expr = self.equality()?;
        if self.match_next(kw!(Keyword::And)) {
            let op = self.peek_bw(1);
            let right = self.equality()?;
            return Ok(self.arena.expression(Expr::Logical {
                left: expr,
                token: op.clone(),
                right,
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(BangEqual), sym!(EqualEqual)], comparison);
    }

    fn comparison(&mut self) -> ParseResult<ExprId> {
        binary!(
            self,
            [
                sym!(Greater),
                sym!(GreaterEqual),
                sym!(Less),
                sym!(LessEqual)
            ],
            term
        );
    }

    fn term(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self) -> ParseResult<ExprId> {
        binary!(self, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self) -> ParseResult<ExprId> {
        if self.match_next_multi(&vec![sym!(Minus), sym!(Bang)]) {
            let token = (*self.peek_bw(1)).clone();
            let unary = self.unary()?;
            return Ok(self.arena.expression(Expr::Unary { expr: unary, token }));
        }
        self.select()
    }

    fn select(&mut self) -> ParseResult<ExprId> {
        if !self.cmp_tok(&skw!(Select)) {
            return self.call();
        }
        let core = self.select_core()?;
        let mut compounds: Vec<(SqlCompoundOperator, SelectCore)> = vec![];
        while self.match_next_multi(&vec![skw!(Union), skw!(Intersect), skw!(Except)]) {
            let op = self.peek_bw(1);
            let compound_op = if &op.tok_type == &skw!(Union) && self.match_next(skw!(All)) {
                SqlCompoundOperator::UnionAll
            } else {
                match op.tok_type {
                    SqlKeyword(Union) => SqlCompoundOperator::Union,
                    SqlKeyword(Intersect) => SqlCompoundOperator::Intersect,
                    SqlKeyword(Except) => SqlCompoundOperator::Except,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            // message: format!("Unexpected token `{}`", op.span.lexeme),
                            token: op.clone(),
                        });
                    }
                }
            };
            let secondary_core = self.select_core()?;
            compounds.push((compound_op, secondary_core))
        }
        Ok(self.arena.expression(Expr::Select(SqlSelect {
            core,
            compound: compounds,
            order_by: None, // TODO(vck)
            limit: None,    // TODO(vck)
            offset: None,   // TODO(vck)
        })))
    }

    fn select_core(&mut self) -> ParseResult<SelectCore> {
        self.expected(skw!(Select))?;
        let distinct = if self.match_next(skw!(Distinct)) {
            SqlDistinct::Distinct
        } else {
            SqlDistinct::All
        };
        /* else if self.match_next(skw!(All)) {
            SqlDistinct::All
        }*/

        Ok(SelectCore {
            distinct,
            projection: self.sql_projection(),
            from: self.sql_from()?,
            r#where: self.sql_where()?,
            group_by: None, // TODO(vck)
            having: None,   // TODO(vck)
        })
    }

    fn sql_projection(&mut self) -> Vec<SqlProjection> {
        let mut projections: Vec<SqlProjection> = vec![];
        loop {
            if self.match_next(sym!(Star)) {
                projections.push(SqlProjection::All);
            } else {
                let expr = self.expression().unwrap();
                let alias: Option<Token> =
                    optional_with_expected!(self, skw!(As), Identifier { dollar: false });
                projections.push(SqlProjection::Complex {
                    expr: SqlExpr::Default(expr),
                    alias,
                });
            }
            if !self.match_next(sym!(Comma)) {
                break;
            }
        }
        // TODO(vck): Add support for table selectors
        projections
    }

    fn sql_from(&mut self) -> ParseResult<Option<SqlFrom>> {
        if self.match_next(skw!(From)) {
            let token = self.expected(Identifier { dollar: false });
            return Ok(Some(SqlFrom::TableSubquery(vec![
                SqlTableSubquery::Simple {
                    namespace: None,
                    table: token.unwrap().clone(),
                    alias: None,
                },
            ])));
        }
        // TODO(vck): Joins
        Ok(None)
    }

    fn sql_where(&mut self) -> ParseResult<Option<SqlExpr>> {
        if self.match_next(skw!(Where)) {
            let expr = self.expression()?;
            return Ok(Some(SqlExpr::Default(expr)));
        }
        Ok(None)
    }

    fn finish_call(&mut self, callee: ExprId) -> ParseResult<ExprId> {
        let mut arguments: Vec<ExprId> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            arguments.push(self.expression()?);
            while self.match_next(sym!(Comma)) {
                arguments.push(self.expression()?);
            }
        }
        let paren = self.expected(sym!(RightParen))?.clone();

        Ok(self.arena.expression(Expr::Call {
            callee,
            paren,
            args: arguments,
        }))
    }

    fn call(&mut self) -> ParseResult<ExprId> {
        let mut expr = self.primary()?;

        loop {
            if self.match_next(sym!(LeftParen)) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<ExprId> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            True => Ok(self.arena.expression(Expr::Literal(RV::Bool(true)))),
            False => Ok(self.arena.expression(Expr::Literal(RV::Bool(false)))),
            TokenType::Null => Ok(self.arena.expression(Expr::Literal(RV::Null))),
            Str | Num => Ok(self
                .arena
                .expression(Expr::Literal(tok.literal.clone().unwrap()))),
            Identifier { dollar: _ } => Ok(self.arena.expression(Expr::Variable(tok.clone()))),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(sym!(RightParen))?;
                Ok(self.arena.expression(Expr::Grouping(expr)))
            }
            _ => Err(ParseError::UnexpectedToken {
                // message: format!("Unexpected token `{}`", tok.span.lexeme),
                token: tok.clone(),
            }),
        }
    }

    fn expected(&mut self, expected_tok_type: TokenType) -> ParseResult<&Token> {
        if self.cmp_tok(&expected_tok_type) {
            return Ok(self.advance());
        };
        let prev_token = self.peek_bw(1);
        Err(ParseError::MissingToken {
            token: prev_token.clone(),
            expected: expected_tok_type,
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

/*
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
                Expr::Literal(RV::Num(1.0))
            )
        ]);
    }

    #[test]
    fn test_parse_unary_expression() {
        compare_parsed_to_expected("-1;", vec![
            Stmt::Expression(
                Expr::Unary(
                    Token {tok_type: sym!(Minus), lexeme: lexm!("-"), literal: None, line: 0},
                    Expr::Literal(RV::Num(1.0)
                )
            )
        )]);
    }
}*/
