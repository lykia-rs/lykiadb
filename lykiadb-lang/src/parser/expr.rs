use std::sync::Arc;

use crate::ast::expr::{Expr, Operation, RangeKind, TypeAnnotation};
use crate::ast::stmt::Stmt;
use crate::ast::{IdentifierKind, Literal, Spanned};
use crate::tokenizer::token::{
    Keyword::*, SqlKeyword::*, Symbol::*, Token, TokenType, TokenType::*,
};
use crate::{kw, skw, sym};
use rustc_hash::FxHashMap;

use super::{ParseError, ParseResult, Parser};

macro_rules! binary {
    ($self: ident, $cparser: expr, [$($operator:expr),*], $builder: ident) => {
        let mut current_expr = $self.$builder($cparser)?;
        while $cparser.match_next_one_of(&[$($operator,)*]) {
            let token = (*$cparser.peek_bw(1)).clone();
            let left: Box<Expr> = current_expr;
            let right: Box<Expr> = $self.$builder($cparser)?;

            let expr = Expr::Binary {
                left: left.clone(),
                operation: $cparser.tok_type_to_op(token.clone().tok_type),
                right: right.clone(),
                span: $cparser.get_merged_span(
                    &left.get_span(),
                    &right.get_span(),
                ),
                id: $cparser.get_expr_id(),
            };

            current_expr = Box::new(expr);
        }
        return Ok(current_expr);
    }
}

pub struct ExprParser {}

impl ExprParser {
    pub fn expression(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if cparser.match_next(&kw!(Fun)) {
            return self.fun_declaration(cparser);
        }
        self.assignment(cparser)
    }

    fn fun_declaration(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let fun_tok = cparser.peek_bw(1);

        let token = if cparser.cmp_tok(&Identifier { dollar: true }) {
            Some(cparser.expect(&Identifier { dollar: true })?.clone())
        } else {
            None
        };

        cparser.expect(&sym!(LeftParen))?;

        let mut parameters: Vec<(Token, Option<TypeAnnotation>)> = vec![];

        if !cparser.cmp_tok(&sym!(RightParen)) {
            let p = cparser.expect(&Identifier { dollar: true })?.clone();
            // cparser.expect(&sym!(Colon))?;
            // parameters.push((p, cparser.expect_type_annotation()?));
            parameters.push((p, None));
            while cparser.match_next(&sym!(Comma)) {
                let q = cparser.expect(&Identifier { dollar: true })?.clone();
                // cparser.expect(&sym!(Colon))?;
                parameters.push((q.clone(), None));
            }
        }
        cparser.expect(&sym!(RightParen))?;
        /*
        cparser.expect(&sym!(RightArrow))?;

        let return_type = cparser.expect_type_annotation()?;
        */
        cparser.expect(&sym!(LeftBrace))?;

        let stmt = cparser.consume_block()?;

        let inner_stmt = &(stmt);

        let body: Vec<Stmt> = match inner_stmt.as_ref() {
            Stmt::Block { body: stmts, .. } => stmts.clone(),
            _ => vec![*stmt.clone()],
        };

        let node = Expr::Function {
            name: token.map(|t| t.extract_identifier().unwrap()),
            parameters: parameters
                .iter()
                .map(|(t, annotation)| (t.extract_identifier().unwrap(), annotation.clone()))
                .collect(),
            return_type: None,
            body: Arc::new(body),
            span: cparser.get_merged_span(&fun_tok.span, &stmt.get_span()),
            id: cparser.get_expr_id(),
        };

        Ok(Box::new(node))
    }

