use self::program::Program;

use super::ast::expr::{Expr, Operation};
use super::ast::stmt::Stmt;
use crate::ast::expr::RangeKind;
use crate::ast::{Literal, Span, Spanned};
use crate::tokenizer::token::{
    Keyword::*, SqlKeyword, SqlKeyword::*, Symbol::*, Token, TokenType, TokenType::*,
};
use crate::{kw, skw, sym};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod program;
pub mod resolver;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    UnexpectedToken { token: Token },
    MissingToken { token: Token, expected: TokenType },
    InvalidAssignmentTarget { left: Token },
    NoTokens,
}

type ParseResult<T> = Result<T, ParseError>;

macro_rules! binary {
    ($self: ident, [$($operator:expr),*], $builder: ident) => {
        let mut current_expr = $self.$builder()?;
        while $self.match_next_one_of(&[$($operator,)*]) {
            let token = (*$self.peek_bw(1)).clone();
            let left: Box<Expr> = current_expr;
            let right: Box<Expr> = $self.$builder()?;

            let expr = Expr::Binary {
                left: left.clone(),
                operation: $self.tok_type_to_op(token.clone().tok_type),
                right: right.clone(),
                span: $self.get_merged_span(
                    &left.get_span(),
                    &right.get_span(),
                ),
                id: $self.get_expr_id(),
            };

            current_expr = Box::new(expr);
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
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    expr_id: usize,
    in_select_depth: usize,
    in_array_depth: usize,
    in_object_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn create(tokens: &Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            in_select_depth: 0,
            in_array_depth: 0,
            in_object_depth: 0,
            expr_id: 0,
        }
    }

    pub fn parse(tokens: &Vec<Token>) -> ParseResult<Program> {
        if tokens.is_empty() || tokens.first().unwrap().tok_type == Eof {
            return Err(ParseError::NoTokens);
        }
        let mut parser = Parser {
            tokens,
            current: 0,
            in_select_depth: 0,
            in_array_depth: 0,
            in_object_depth: 0,
            expr_id: 0,
        };
        let program = parser.program()?;
        Ok(Program::new(program))
    }

    fn get_expr_id(&mut self) -> usize {
        let id = self.expr_id;
        self.expr_id += 1;
        id
    }

    pub fn program(&mut self) -> ParseResult<Box<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_at_end() {
            statements.push(*self.declaration()?);
        }
        self.expected(&Eof)?;
        Ok(Box::new(Stmt::Program {
            body: statements.clone(),
            span: self.get_merged_span(&(statements[0]), &(statements[statements.len() - 1])),
        }))
    }

    fn declaration(&mut self) -> ParseResult<Box<Stmt>> {
        match_next!(self, &kw!(Var), var_declaration);
        self.statement()
    }

    fn statement(&mut self) -> ParseResult<Box<Stmt>> {
        match_next!(self, &kw!(If), if_statement);
        match_next!(self, &kw!(While), while_statement);
        match_next!(self, &kw!(For), for_statement);
        match_next!(self, &kw!(Loop), loop_statement);
        match_next!(self, &kw!(Break), break_statement);
        match_next!(self, &kw!(Continue), continue_statement);
        match_next!(self, &kw!(Return), return_statement);
        if self.peek_next_all_of(&[sym!(LeftBrace), Identifier { dollar: false }, sym!(Colon)])
            || self.peek_next_all_of(&[sym!(LeftBrace), Str, sym!(Colon)])
            || self.peek_next_all_of(&[sym!(LeftBrace), Num, sym!(Colon)])
            || self.peek_next_all_of(&[sym!(LeftBrace), sym!(RightBrace)])
        {
            return self.expression_statement();
        }
        match_next!(self, &sym!(LeftBrace), block);
        self.expression_statement()
    }

    fn if_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let if_tok = self.peek_bw(1);
        self.expected(&sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(&sym!(RightParen))?;
        let if_branch = self.statement()?;

        if self.match_next(&kw!(Else)) {
            let else_branch = self.statement()?;
            return Ok(Box::new(Stmt::If {
                condition,
                body: if_branch.clone(),
                r#else_body: Some(else_branch.clone()),
                span: self.get_merged_span(&if_tok.span, &else_branch.get_span()),
            }));
        }
        Ok(Box::new(Stmt::If {
            condition,
            body: if_branch.clone(),
            r#else_body: None,
            span: self.get_merged_span(&if_tok.span, &if_branch.get_span()),
        }))
    }

    fn loop_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let loop_tok = self.peek_bw(1);
        self.expected(&sym!(LeftBrace))?;
        let inner_stmt = self.block()?;
        self.match_next(&sym!(Semicolon));
        Ok(Box::new(Stmt::Loop {
            condition: None,
            body: inner_stmt.clone(),
            post: None,
            span: self.get_merged_span(&loop_tok.span, &inner_stmt.get_span()),
        }))
    }

    fn while_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let while_tok = self.peek_bw(1);
        self.expected(&sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(&sym!(RightParen))?;
        self.expected(&sym!(LeftBrace))?;
        let inner_stmt = self.block()?;
        self.match_next(&sym!(Semicolon));

        Ok(Box::new(Stmt::Loop {
            condition: Some(condition),
            body: inner_stmt.clone(),
            post: None,
            span: self.get_merged_span(&while_tok.span, &inner_stmt.get_span()),
        }))
    }

    fn return_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let ret_tok = self.peek_bw(1);
        let mut expr: Option<Box<Expr>> = None;
        if !self.cmp_tok(&sym!(Semicolon)) {
            expr = Some(self.expression()?);
        }
        self.expected(&sym!(Semicolon))?;

        if expr.is_none() {
            return Ok(Box::new(Stmt::Return {
                span: ret_tok.span,
                expr: None,
            }));
        }

        Ok(Box::new(Stmt::Return {
            span: self.get_merged_span(&ret_tok.span, &expr.as_ref().unwrap().get_span()),
            expr,
        }))
    }

    fn for_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let for_tok = self.peek_bw(1);
        self.expected(&sym!(LeftParen))?;

        let initializer = if self.match_next(&sym!(Semicolon)) {
            None
        } else {
            Some(self.declaration()?)
        };

        let condition = if self.match_next(&sym!(Semicolon)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(&sym!(Semicolon))?;
            Some(wrapped)
        };

        let increment = if self.match_next(&sym!(RightParen)) {
            None
        } else {
            let wrapped = self.expression()?;
            self.expected(&sym!(RightParen))?;
            Some(Box::new(Stmt::Expression {
                expr: wrapped.clone(),
                span: wrapped.get_span(),
            }))
        };

        self.expected(&sym!(LeftBrace))?;
        let inner_stmt = self.block()?;
        self.match_next(&sym!(Semicolon));

        if initializer.is_none() {
            return Ok(Box::new(Stmt::Loop {
                condition,
                body: inner_stmt.clone(),
                post: increment,
                span: self.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
            }));
        }
        let loop_stmt = Box::new(Stmt::Loop {
            condition,
            body: inner_stmt.clone(),
            post: increment,
            span: self.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
        });
        Ok(Box::new(Stmt::Block {
            body: vec![*initializer.unwrap(), *loop_stmt],
            span: self.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
        }))
    }

    fn block(&mut self) -> ParseResult<Box<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];

        let opening_brace = self.peek_bw(1);

        while !self.cmp_tok(&sym!(RightBrace)) && !self.is_at_end() {
            statements.push(*self.declaration()?);
        }

        let closing_brace = self.peek_bw(1);

        self.expected(&sym!(RightBrace))?;

        Ok(Box::new(Stmt::Block {
            body: statements.clone(),
            span: self.get_merged_span(&opening_brace.span, &closing_brace.span),
        }))
    }

    fn break_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let tok = self.peek_bw(1);
        self.expected(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Break { span: tok.span }))
    }

    fn continue_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let tok = self.peek_bw(1);
        self.expected(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Continue { span: tok.span }))
    }

    fn expression_statement(&mut self) -> ParseResult<Box<Stmt>> {
        let expr = self.expression()?;
        let span = expr.get_span();
        self.expected(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Expression { expr, span }))
    }

    fn var_declaration(&mut self) -> ParseResult<Box<Stmt>> {
        let var_tok = self.peek_bw(1);
        let ident = self.expected(&Identifier { dollar: true })?.clone();
        let expr = match self.match_next(&sym!(Equal)) {
            true => self.expression()?,
            false => {
                let node = Expr::Literal {
                    value: Literal::Undefined,
                    raw: "undefined".to_string(),
                    span: var_tok.span,
                    id: self.get_expr_id(),
                };
                Box::new(node)
            }
        };
        self.expected(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Declaration {
            dst: ident.extract_identifier().unwrap(),
            expr: expr.clone(),
            span: self.get_merged_span(&var_tok.span, &expr.get_span()),
        }))
    }

    fn fun_declaration(&mut self) -> ParseResult<Box<Expr>> {
        let mut tokens = vec![];
        let fun_tok = self.peek_bw(1);
        tokens.push(fun_tok.clone());
        let token = if self.cmp_tok(&Identifier { dollar: true }) {
            Some(self.expected(&Identifier { dollar: true })?.clone())
        } else {
            None
        };

        if let Some(t) = token.clone() {
            tokens.push(t.clone());
        }

        let left_paren = self.expected(&sym!(LeftParen))?;

        tokens.push(left_paren.clone());

        let mut parameters: Vec<Token> = vec![];
        if !self.cmp_tok(&sym!(RightParen)) {
            let p = self.expected(&Identifier { dollar: true })?;
            tokens.push(p.clone());
            parameters.push(p.clone());
            while self.match_next(&sym!(Comma)) {
                let q = self.expected(&Identifier { dollar: true })?;
                tokens.push(q.clone());
                parameters.push(q.clone());
            }
        }
        let right_par = self.expected(&sym!(RightParen))?;
        tokens.push(right_par.clone());

        self.expected(&sym!(LeftBrace))?;

        let stmt = self.block()?;

        let inner_stmt = &(stmt);

        let body: Vec<Stmt> = match inner_stmt.as_ref() {
            Stmt::Block { body: stmts, .. } => stmts.clone(),
            _ => vec![*stmt.clone()],
        };

        let node = Expr::Function {
            name: token.map(|t| t.extract_identifier().unwrap()),
            parameters: parameters
                .iter()
                .map(|t| t.extract_identifier().unwrap())
                .collect(),
            body: Arc::new(body),
            span: self.get_merged_span(&fun_tok.span, &stmt.get_span()),
            id: self.get_expr_id(),
        };

        Ok(Box::new(node))
    }

    fn expression(&mut self) -> ParseResult<Box<Expr>> {
        match_next!(self, &kw!(Fun), fun_declaration);
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.or()?;

        if self.match_next(&sym!(Equal)) {
            let value = self.assignment()?;
            match expr.as_ref() {
                Expr::Variable { name, span, .. } => {
                    return Ok(Box::new(Expr::Assignment {
                        id: self.get_expr_id(),
                        dst: name.clone(),
                        expr: value.clone(),
                        span: self.get_merged_span(span, &value.get_span()),
                    }));
                }
                Expr::Get {
                    object, name, span, ..
                } => {
                    return Ok(Box::new(Expr::Set {
                        object: object.clone(),
                        name: name.clone(),
                        value: value.clone(),
                        span: self.get_merged_span(span, &value.get_span()),
                        id: self.get_expr_id(),
                    }));
                }
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget {
                        left: self.peek_bw(3).clone(),
                    });
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.and()?;
        let operator = if self.in_select_depth > 0 {
            skw!(Or)
        } else {
            sym!(LogicalOr)
        };
        if self.match_next(&operator) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(Box::new(Expr::Logical {
                left: expr.clone(),
                operation: self.tok_type_to_op(op.tok_type.clone()),
                right: right.clone(),
                span: self.get_merged_span(&expr.get_span(), &right.get_span()),
                id: self.get_expr_id(),
            }));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.equality()?;
        let operator = if self.in_select_depth > 0 {
            skw!(And)
        } else {
            sym!(LogicalAnd)
        };
        if self.match_next(&operator) {
            let op = self.peek_bw(1);
            let right = self.equality()?;
            return Ok(Box::new(Expr::Logical {
                left: expr.clone(),
                operation: self.tok_type_to_op(op.tok_type.clone()),
                right: right.clone(),
                span: self.get_merged_span(&expr.get_span(), &right.get_span()),
                id: self.get_expr_id(),
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Box<Expr>> {
        if self.in_select_depth > 0 {
            binary!(self, [sym!(BangEqual), sym!(Equal)], cmp_basic);
        } else {
            binary!(self, [sym!(BangEqual), sym!(EqualEqual)], cmp_basic);
        }
    }

    fn cmp_basic(&mut self) -> ParseResult<Box<Expr>> {
        binary!(
            self,
            [
                sym!(Greater),
                sym!(GreaterEqual),
                sym!(Less),
                sym!(LessEqual)
            ],
            cmp_advanced
        );
    }

    fn cmp_advanced(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.term()?;
        let expr_conj_fst = if self.match_next_one_of(&[
            skw!(Is),
            skw!(Not),
            skw!(In),
            skw!(Between),
            skw!(Like),
        ]) {
            Some((*self.peek_bw(1)).clone().tok_type)
        } else {
            None
        };
        let expr_conj_sec = if expr_conj_fst.is_some()
            && self.match_next_one_of(&[skw!(Is), skw!(Not), skw!(In), skw!(Between), skw!(Like)])
        {
            Some((*self.peek_bw(1)).clone().tok_type)
        } else {
            None
        };

        let left = expr.clone();

        if expr_conj_fst.is_none() && expr_conj_sec.is_none() {
            return Ok(left);
        }

        let operation = match (&expr_conj_fst, &expr_conj_sec) {
            (Some(SqlKeyword(Is)), None) => Some(Operation::Is),
            (Some(SqlKeyword(Is)), Some(SqlKeyword(Not))) => Some(Operation::IsNot),
            (Some(SqlKeyword(In)), None) => Some(Operation::In),
            (Some(SqlKeyword(Not)), Some(SqlKeyword(In))) => Some(Operation::NotIn),
            (Some(SqlKeyword(Like)), None) => Some(Operation::Like),
            (Some(SqlKeyword(Not)), Some(SqlKeyword(Like))) => Some(Operation::NotLike),
            _ => None,
        };

        if let Some(operation) = operation {
            let right = self.term()?;
            return Ok(Box::new(Expr::Binary {
                left: left.clone(),
                operation,
                right: right.clone(),
                span: left.get_span().merge(&right.get_span()),
                id: self.get_expr_id(),
            }));
        }

        match (&expr_conj_fst, &expr_conj_sec) {
            (Some(SqlKeyword(Between)), None) => self.cmp_between(left, false),
            (Some(SqlKeyword(Not)), Some(SqlKeyword(Between))) => self.cmp_between(left, true),
            _ => Ok(expr),
        }
    }

    fn cmp_between(&mut self, subject: Box<Expr>, falsy: bool) -> ParseResult<Box<Expr>> {
        let lower = self.term()?;

        self.expected(&skw!(And))?;
        let upper = self.term()?;

        Ok(Box::new(Expr::Between {
            subject: subject.clone(),
            lower,
            upper: upper.clone(),
            kind: if falsy {
                RangeKind::NotBetween
            } else {
                RangeKind::Between
            },
            span: subject.get_span().merge(&upper.get_span()),
            id: self.get_expr_id(),
        }))
    }

    fn term(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self) -> ParseResult<Box<Expr>> {
        binary!(self, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self) -> ParseResult<Box<Expr>> {
        if self.match_next_one_of(&[sym!(Minus), sym!(Bang)]) {
            let token = (*self.peek_bw(1)).clone();
            let unary = self.unary()?;
            return Ok(Box::new(Expr::Unary {
                operation: self.tok_type_to_op(token.tok_type),
                expr: unary.clone(),
                span: self.get_merged_span(&token.span, &(unary).get_span()),
                id: self.get_expr_id(),
            }));
        }
        self.sql_insert()
    }

    fn expect_get_path(&mut self, initial: Box<Expr>, tok: TokenType) -> ParseResult<Box<Expr>> {
        let mut expr = initial;

        loop {
            if self.match_next(&sym!(LeftParen)) {
                expr = self.finish_call(expr)?;
            } else if self.match_next(&tok.clone()) {
                let identifier = self.expected(&Identifier { dollar: false })?.clone();
                expr = Box::new(Expr::Get {
                    object: expr.clone(),
                    name: identifier.extract_identifier().unwrap(),
                    span: self.get_merged_span(&(expr).get_span(), &identifier.span),
                    id: self.get_expr_id(),
                })
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn call(&mut self) -> ParseResult<Box<Expr>> {
        let expr = self.primary()?;

        if let Expr::Variable { name, span, id } = expr.as_ref() {
            if !name.dollar {
                let next_tok = &self.peek_bw(0).tok_type;

                if (next_tok == &sym!(Dot) || next_tok != &sym!(LeftParen))
                    && self.in_select_depth > 0
                {
                    let head = name.clone();
                    let mut tail: Vec<super::ast::Identifier> = vec![];
                    while self.match_next(&sym!(Dot)) {
                        let identifier = self.expected(&Identifier { dollar: false })?.clone();
                        tail.push(identifier.extract_identifier().unwrap());
                    }
                    return Ok(Box::new(Expr::FieldPath {
                        head,
                        tail,
                        span: self.get_merged_span(span, &self.peek_bw(0).span),
                        id: *id,
                    }));
                }

                return self.expect_get_path(expr, sym!(DoubleColon));
            }
        }

        self.expect_get_path(expr, sym!(Dot))
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> ParseResult<Box<Expr>> {
        let mut arguments: Vec<Expr> = vec![];

        if !self.cmp_tok(&sym!(RightParen)) {
            arguments.push(*self.expression()?);
            while self.match_next(&sym!(Comma)) {
                arguments.push(*self.expression()?);
            }
        }

        let paren = self.expected(&sym!(RightParen))?.clone();

        Ok(Box::new(Expr::Call {
            callee: callee.clone(),
            span: self.get_merged_span(&(callee).get_span(), &paren.span),
            args: arguments,
            id: self.get_expr_id(),
        }))
    }

    fn object_literal(&mut self, tok: &Token) -> ParseResult<Box<Expr>> {
        let mut obj_literal: FxHashMap<String, Box<Expr>> = FxHashMap::default();
        self.in_object_depth += 1;
        while !self.cmp_tok(&sym!(RightBrace)) {
            let key = if self.match_next_one_of(&[Identifier { dollar: false }, Str, Num]) {
                let key_tok = self.peek_bw(1).clone();
                match key_tok.tok_type {
                    Identifier { dollar: false } | Str => key_tok
                        .literal
                        .as_ref()
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                    Num => match key_tok.literal {
                        Some(Literal::Num(n)) => n.to_string(),
                        _ => {
                            return Err(ParseError::UnexpectedToken { token: key_tok });
                        }
                    },
                    _ => {
                        return Err(ParseError::UnexpectedToken { token: key_tok });
                    }
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: self.peek_bw(1).clone(),
                });
            };

            self.expected(&sym!(Colon))?;
            let value = self.expression()?;
            obj_literal.insert(key, value);
            if !self.match_next(&sym!(Comma)) {
                break;
            }
        }
        self.expected(&sym!(RightBrace))?;
        self.in_object_depth -= 1;
        Ok(Box::new(Expr::Literal {
            value: Literal::Object(obj_literal),
            raw: "".to_string(),
            span: self.get_merged_span(&tok.span, &self.peek_bw(0).span),
            id: self.get_expr_id(),
        }))
    }

    fn array_literal(&mut self, tok: &Token) -> ParseResult<Box<Expr>> {
        let mut array_literal: Vec<Expr> = vec![];
        self.in_array_depth += 1;
        while !self.cmp_tok(&sym!(RightBracket)) {
            let value = self.expression()?;
            array_literal.push(*value.clone());
            if !self.match_next(&sym!(Comma)) {
                break;
            }
        }
        self.expected(&sym!(RightBracket))?;
        self.in_array_depth -= 1;
        Ok(Box::new(Expr::Literal {
            value: Literal::Array(array_literal),
            raw: "".to_string(),
            span: self.get_merged_span(&tok.span, &self.peek_bw(0).span),
            id: self.get_expr_id(),
        }))
    }

    fn primary(&mut self) -> ParseResult<Box<Expr>> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            Symbol(LeftBrace) => self.object_literal(tok),
            Symbol(LeftBracket) => self.array_literal(tok),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(&sym!(RightParen))?;
                Ok(Box::new(Expr::Grouping {
                    span: (expr).get_span(),
                    expr,
                    id: self.get_expr_id(),
                }))
            }
            True => Ok(Box::new(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: tok.span,
                id: self.get_expr_id(),
            })),
            False => Ok(Box::new(Expr::Literal {
                value: Literal::Bool(false),
                raw: "false".to_string(),
                span: tok.span,
                id: self.get_expr_id(),
            })),
            TokenType::Undefined => Ok(Box::new(Expr::Literal {
                value: Literal::Undefined,
                raw: "undefined".to_string(),
                span: tok.span,
                id: self.get_expr_id(),
            })),
            Str | Num => Ok(Box::new(Expr::Literal {
                value: tok.literal.clone().unwrap(),
                raw: tok.lexeme.clone().unwrap(),
                span: tok.span,
                id: self.get_expr_id(),
            })),
            Identifier { .. } => Ok(Box::new(Expr::Variable {
                name: tok.extract_identifier().unwrap(),
                span: tok.span,
                id: self.get_expr_id(),
            })),
            _ => Err(ParseError::UnexpectedToken { token: tok.clone() }),
        }
    }

    fn expected(&mut self, expected_tok_type: &TokenType) -> ParseResult<&Token> {
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

    fn peek_fw(&self, offset: usize) -> &'a Token {
        &self.tokens[self.current + offset]
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
                    if self.in_select_depth > 0 {
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

use crate::ast::sql::{
    SqlCollectionIdentifier, SqlCompoundOperator, SqlDelete, SqlDistinct, SqlExpressionSource,
    SqlFrom, SqlInsert, SqlJoinType, SqlLimitClause, SqlOrderByClause, SqlOrdering, SqlProjection,
    SqlSelect, SqlSelectCompound, SqlSelectCore, SqlSource, SqlUpdate, SqlValues,
};

macro_rules! optional_with_expected {
    ($self: ident, $optional: expr, $expected: expr) => {
        if $self.match_next(&$optional) {
            let token = $self.expected(&$expected);
            Some(token.unwrap().clone())
        } else if $self.match_next(&$expected) {
            let token = $self.peek_bw(1);
            Some(token.clone())
        } else {
            None
        }
    };
}

impl Parser<'_> {
    fn sql_insert(&mut self) -> ParseResult<Box<Expr>> {
        if !self.match_next(&skw!(Insert)) {
            return self.sql_update();
        }

        self.expected(&skw!(Into))?;

        if let Some(collection) = self.sql_collection_identifier()? {
            let values = if self.cmp_tok(&skw!(Select)) {
                let select_inner = self.sql_select_inner();

                if select_inner.is_err() {
                    return Err(select_inner.err().unwrap());
                }

                SqlValues::Select(select_inner.unwrap())
            } else if self.match_next(&skw!(Values)) {
                self.expected(&sym!(LeftParen))?;
                let mut values: Vec<Expr> = vec![];
                loop {
                    values.push(*self.expression()?);
                    if !self.match_next(&sym!(Comma)) {
                        break;
                    }
                }
                self.expected(&sym!(RightParen))?;
                SqlValues::Values { values }
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: self.peek_bw(0).clone(),
                });
            };
            Ok(Box::new(Expr::Insert {
                command: SqlInsert { collection, values },
                span: Span::default(),
                id: self.get_expr_id(),
            }))
        } else {
            Err(ParseError::UnexpectedToken {
                token: self.peek_bw(0).clone(),
            })
        }
    }

    fn sql_update(&mut self) -> ParseResult<Box<Expr>> {
        if !self.match_next(&skw!(Update)) {
            return self.sql_delete();
        }

        let collection = self.sql_collection_identifier()?;

        self.expected(&skw!(Set))?;

        let mut assignments: Vec<Expr> = vec![];

        loop {
            self.expected(&Identifier { dollar: false })?;
            self.expected(&sym!(Equal))?;
            assignments.push(*self.expression()?);
            if !self.match_next(&sym!(Comma)) {
                break;
            }
        }

        let r#where = if self.match_next(&skw!(Where)) {
            Some(self.expression()?)
        } else {
            None
        };

        Ok(Box::new(Expr::Update {
            command: SqlUpdate {
                collection: collection.unwrap(),
                assignments,
                r#where,
            },
            span: Span::default(),
            id: self.get_expr_id(),
        }))
    }

    fn sql_delete(&mut self) -> ParseResult<Box<Expr>> {
        if !self.match_next(&skw!(Delete)) {
            return self.sql_select();
        }

        self.expected(&skw!(From))?;

        if let Some(collection) = self.sql_collection_identifier()? {
            let r#where = if self.match_next(&skw!(Where)) {
                Some(self.expression()?)
            } else {
                None
            };

            Ok(Box::new(Expr::Delete {
                command: SqlDelete {
                    collection,
                    r#where,
                },
                span: Span::default(),
                id: self.get_expr_id(),
            }))
        } else {
            Err(ParseError::UnexpectedToken {
                token: self.peek_bw(0).clone(),
            })
        }
    }

    fn sql_collection_identifier(&mut self) -> ParseResult<Option<SqlCollectionIdentifier>> {
        if self.cmp_tok(&Identifier { dollar: false }) {
            if self.match_next_all_of(&[
                Identifier { dollar: false },
                sym!(Dot),
                Identifier { dollar: false },
            ]) {
                return Ok(Some(SqlCollectionIdentifier {
                    namespace: Some(self.peek_bw(3).extract_identifier().unwrap()),
                    name: self.peek_bw(1).extract_identifier().unwrap(),
                    alias: optional_with_expected!(self, skw!(As), Identifier { dollar: false })
                        .map(|t| t.extract_identifier().unwrap()),
                }));
            }
            return Ok(Some(SqlCollectionIdentifier {
                namespace: None,
                name: self
                    .expected(&Identifier { dollar: false })?
                    .extract_identifier()
                    .unwrap(),
                alias: optional_with_expected!(self, skw!(As), Identifier { dollar: false })
                    .map(|t| t.extract_identifier().unwrap()),
            }));
        }
        Ok(None)
    }

    fn sql_select(&mut self) -> ParseResult<Box<Expr>> {
        if !self.cmp_tok(&skw!(Select)) {
            return self.call();
        }

        let query: ParseResult<SqlSelect> = {
            let select_inner = self.sql_select_inner();

            if select_inner.is_err() {
                return Err(select_inner.err().unwrap());
            }

            Ok(select_inner.unwrap())
        };

        Ok(Box::new(Expr::Select {
            span: Span::default(),
            query: query.unwrap(),
            id: self.get_expr_id(),
        }))
    }

    fn sql_select_inner(&mut self) -> ParseResult<SqlSelect> {
        self.in_select_depth += 1;
        let core: SqlSelectCore = self.sql_select_core()?;
        let order_by = if self.match_next(&skw!(Order)) {
            self.expected(&skw!(By))?;
            let mut ordering: Vec<SqlOrderByClause> = vec![];

            loop {
                let order_expr = self.expression()?;
                let order = if self.match_next(&skw!(Desc)) {
                    Some(SqlOrdering::Desc)
                } else {
                    self.match_next(&skw!(Asc));
                    Some(SqlOrdering::Asc)
                };
                ordering.push(SqlOrderByClause {
                    expr: order_expr,
                    ordering: order.unwrap(),
                });
                if !self.match_next(&sym!(Comma)) {
                    break;
                }
            }

            Some(ordering)
        } else {
            None
        };

        let limit = if self.match_next(&skw!(Limit)) {
            let first_expr = self.expression()?;
            let (second_expr, reverse) = if self.match_next(&skw!(Offset)) {
                (Some(self.expression()?), false)
            } else if self.match_next(&sym!(Comma)) {
                (Some(self.expression()?), true)
            } else {
                (None, false)
            };

            match (&second_expr, reverse) {
                (Some(_), true) => Some(SqlLimitClause {
                    count: second_expr.unwrap(),
                    offset: Some(first_expr),
                }),
                _ => Some(SqlLimitClause {
                    count: first_expr,
                    offset: second_expr,
                })
            }
        } else {
            None
        };

        self.in_select_depth -= 1;

        Ok(SqlSelect {
            core,
            order_by,
            limit,
        })
    }

    fn sql_select_core(&mut self) -> ParseResult<SqlSelectCore> {
        self.expected(&skw!(Select))?;
        let distinct = if self.match_next(&skw!(Distinct)) {
            SqlDistinct::Distinct
        } else if self.match_next(&skw!(All)) {
            SqlDistinct::All
        } else {
            SqlDistinct::ImplicitAll
        };

        let projection = self.sql_select_projection()?;
        let from = self.sql_select_from()?;
        let r#where = self.sql_select_where()?;
        let group_by = self.sql_select_group_by()?;
        let having = if group_by.is_some() && self.match_next(&skw!(Having)) {
            Some(self.expression()?)
        } else {
            None
        };

        let compound: Option<Box<SqlSelectCompound>> =
            if self.match_next_one_of(&[skw!(Union), skw!(Intersect), skw!(Except)]) {
                let op = self.peek_bw(1);
                let compound_op = if op.tok_type == skw!(Union) && self.match_next(&skw!(All)) {
                    SqlCompoundOperator::UnionAll
                } else {
                    match op.tok_type {
                        SqlKeyword(Union) => SqlCompoundOperator::Union,
                        SqlKeyword(Intersect) => SqlCompoundOperator::Intersect,
                        SqlKeyword(Except) => SqlCompoundOperator::Except,
                        _ => {
                            return Err(ParseError::UnexpectedToken { token: op.clone() });
                        }
                    }
                };
                Some(Box::from(SqlSelectCompound {
                    operator: compound_op,
                    core: self.sql_select_core()?,
                }))
            } else {
                None
            };

        Ok(SqlSelectCore {
            distinct,
            projection,
            from,
            r#where,
            group_by,
            having,
            compound,
        })
    }

    fn sql_select_projection(&mut self) -> ParseResult<Vec<SqlProjection>> {
        let mut projections: Vec<SqlProjection> = vec![];
        loop {
            if self.match_next(&sym!(Star)) {
                projections.push(SqlProjection::All { collection: None });
            } else if self.match_next_all_of(&[Identifier { dollar: false }, sym!(Dot), sym!(Star)])
            {
                projections.push(SqlProjection::All {
                    collection: Some(self.peek_bw(3).extract_identifier().unwrap()),
                });
            } else {
                let expr = self.expression()?;
                let alias: Option<Token> =
                    optional_with_expected!(self, skw!(As), Identifier { dollar: false });
                projections.push(SqlProjection::Expr {
                    expr,
                    alias: alias.map(|t| t.extract_identifier().unwrap()),
                });
            }
            if !self.match_next(&sym!(Comma)) {
                break;
            }
        }
        Ok(projections)
    }

    fn sql_select_from(&mut self) -> ParseResult<Option<SqlFrom>> {
        if self.match_next(&skw!(From)) {
            return Ok(Some(self.sql_select_from_join()?));
        }
        Ok(None)
    }

    fn sql_select_from_join(&mut self) -> ParseResult<SqlFrom> {
        let mut from_group: Vec<SqlFrom> = vec![];

        loop {
            let left = self.sql_select_from_source()?;
            from_group.push(left);
            while self.match_next_one_of(&[
                skw!(Left),
                skw!(Right),
                skw!(Inner),
                skw!(Cross),
                skw!(Join),
            ]) {
                // If the next token is a join keyword, then it must be a join from
                let peek = self.peek_bw(1);
                if peek.tok_type != SqlKeyword(Join) {
                    self.expected(&skw!(Join))?;
                }
                let join_type = match peek.tok_type {
                    SqlKeyword(Inner) => SqlJoinType::Inner,
                    SqlKeyword(Left) => SqlJoinType::Left,
                    SqlKeyword(Right) => SqlJoinType::Right,
                    SqlKeyword(Cross) => SqlJoinType::Cross,
                    SqlKeyword(Join) => SqlJoinType::Inner,
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            token: peek.clone(),
                        });
                    }
                };
                let right = self.sql_select_from_source()?;
                let join_constraint: Option<Box<Expr>> = if self.match_next(&skw!(On)) {
                    Some(self.expression()?)
                } else {
                    None
                };

                let left_popped = from_group.pop().unwrap();

                from_group.push(SqlFrom::Join {
                    left: Box::new(left_popped),
                    right: Box::new(right),
                    join_type,
                    constraint: join_constraint,
                });
            }
            if !self.match_next(&sym!(Comma)) {
                break;
            }
        }

        Ok(SqlFrom::Group { values: from_group })
    }

    fn sql_select_where(&mut self) -> ParseResult<Option<Box<Expr>>> {
        if self.match_next(&skw!(Where)) {
            return Ok(Some(self.expression()?));
        }
        Ok(None)
    }

    fn sql_select_group_by(&mut self) -> ParseResult<Option<Vec<Expr>>> {
        if self.match_next(&skw!(Group)) {
            self.expected(&skw!(By))?;
            let mut groups: Vec<Expr> = vec![];

            loop {
                let sql_expr = self.expression()?;
                groups.push(*sql_expr);
                if !self.match_next(&sym!(Comma)) {
                    break;
                }
            }
            Ok(Some(groups))
        } else {
            Ok(None)
        }
    }

    fn sql_select_from_source(&mut self) -> ParseResult<SqlFrom> {
        if self.match_next(&sym!(LeftParen)) {
            if self.cmp_tok(&skw!(Select)) {
                let subquery = Box::new(self.sql_select_inner()?);
                self.expected(&sym!(RightParen))?;
                let alias: Option<Token> =
                    optional_with_expected!(self, skw!(As), Identifier { dollar: false });
                return Ok(SqlFrom::Select {
                    subquery,
                    alias: alias.map(|t| t.extract_identifier().unwrap()),
                });
            }
            // If the next token is a left paren, then it must be either a select statement or a recursive "from" clause
            let parsed = self.sql_select_from_join()?;
            self.expected(&sym!(RightParen))?;
            Ok(parsed)
        } else if let Some(collection) = self.sql_collection_identifier()? {
            return Ok(SqlFrom::Source(SqlSource::Collection(collection)));
        } else {
            let expr = self.expression()?;
            self.expected(&skw!(As))?;
            let identifier = self.expected(&Identifier { dollar: false })?.clone();
            return Ok(SqlFrom::Source(SqlSource::Expr(SqlExpressionSource {
                expr,
                alias: identifier.extract_identifier().unwrap(),
            })));
        }
    }
}
