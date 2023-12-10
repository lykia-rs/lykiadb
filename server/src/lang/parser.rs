use super::ast::expr::{Expr, ExprId};
use super::ast::sql::{
    SelectCore, SqlCollectionSubquery, SqlCompoundOperator, SqlDistinct, SqlExpr, SqlFrom,
    SqlOrdering, SqlProjection, SqlSelect,
};
use super::ast::stmt::{Stmt, StmtId};
use super::ast::{Literal, ParserArena};
use crate::lang::token::{
    Keyword, Keyword::*, Span, Spanned, SqlKeyword::*, Symbol::*, Token, TokenType, TokenType::*,
};
use crate::{kw, skw, sym};
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    arena: ParserArena,
}

pub struct Parsed {
    pub program: StmtId,
    pub arena: Rc<ParserArena>,
}

impl Parsed {
    pub fn new(program: StmtId, arena: Rc<ParserArena>) -> Parsed {
        Parsed { program, arena }
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

            current_expr = $self.arena.expression(Expr::Binary {
                left,
                symbol: token.tok_type,
                right,
                span: $self.get_merged_span(
                    $self.arena.get_expression(left),
                    $self.arena.get_expression(right),
                ),
            });
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
        let program = parser.program()?;
        Ok(Parsed::new(program, Rc::new(parser.arena)))
    }

    fn program(&mut self) -> ParseResult<StmtId> {
        let mut statements: Vec<StmtId> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.expected(Eof)?;
        Ok(self.arena.statement(Stmt::Program {
            stmts: statements.clone(),
            span: self.get_merged_span(
                self.arena.get_statement(statements[0]),
                self.arena.get_statement(statements[statements.len() - 1]),
            ),
        }))
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
        let if_tok = self.peek_bw(1);
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let if_branch = self.statement()?;

        if self.match_next(kw!(Else)) {
            let else_branch = self.statement()?;
            return Ok(self.arena.statement(Stmt::If {
                condition,
                body: if_branch,
                r#else: Some(else_branch),
                span: self.get_merged_span(&if_tok.span, self.arena.get_statement(else_branch)),
            }));
        }
        Ok(self.arena.statement(Stmt::If {
            condition,
            body: if_branch,
            r#else: None,
            span: self.get_merged_span(&if_tok.span, self.arena.get_statement(if_branch)),
        }))
    }

    fn loop_statement(&mut self) -> ParseResult<StmtId> {
        let loop_tok = self.peek_bw(1);
        let inner_stmt = self.declaration()?;
        Ok(self.arena.statement(Stmt::Loop {
            condition: None,
            body: inner_stmt,
            post: None,
            span: self.get_merged_span(&loop_tok.span, self.arena.get_statement(inner_stmt)),
        }))
    }

    fn while_statement(&mut self) -> ParseResult<StmtId> {
        let while_tok = self.peek_bw(1);
        self.expected(sym!(LeftParen))?;
        let condition = self.expression()?;
        self.expected(sym!(RightParen))?;
        let inner_stmt = self.declaration()?;

        Ok(self.arena.statement(Stmt::Loop {
            condition: Some(condition),
            body: inner_stmt,
            post: None,
            span: self.get_merged_span(&while_tok.span, self.arena.get_statement(inner_stmt)),
        }))
    }

    fn return_statement(&mut self) -> ParseResult<StmtId> {
        let ret_tok = self.peek_bw(1);
        let mut expr: Option<ExprId> = None;
        if !self.cmp_tok(&sym!(Semicolon)) {
            expr = Some(self.expression()?);
        }
        self.expected(sym!(Semicolon))?;

        if expr.is_none() {
            return Ok(self.arena.statement(Stmt::Return {
                span: ret_tok.span,
                expr: None,
            }));
        }

        Ok(self.arena.statement(Stmt::Return {
            span: self.get_merged_span(&ret_tok.span, self.arena.get_expression(expr.unwrap())),
            expr,
        }))
    }

    fn for_statement(&mut self) -> ParseResult<StmtId> {
        let for_tok = self.peek_bw(1);
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
            Some(self.arena.statement(Stmt::Expression {
                expr: wrapped,
                span: self.arena.get_expression(wrapped).get_span(),
            }))
        };

        let inner_stmt = self.declaration()?;