    fn assignment(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let expr = self.or(cparser)?;

        if cparser.match_next(&sym!(Equal)) {
            let value = self.assignment(cparser)?;
            match expr.as_ref() {
                Expr::Variable { name, span, .. } => {
                    return Ok(Box::new(Expr::Assignment {
                        id: cparser.get_expr_id(),
                        dst: name.clone(),
                        expr: value.clone(),
                        span: cparser.get_merged_span(span, &value.get_span()),
                    }));
                }
                Expr::Get {
                    object, name, span, ..
                } => {
                    return Ok(Box::new(Expr::Set {
                        object: object.clone(),
                        name: name.clone(),
                        value: value.clone(),
                        span: cparser.get_merged_span(span, &value.get_span()),
                        id: cparser.get_expr_id(),
                    }));
                }
                _ => {
                    return Err(ParseError::InvalidAssignmentTarget {
                        left: cparser.peek_bw(3).clone(),
                    });
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let expr = self.and(cparser)?;
        let operator = if cparser.get_count("in_select_depth") > 0 {
            skw!(Or)
        } else {
            sym!(LogicalOr)
        };
        if cparser.match_next(&operator) {
            let op = cparser.peek_bw(1);
            let right = self.and(cparser)?;
            return Ok(Box::new(Expr::Logical {
                left: expr.clone(),
                operation: cparser.tok_type_to_op(op.tok_type.clone()),
                right: right.clone(),
                span: cparser.get_merged_span(&expr.get_span(), &right.get_span()),
                id: cparser.get_expr_id(),
            }));
        }
        Ok(expr)
    }

    fn and(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let expr = self.equality(cparser)?;
        let operator = if cparser.get_count("in_select_depth") > 0 {
            skw!(And)
        } else {
            sym!(LogicalAnd)
        };
        if cparser.match_next(&operator) {
            let op = cparser.peek_bw(1);
            let right = self.equality(cparser)?;
            return Ok(Box::new(Expr::Logical {
                left: expr.clone(),
                operation: cparser.tok_type_to_op(op.tok_type.clone()),
                right: right.clone(),
                span: cparser.get_merged_span(&expr.get_span(), &right.get_span()),
                id: cparser.get_expr_id(),
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if cparser.get_count("in_select_depth") > 0 {
            binary!(self, cparser, [sym!(BangEqual), sym!(Equal)], cmp_basic);
        } else {
            binary!(
                self,
                cparser,
                [sym!(BangEqual), sym!(EqualEqual)],
                cmp_basic
            );
        }
    }

    fn cmp_basic(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        binary!(
            self,
            cparser,
            [
                sym!(Greater),
                sym!(GreaterEqual),
                sym!(Less),
                sym!(LessEqual)
            ],
            cmp_advanced
        );
    }

    fn cmp_advanced(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let expr = self.term(cparser)?;
        let expr_conj_fst = if cparser.match_next_one_of(&[
            skw!(Is),
            skw!(Not),
            skw!(In),
            skw!(Between),
            skw!(Like),
        ]) {
            Some((*cparser.peek_bw(1)).clone().tok_type)
        } else {
            None
        };
        let expr_conj_sec = if expr_conj_fst.is_some()
            && cparser.match_next_one_of(&[
                skw!(Is),
                skw!(Not),
                skw!(In),
                skw!(Between),
                skw!(Like),
            ]) {
            Some((*cparser.peek_bw(1)).clone().tok_type)
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
            let right = self.term(cparser)?;
            return Ok(Box::new(Expr::Binary {
                left: left.clone(),
                operation,
                right: right.clone(),
                span: left.get_span().merge(&right.get_span()),
                id: cparser.get_expr_id(),
            }));
        }

        match (&expr_conj_fst, &expr_conj_sec) {
            (Some(SqlKeyword(Between)), None) => self.cmp_between(left, false, cparser),
            (Some(SqlKeyword(Not)), Some(SqlKeyword(Between))) => {
                self.cmp_between(left, true, cparser)
            }
            _ => Ok(expr),
        }
    }

    fn cmp_between(
        &mut self,
        subject: Box<Expr>,
        falsy: bool,
        cparser: &mut Parser,
    ) -> ParseResult<Box<Expr>> {
        let lower = self.term(cparser)?;

        cparser.expect(&skw!(And))?;
        let upper = self.term(cparser)?;

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
            id: cparser.get_expr_id(),
        }))
    }

    fn term(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        binary!(self, cparser, [sym!(Plus), sym!(Minus)], factor);
    }

    fn factor(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        binary!(self, cparser, [sym!(Star), sym!(Slash)], unary);
    }

    fn unary(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if cparser.match_next_one_of(&[sym!(Minus), sym!(Bang)]) {
            let token = (*cparser.peek_bw(1)).clone();
            let unary = self.unary(cparser)?;
            return Ok(Box::new(Expr::Unary {
                operation: cparser.tok_type_to_op(token.tok_type),
                expr: unary.clone(),
                span: cparser.get_merged_span(&token.span, &(unary).get_span()),
                id: cparser.get_expr_id(),
            }));
        }
        cparser.consume_call()
    }

    fn expect_get_path(
        &mut self,
        initial: Box<Expr>,
        tok: TokenType,
        cparser: &mut Parser,
    ) -> ParseResult<Box<Expr>> {
        let mut expr = initial;

        loop {
            if cparser.match_next(&sym!(LeftParen)) {
                expr = self.finish_call(expr, cparser)?;
            } else if cparser.match_next(&tok.clone()) {
                let identifier = cparser.expect(&Identifier { dollar: false })?.clone();
                expr = Box::new(Expr::Get {
                    object: expr.clone(),
                    name: identifier.extract_identifier().unwrap(),
                    span: cparser.get_merged_span(&(expr).get_span(), &identifier.span),
                    id: cparser.get_expr_id(),
                })
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // After unary
    pub fn call(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let expr = self.primary(cparser)?;

        if let Expr::Variable { name, span, id } = expr.as_ref() {
            if name.kind == IdentifierKind::Symbol {
                let next_tok = &cparser.peek_bw(0).tok_type;

                if ((next_tok == &sym!(Dot) || next_tok != &sym!(LeftParen))
                    && next_tok != &sym!(DoubleColon))
                    && cparser.get_count("in_select_depth") > 0
                {
                    let head = name.clone();
                    let mut tail: Vec<crate::ast::Identifier> = vec![];
                    while cparser.match_next(&sym!(Dot)) {
                        let identifier = cparser.expect(&Identifier { dollar: false })?.clone();
                        tail.push(identifier.extract_identifier().unwrap());
                    }
                    return Ok(Box::new(Expr::FieldPath {
                        head,
                        tail,
                        span: cparser.get_merged_span(span, &cparser.peek_bw(0).span),
                        id: *id,
                    }));
                }

                return self.expect_get_path(expr, sym!(DoubleColon), cparser);
            }
        }

        self.expect_get_path(expr, sym!(Dot), cparser)
    }

    fn finish_call(&mut self, callee: Box<Expr>, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let mut arguments: Vec<Expr> = vec![];

        if !cparser.cmp_tok(&sym!(RightParen)) {
            arguments.push(*self.expression(cparser)?);
            while cparser.match_next(&sym!(Comma)) {
                arguments.push(*self.expression(cparser)?);
            }
        }

        let paren = cparser.expect(&sym!(RightParen))?.clone();

        Ok(Box::new(Expr::Call {
            callee: callee.clone(),
            span: cparser.get_merged_span(&(callee).get_span(), &paren.span),
            args: arguments,
            id: cparser.get_expr_id(),
        }))
    }

    fn object_literal(&mut self, tok: &Token, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let mut obj_literal: FxHashMap<String, Box<Expr>> = FxHashMap::default();
        cparser.increment_count("in_object_depth");
        while !cparser.cmp_tok(&sym!(RightBrace)) {
            let key = if cparser.match_next_one_of(&[Identifier { dollar: false }, Str, Num]) {
                let key_tok = cparser.peek_bw(1).clone();
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
                    token: cparser.peek_bw(1).clone(),
                });
            };

            cparser.expect(&sym!(Colon))?;
            let value = self.expression(cparser)?;
            obj_literal.insert(key, value);
            if !cparser.match_next(&sym!(Comma)) {
                break;
            }
        }
        cparser.expect(&sym!(RightBrace))?;
        cparser.decrement_count("in_object_depth");
        Ok(Box::new(Expr::Literal {
            value: Literal::Object(obj_literal),
            raw: "".to_string(),
            span: cparser.get_merged_span(&tok.span, &cparser.peek_bw(0).span),
            id: cparser.get_expr_id(),
        }))
    }

    fn array_literal(&mut self, tok: &Token, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let mut array_literal: Vec<Expr> = vec![];
        cparser.increment_count("in_array_depth");
        while !cparser.cmp_tok(&sym!(RightBracket)) {
            let value = self.expression(cparser)?;
            array_literal.push(*value.clone());
            if !cparser.match_next(&sym!(Comma)) {
                break;
            }
        }
        cparser.expect(&sym!(RightBracket))?;
        cparser.decrement_count("in_array_depth");
        Ok(Box::new(Expr::Literal {
            value: Literal::Array(array_literal),
            raw: "".to_string(),
            span: cparser.get_merged_span(&tok.span, &cparser.peek_bw(0).span),
            id: cparser.get_expr_id(),
        }))
    }

    fn primary(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        let tok = cparser.peek_bw(0);
        cparser.increment_count("current");
        match &tok.tok_type {
            Symbol(LeftBrace) => self.object_literal(tok, cparser),
            Symbol(LeftBracket) => self.array_literal(tok, cparser),
            Symbol(LeftParen) => {
                let expr = self.expression(cparser)?;
                cparser.expect(&sym!(RightParen))?;
                Ok(Box::new(Expr::Grouping {
                    span: (expr).get_span(),
                    expr,
                    id: cparser.get_expr_id(),
                }))
            }
            True => Ok(Box::new(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: tok.span,
                id: cparser.get_expr_id(),
            })),
            False => Ok(Box::new(Expr::Literal {
                value: Literal::Bool(false),
                raw: "false".to_string(),
                span: tok.span,
                id: cparser.get_expr_id(),
            })),
            TokenType::Undefined => Ok(Box::new(Expr::Literal {
                value: Literal::Undefined,
                raw: "undefined".to_string(),
                span: tok.span,
                id: cparser.get_expr_id(),
            })),
            Str | Num => Ok(Box::new(Expr::Literal {
                value: tok.literal.clone().unwrap(),
                raw: tok.lexeme.clone().unwrap(),
                span: tok.span,
                id: cparser.get_expr_id(),
            })),
            Identifier { .. } => Ok(Box::new(Expr::Variable {
                name: tok.extract_identifier().unwrap(),
                span: tok.span,
                id: cparser.get_expr_id(),
            })),
            _ => Err(ParseError::UnexpectedToken { token: tok.clone() }),
        }
    }
}
