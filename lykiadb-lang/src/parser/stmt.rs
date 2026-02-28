use super::{ParseResult, Parser};
use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::ast::{Literal, Spanned};
use crate::tokenizer::token::{Keyword::*, SqlKeyword::*, Symbol::*, TokenType, TokenType::*};
use crate::{kw, skw, sym};

pub struct StmtParser {}

// a macro for repeating match_next pattern
macro_rules! match_next {
    ($self: ident, $cparser: expr, $t: expr, $callee: ident) => {
        if $cparser.match_next($t) {
            return $self.$callee($cparser);
        }
    };
}

impl StmtParser {
    pub fn declaration(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        match_next!(self, cparser, &kw!(Var), var_declaration);
        self.statement(cparser)
    }

    fn statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        match_next!(self, cparser, &kw!(If), if_statement);
        match_next!(self, cparser, &kw!(While), while_statement);
        match_next!(self, cparser, &kw!(For), for_statement);
        match_next!(self, cparser, &kw!(Loop), loop_statement);
        match_next!(self, cparser, &kw!(Return), return_statement);
        match_next!(self, cparser, &skw!(Explain), explain_statement);
        if cparser.peek_next_all_of(&[sym!(LeftBrace), Identifier { dollar: false }, sym!(Colon)])
            || cparser.peek_next_all_of(&[sym!(LeftBrace), Str, sym!(Colon)])
            || cparser.peek_next_all_of(&[sym!(LeftBrace), Num, sym!(Colon)])
            || cparser.peek_next_all_of(&[sym!(LeftBrace), sym!(RightBrace)])
        {
            return self.expression_statement(cparser);
        }
        match_next!(self, cparser, &sym!(LeftBrace), block);
        self.expression_statement(cparser)
    }

    fn if_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let if_tok = cparser.peek_bw(1);
        cparser.expect(&sym!(LeftParen))?;
        let condition = cparser.consume_expr()?;
        cparser.expect(&sym!(RightParen))?;
        let if_branch = self.statement(cparser)?;

        if cparser.match_next(&kw!(Else)) {
            let else_branch = self.statement(cparser)?;
            return Ok(Box::new(Stmt::If {
                condition,
                body: if_branch.clone(),
                r#else_body: Some(else_branch.clone()),
                span: cparser.get_merged_span(&if_tok.span, &else_branch.get_span()),
            }));
        }
        Ok(Box::new(Stmt::If {
            condition,
            body: if_branch.clone(),
            r#else_body: None,
            span: cparser.get_merged_span(&if_tok.span, &if_branch.get_span()),
        }))
    }

    fn loop_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let loop_tok = cparser.peek_bw(1);
        cparser.expect(&sym!(LeftBrace))?;
        let inner_stmt = self.block(cparser)?;
        cparser.match_next(&sym!(Semicolon));
        Ok(Box::new(Stmt::Loop {
            condition: None,
            body: inner_stmt.clone(),
            post: None,
            span: cparser.get_merged_span(&loop_tok.span, &inner_stmt.get_span()),
        }))
    }

    fn while_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let while_tok = cparser.peek_bw(1);
        cparser.expect(&sym!(LeftParen))?;
        let condition = cparser.consume_expr()?;
        cparser.expect(&sym!(RightParen))?;
        cparser.expect(&sym!(LeftBrace))?;
        let inner_stmt = self.block(cparser)?;
        cparser.match_next(&sym!(Semicolon));

        Ok(Box::new(Stmt::Loop {
            condition: Some(condition),
            body: inner_stmt.clone(),
            post: None,
            span: cparser.get_merged_span(&while_tok.span, &inner_stmt.get_span()),
        }))
    }

    fn return_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let ret_tok = cparser.peek_bw(1);
        let mut expr: Option<Box<Expr>> = None;
        if !cparser.cmp_tok(&sym!(Semicolon)) {
            expr = Some(cparser.consume_expr()?);
        }
        cparser.expect(&sym!(Semicolon))?;

        if expr.is_none() {
            return Ok(Box::new(Stmt::Return {
                span: ret_tok.span,
                expr: None,
            }));
        }

        Ok(Box::new(Stmt::Return {
            span: expr
                .as_ref()
                .map(|e| cparser.get_merged_span(&ret_tok.span, &e.get_span()))
                .unwrap_or(ret_tok.span),
            expr,
        }))
    }

    fn for_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let for_tok = cparser.peek_bw(1);
        cparser.expect(&sym!(LeftParen))?;

        let initializer = if cparser.match_next(&sym!(Semicolon)) {
            None
        } else {
            Some(self.declaration(cparser)?)
        };

        let condition = if cparser.match_next(&sym!(Semicolon)) {
            None
        } else {
            let wrapped = cparser.consume_expr()?;
            cparser.expect(&sym!(Semicolon))?;
            Some(wrapped)
        };

        let increment = if cparser.match_next(&sym!(RightParen)) {
            None
        } else {
            let wrapped = cparser.consume_expr()?;
            cparser.expect(&sym!(RightParen))?;
            Some(Box::new(Stmt::Expression {
                expr: wrapped.clone(),
                span: wrapped.get_span(),
            }))
        };

        cparser.expect(&sym!(LeftBrace))?;
        let inner_stmt = self.block(cparser)?;
        cparser.match_next(&sym!(Semicolon));

        let header = match initializer {
            Some(init) => init,
            None => {
                return Ok(Box::new(Stmt::Loop {
                    condition,
                    body: inner_stmt.clone(),
                    post: increment,
                    span: cparser.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
                }));
            }
        };

        let loop_stmt = Box::new(Stmt::Loop {
            condition,
            body: inner_stmt.clone(),
            post: increment,
            span: cparser.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
        });
        Ok(Box::new(Stmt::Block {
            body: vec![*header, *loop_stmt],
            span: cparser.get_merged_span(&for_tok.span, &inner_stmt.get_span()),
        }))
    }

    pub fn block(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];

        let opening_brace = cparser.peek_bw(1);

        while !cparser.cmp_tok(&sym!(RightBrace)) && !cparser.is_at_end() {
            statements.push(*self.declaration(cparser)?);
        }

        let closing_brace = cparser.peek_bw(1);

        cparser.expect(&sym!(RightBrace))?;

        Ok(Box::new(Stmt::Block {
            body: statements.clone(),
            span: cparser.get_merged_span(&opening_brace.span, &closing_brace.span),
        }))
    }

    fn explain_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let expr = cparser.consume_expr()?;
        let span = expr.get_span();
        cparser.expect(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Explain { expr, span }))
    }

    fn expression_statement(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let expr = cparser.consume_expr()?;
        let span = expr.get_span();
        cparser.expect(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Expression { expr, span }))
    }

    fn var_declaration(&mut self, cparser: &mut Parser) -> ParseResult<Box<Stmt>> {
        let var_tok = cparser.peek_bw(1);
        let ident = cparser.expect(&Identifier { dollar: true })?.clone();
        let expr = match cparser.match_next(&sym!(Equal)) {
            true => cparser.consume_expr()?,
            false => {
                let node = Expr::Literal {
                    value: Literal::Undefined,
                    raw: "undefined".to_string(),
                    span: var_tok.span,
                    id: cparser.get_expr_id(),
                };
                Box::new(node)
            }
        };
        cparser.expect(&sym!(Semicolon))?;
        Ok(Box::new(Stmt::Declaration {
            dst: ident.extract_identifier()?,
            expr: expr.clone(),
            span: cparser.get_merged_span(&var_tok.span, &expr.get_span()),
        }))
    }
}