        if initializer.is_none() {
            return Ok(self.arena.statement(Stmt::Loop {
                condition,
                body: inner_stmt,
                post: increment,
                span: self.get_merged_span(&for_tok.span, self.arena.get_statement(inner_stmt)),
            }));
        }
        let loop_stmt = self.arena.statement(Stmt::Loop {
            condition,
            body: inner_stmt,
            post: increment,
            span: self.get_merged_span(&for_tok.span, self.arena.get_statement(inner_stmt)),
        });
        Ok(self.arena.statement(Stmt::Block {
            stmts: vec![initializer.unwrap(), loop_stmt],
            span: self.get_merged_span(&for_tok.span, self.arena.get_statement(inner_stmt)),
        }))
    }

    fn block(&mut self) -> ParseResult<StmtId> {
        let mut statements: Vec<StmtId> = vec![];

        while !self.cmp_tok(&sym!(RightBrace)) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.expected(sym!(RightBrace))?;

        Ok(self.arena.statement(Stmt::Block {
            stmts: statements.clone(),
            span: self.get_merged_span(
                self.arena.get_statement(statements[0]),
                self.arena.get_statement(statements[statements.len() - 1]),
            ),
        }))
    }

    fn break_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Break { span: tok.span }))
    }

    fn continue_statement(&mut self) -> ParseResult<StmtId> {
        let tok = self.peek_bw(1);
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Continue { span: tok.span }))
    }

    fn expression_statement(&mut self) -> ParseResult<StmtId> {
        let expr = self.expression()?;
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Expression {
            expr,
            span: self.arena.get_expression(expr).get_span(),
        }))
    }

    fn var_declaration(&mut self) -> ParseResult<StmtId> {
        let var_tok = self.peek_bw(1);
        let ident = self.expected(Identifier { dollar: true })?.clone();
        let expr = match self.match_next(sym!(Equal)) {
            true => self.expression()?,
            false => self.arena.expression(Expr::Literal {
                value: Literal::Undefined,
                raw: "undefined".to_string(),
                span: self.get_merged_span(&var_tok.span, &ident.span),
            }),
        };
        self.expected(sym!(Semicolon))?;
        Ok(self.arena.statement(Stmt::Declaration {
            dst: ident,
            expr,
            span: self.get_merged_span(&var_tok.span, &self.arena.get_expression(expr).get_span()),
        }))
    }

    fn fun_declaration(&mut self) -> ParseResult<ExprId> {
        let fun_tok = self.peek_bw(1);

        let token = if self.cmp_tok(&Identifier { dollar: false }) {
            Some(self.expected(Identifier { dollar: false })?.clone())
        } else {
            None
        };

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
        let stmt_idx = self.block()?;

        let inner_stmt = self.arena.get_statement(stmt_idx);

        let body: Vec<StmtId> = match inner_stmt {
            Stmt::Block { stmts, span: _ } => stmts.clone(),
            _ => vec![stmt_idx],
        };

        Ok(self.arena.expression(Expr::Function {
            name: token,
            parameters,
            body: Rc::new(body),
            span: self.get_merged_span(
                &fun_tok.span,
                &self.arena.get_statement(stmt_idx).get_span(),
            ),
        }))
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
                Expr::Variable { name, span: _ } => {
                    return Ok(self.arena.expression(Expr::Assignment {
                        dst: name.clone(),
                        expr: value,
                        span: self.get_merged_span(
                            &name.span,
                            &self.arena.get_expression(value).get_span(),
                        ),
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

    fn or(&mut self) -> ParseResult<ExprId> {
        let expr = self.and()?;
        if self.match_next(kw!(Keyword::Or)) {
            let op = self.peek_bw(1);
            let right = self.and()?;
            return Ok(self.arena.expression(Expr::Logical {
                left: expr,
                symbol: op.tok_type.clone(),
                right,
                span: self.get_merged_span(
                    self.arena.get_expression(expr),
                    self.arena.get_expression(right),
                ),
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
                symbol: op.tok_type.clone(),
                right,
                span: self.get_merged_span(
                    self.arena.get_expression(expr),
                    self.arena.get_expression(right),
                ),
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
            return Ok(self.arena.expression(Expr::Unary {
                symbol: token.tok_type,
                expr: unary,
                span: self
                    .get_merged_span(&token.span, &self.arena.get_expression(unary).get_span()),
            }));
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
            let compound_op = if op.tok_type == skw!(Union) && self.match_next(skw!(All)) {
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
            let secondary_core = self.select_core()?;
            compounds.push((compound_op, secondary_core))
        }
        let order_by = if self.match_next(skw!(Order)) {
            self.expected(skw!(By))?;
            let mut ordering: Vec<(SqlExpr, SqlOrdering)> = vec![];

            loop {
                let order_expr = self.expression()?;
                let order = if self.match_next(skw!(Desc)) {
                    Some(SqlOrdering::Desc)
                } else {
                    Some(SqlOrdering::Asc)
                };
                ordering.push((SqlExpr::Default(order_expr), order.unwrap()));
                if !self.match_next(sym!(Comma)) {
                    break;
                }
            }

            Some(ordering)
        } else {
            None
        };

        let limit = if self.match_next(skw!(Limit)) {
            let first_expr = SqlExpr::Default(self.expression()?);
            let (second_expr, reverse) = if self.match_next(skw!(Offset)) {
                (Some(SqlExpr::Default(self.expression()?)), true)
            } else if self.match_next(sym!(Comma)) {
                (Some(SqlExpr::Default(self.expression()?)), false)
            } else {
                (None, false)
            };

            if second_expr.is_some() && reverse {
                Some((second_expr.unwrap(), Some(first_expr)))
            } else {
                Some((first_expr, second_expr))
            }
        } else {
            None
        };

        Ok(self.arena.expression(Expr::Select {
            span: Span::default(),
            query: SqlSelect {
                core,
                compound: compounds,
                order_by,
                limit,
            },
        }))
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

        let projection = self.sql_projection();
        let from = self.sql_from()?;
        let r#where = self.sql_where()?;
        let group_by = self.sql_group_by()?;
        let having = if group_by.is_some() && self.match_next(skw!(Having)) {
            Some(SqlExpr::Default(self.expression()?))
        } else {
            None
        };

        Ok(SelectCore {
            distinct,
            projection,
            from,
            r#where,
            group_by,
            having,
        })
    }

    fn sql_group_by(&mut self) -> ParseResult<Option<Vec<SqlExpr>>> {
        if self.match_next(skw!(Group)) {
            self.expected(skw!(By))?;
            let mut groups: Vec<SqlExpr> = vec![];

            loop {
                let group_expr = self.expression()?;
                groups.push(SqlExpr::Default(group_expr));
                if !self.match_next(sym!(Comma)) {
                    break;
                }
            }
            Ok(Some(groups))
        } else {
            Ok(None)
        }
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
        // TODO(vck): Add support for collection selectors
        projections
    }

    fn sql_from(&mut self) -> ParseResult<Option<SqlFrom>> {
        if self.match_next(skw!(From)) {
            let token = self.expected(Identifier { dollar: false });
            return Ok(Some(SqlFrom::CollectionSubquery(vec![
                SqlCollectionSubquery::Simple {
                    namespace: None,
                    collection: token.unwrap().clone(),
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

    fn call(&mut self) -> ParseResult<ExprId> {
        let mut expr = self.primary()?;

        loop {
            if self.match_next(sym!(LeftParen)) {
                expr = self.finish_call(expr)?;
            } else if self.match_next(sym!(Dot)) {
                let identifier = self.expected(Identifier { dollar: false })?.clone();
                expr = self.arena.expression(Expr::Get {
                    object: expr,
                    name: identifier.clone(),
                    span: self.get_merged_span(
                        &self.arena.get_expression(expr).get_span(),
                        &identifier.span,
                    ),
                })
            } else {
                break;
            }
        }

        Ok(expr)
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
            span: self.get_merged_span(&self.arena.get_expression(callee).get_span(), &paren.span),
            args: arguments,
        }))
    }

    fn object_literal(&mut self, tok: &Token) -> ParseResult<ExprId> {
        let mut obj_literal: FxHashMap<String, ExprId> = FxHashMap::default();
        while !self.cmp_tok(&sym!(RightBrace)) {
            let key = self.expected(Identifier { dollar: false })?.clone();
            self.expected(sym!(Colon))?;
            let value = self.expression()?;
            obj_literal.insert(key.lexeme.unwrap(), value);
            if !self.match_next(sym!(Comma)) {
                break;
            }
        }
        self.expected(sym!(RightBrace))?;
        Ok(self.arena.expression(Expr::Literal {
            value: Literal::Object(obj_literal),
            raw: "".to_string(),
            span: self.get_merged_span(&tok.span, &self.peek_bw(0).span),
        }))
    }

    fn array_literal(&mut self, tok: &Token) -> ParseResult<ExprId> {
        let mut array_literal: Vec<ExprId> = vec![];
        while !self.cmp_tok(&sym!(RightBracket)) {
            let value = self.expression()?;
            array_literal.push(value);
            if !self.match_next(sym!(Comma)) {
                break;
            }
        }
        self.expected(sym!(RightBracket))?;
        Ok(self.arena.expression(Expr::Literal {
            value: Literal::Array(array_literal),
            raw: "".to_string(),
            span: self.get_merged_span(&tok.span, &self.peek_bw(0).span),
        }))
    }

    fn primary(&mut self) -> ParseResult<ExprId> {
        let tok = self.peek_bw(0);
        self.current += 1;
        match &tok.tok_type {
            Symbol(LeftBrace) => self.object_literal(tok),
            Symbol(LeftBracket) => self.array_literal(tok),
            Symbol(LeftParen) => {
                let expr = self.expression()?;
                self.expected(sym!(RightParen))?;
                Ok(self.arena.expression(Expr::Grouping {
                    span: self.arena.get_expression(expr).get_span(),
                    expr,
                }))
            }
            True => Ok(self.arena.expression(Expr::Literal {
                value: Literal::Bool(true),
                raw: "true".to_string(),
                span: tok.span,
            })),
            False => Ok(self.arena.expression(Expr::Literal {
                value: Literal::Bool(false),
                raw: "false".to_string(),
                span: tok.span,
            })),
            TokenType::Null => Ok(self.arena.expression(Expr::Literal {
                value: Literal::Null,
                raw: "null".to_string(),
                span: tok.span,
            })),
            Str | Num => Ok(self.arena.expression(Expr::Literal {
                value: tok.literal.clone().unwrap(),
                raw: tok.lexeme.clone().unwrap(),
                span: tok.span,
            })),
            Identifier { dollar: _ } => Ok(self.arena.expression(Expr::Variable {
                name: tok.clone(),
                span: tok.span,
            })),
            _ => Err(ParseError::UnexpectedToken { token: tok.clone() }),
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

    fn get_merged_span(&self, left: &impl Spanned, right: &impl Spanned) -> Span {
        let left_span = &left.get_span();
        let right_span = &right.get_span();
        left_span.merge(right_span)
    }
}

#[cfg(test)]
mod test {

    use assert_json_diff::assert_json_eq;
    use serde_json::{json, Value};

    use crate::lang::{scanner::Scanner, token::Token};

    use super::*;

    fn get_tokens(source: &str) -> Vec<Token> {
        return Scanner::scan(source).unwrap();
    }

    fn compare_parsed_to_expected(source: &str, expected: Value) {
        let tokens = get_tokens(source);
        let mut parsed = Parser::parse(&tokens).unwrap();
        let actual = parsed.to_json();
        assert_json_eq!(actual, expected);
    }

    #[test]
    fn test_parse_literal_expression() {
        compare_parsed_to_expected(
            "1;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Literal",
                            "value": "Num(1.0)",
                            "raw": "1"
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_unary_expression() {
        compare_parsed_to_expected(
            "-1;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Unary",
                            "operator": {
                                "Symbol": "Minus"
                            },
                            "value": {
                                "type": "Expr::Literal",
                                "value": "Num(1.0)",
                                "raw": "1"
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_binary_expression() {
        compare_parsed_to_expected(
            "1 + 2;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Binary",
                            "left": {
                                "type": "Expr::Literal",
                                "value": "Num(1.0)",
                                "raw": "1"
                            },
                            "operator": {
                                "Symbol": "Plus"
                            },
                            "right": {
                                "type": "Expr::Literal",
                                "value": "Num(2.0)",
                                "raw": "2"
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_grouping_expression() {
        compare_parsed_to_expected(
            "(1 + 2) * (3 / (4 - 7));",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Binary",
                            "left": {
                                "type": "Expr::Grouping",
                                "value": {
                                    "type": "Expr::Binary",
                                    "left": {
                                        "raw": "1",
                                        "type": "Expr::Literal",
                                        "value": "Num(1.0)",
                                    },
                                    "operator": {
                                        "Symbol": "Plus"
                                    },
                                    "right": {
                                        "raw": "2",
                                        "type": "Expr::Literal",
                                        "value": "Num(2.0)",
                                    }
                                }
                            },
                            "operator": {
                                "Symbol": "Star"
                            },
                            "right": {
                                "type": "Expr::Grouping",
                                "value": {
                                    "type": "Expr::Binary",
                                    "left": {
                                        "raw": "3",
                                        "type": "Expr::Literal",
                                        "value": "Num(3.0)",
                                    },
                                    "operator": {
                                        "Symbol": "Slash"
                                    },
                                    "right": {
                                        "type": "Expr::Grouping",
                                        "value": {
                                            "type": "Expr::Binary",
                                            "left": {
                                                "raw": "4",
                                                "type": "Expr::Literal",
                                                "value": "Num(4.0)",
                                            },
                                            "operator":  {
                                                "Symbol": "Minus"
                                            },
                                            "right": {
                                                "raw": "7",
                                                "type": "Expr::Literal",
                                                "value": "Num(7.0)",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                ]
            }),
        );
    }

    #[test]
    fn test_parse_variable_expression() {
        compare_parsed_to_expected(
            "$a;",
            json!({
                "type": "Stmt::Program",
                "body": [
                    {
                        "type": "Stmt::Expression",
                        "value": {
                            "type": "Expr::Variable",
                            "value": "$a",
                        }
                    }
                ]
            }),
        );
    }
}
