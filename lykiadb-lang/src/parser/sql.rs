use crate::ast::{
    expr::Expr,
    sql::{
        SqlCollectionIdentifier, SqlCompoundOperator, SqlDelete, SqlDistinct, SqlExpressionSource,
        SqlFrom, SqlInsert, SqlJoinType, SqlLimitClause, SqlOrderByClause, SqlOrdering,
        SqlProjection, SqlSelect, SqlSelectCompound, SqlSelectCore, SqlSource, SqlUpdate,
        SqlValues,
    },
    Span,
};

use super::{ParseError, ParseResult, Parser};

use crate::tokenizer::token::{SqlKeyword::*, Symbol::*, Token, TokenType, TokenType::*};
use crate::{skw, sym};

macro_rules! optional_with_expected {
    ($self: ident, $cparser: expr, $optional: expr, $expected: expr) => {
        if $cparser.match_next(&$optional) {
            let token = $cparser.expected(&$expected);
            Some(token.unwrap().clone())
        } else if $cparser.match_next(&$expected) {
            let token = $cparser.peek_bw(1);
            Some(token.clone())
        } else {
            None
        }
    };
}

pub struct SqlParser {}

impl SqlParser {
    pub fn sql_insert(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if !cparser.match_next(&skw!(Insert)) {
            return self.sql_update(cparser);
        }

        cparser.expected(&skw!(Into))?;

        if let Some(collection) = self.sql_collection_identifier(cparser)? {
            let values = if cparser.cmp_tok(&skw!(Select)) {
                let select_inner = self.sql_select_inner(cparser);

                if select_inner.is_err() {
                    return Err(select_inner.err().unwrap());
                }

                SqlValues::Select(select_inner.unwrap())
            } else if cparser.match_next(&skw!(Values)) {
                cparser.expected(&sym!(LeftParen))?;
                let mut values: Vec<Expr> = vec![];
                loop {
                    values.push(*cparser.consume_expr()?);
                    if !cparser.match_next(&sym!(Comma)) {
                        break;
                    }
                }
                cparser.expected(&sym!(RightParen))?;
                SqlValues::Values { values }
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: cparser.peek_bw(0).clone(),
                });
            };
            Ok(Box::new(Expr::Insert {
                command: SqlInsert { collection, values },
                span: Span::default(),
                id: cparser.get_expr_id(),
            }))
        } else {
            Err(ParseError::UnexpectedToken {
                token: cparser.peek_bw(0).clone(),
            })
        }
    }

    fn sql_update(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if !cparser.match_next(&skw!(Update)) {
            return self.sql_delete(cparser);
        }

        let collection = self.sql_collection_identifier(cparser)?;

        cparser.expected(&skw!(Set))?;

        let mut assignments: Vec<Expr> = vec![];

        loop {
            cparser.expected(&Identifier { dollar: false })?;
            cparser.expected(&sym!(Equal))?;
            assignments.push(*cparser.consume_expr()?);
            if !cparser.match_next(&sym!(Comma)) {
                break;
            }
        }

        let r#where = if cparser.match_next(&skw!(Where)) {
            Some(cparser.consume_expr()?)
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
            id: cparser.get_expr_id(),
        }))
    }

    fn sql_delete(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if !cparser.match_next(&skw!(Delete)) {
            return self.sql_select(cparser);
        }

        cparser.expected(&skw!(From))?;

        if let Some(collection) = self.sql_collection_identifier(cparser)? {
            let r#where = if cparser.match_next(&skw!(Where)) {
                Some(cparser.consume_expr()?)
            } else {
                None
            };

            Ok(Box::new(Expr::Delete {
                command: SqlDelete {
                    collection,
                    r#where,
                },
                span: Span::default(),
                id: cparser.get_expr_id(),
            }))
        } else {
            Err(ParseError::UnexpectedToken {
                token: cparser.peek_bw(0).clone(),
            })
        }
    }

    fn sql_collection_identifier(
        &mut self,
        cparser: &mut Parser,
    ) -> ParseResult<Option<SqlCollectionIdentifier>> {
        if cparser.cmp_tok(&Identifier { dollar: false }) {
            if cparser.match_next_all_of(&[
                Identifier { dollar: false },
                sym!(Dot),
                Identifier { dollar: false },
            ]) {
                return Ok(Some(SqlCollectionIdentifier {
                    namespace: Some(cparser.peek_bw(3).extract_identifier().unwrap()),
                    name: cparser.peek_bw(1).extract_identifier().unwrap(),
                    alias: optional_with_expected!(
                        self,
                        cparser,
                        skw!(As),
                        Identifier { dollar: false }
                    )
                    .map(|t| t.extract_identifier().unwrap()),
                }));
            }
            return Ok(Some(SqlCollectionIdentifier {
                namespace: None,
                name: cparser
                    .expected(&Identifier { dollar: false })?
                    .extract_identifier()
                    .unwrap(),
                alias: optional_with_expected!(
                    self,
                    cparser,
                    skw!(As),
                    Identifier { dollar: false }
                )
                .map(|t| t.extract_identifier().unwrap()),
            }));
        }
        Ok(None)
    }

    fn sql_select(&mut self, cparser: &mut Parser) -> ParseResult<Box<Expr>> {
        if !cparser.cmp_tok(&skw!(Select)) {
            return cparser.consume_call2();
        }

        let query: ParseResult<SqlSelect> = {
            let select_inner = self.sql_select_inner(cparser);

            if select_inner.is_err() {
                return Err(select_inner.err().unwrap());
            }

            Ok(select_inner.unwrap())
        };

        Ok(Box::new(Expr::Select {
            span: Span::default(),
            query: query.unwrap(),
            id: cparser.get_expr_id(),
        }))
    }

    fn sql_select_inner(&mut self, cparser: &mut Parser) -> ParseResult<SqlSelect> {
        cparser.increment_count("in_select_depth");
        let core: SqlSelectCore = self.sql_select_core(cparser)?;
        let order_by = if cparser.match_next(&skw!(Order)) {
            cparser.expected(&skw!(By))?;
            let mut ordering: Vec<SqlOrderByClause> = vec![];

            loop {
                let order_expr = cparser.consume_expr()?;
                let order = if cparser.match_next(&skw!(Desc)) {
                    Some(SqlOrdering::Desc)
                } else {
                    cparser.match_next(&skw!(Asc));
                    Some(SqlOrdering::Asc)
                };
                ordering.push(SqlOrderByClause {
                    expr: order_expr,
                    ordering: order.unwrap(),
                });
                if !cparser.match_next(&sym!(Comma)) {
                    break;
                }
            }

            Some(ordering)
        } else {
            None
        };

        let limit = if cparser.match_next(&skw!(Limit)) {
            let first_expr = cparser.consume_expr()?;
            let (second_expr, reverse) = if cparser.match_next(&skw!(Offset)) {
                (Some(cparser.consume_expr()?), false)
            } else if cparser.match_next(&sym!(Comma)) {
                (Some(cparser.consume_expr()?), true)
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
                }),
            }
        } else {
            None
        };

        cparser.decrement_count("in_select_depth");

        Ok(SqlSelect {
            core,
            order_by,
            limit,
        })
    }

    fn sql_select_core(&mut self, cparser: &mut Parser) -> ParseResult<SqlSelectCore> {
        cparser.expected(&skw!(Select))?;
        let distinct = if cparser.match_next(&skw!(Distinct)) {
            SqlDistinct::Distinct
        } else if cparser.match_next(&skw!(All)) {
            SqlDistinct::All
        } else {
            SqlDistinct::ImplicitAll
        };

        let projection = self.sql_select_projection(cparser)?;
        let from = self.sql_select_from(cparser)?;
        let r#where = self.sql_select_where(cparser)?;
        let group_by = self.sql_select_group_by(cparser)?;
        let having = if group_by.is_some() && cparser.match_next(&skw!(Having)) {
            Some(cparser.consume_expr()?)
        } else {
            None
        };

        let compound: Option<Box<SqlSelectCompound>> =
            if cparser.match_next_one_of(&[skw!(Union), skw!(Intersect), skw!(Except)]) {
                let op = cparser.peek_bw(1);
                let compound_op = if op.tok_type == skw!(Union) && cparser.match_next(&skw!(All)) {
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
                    core: self.sql_select_core(cparser)?,
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

    fn sql_select_projection(&mut self, cparser: &mut Parser) -> ParseResult<Vec<SqlProjection>> {
        let mut projections: Vec<SqlProjection> = vec![];
        loop {
            if cparser.match_next(&sym!(Star)) {
                projections.push(SqlProjection::All { collection: None });
            } else if cparser.match_next_all_of(&[
                Identifier { dollar: false },
                sym!(Dot),
                sym!(Star),
            ]) {
                projections.push(SqlProjection::All {
                    collection: Some(cparser.peek_bw(3).extract_identifier().unwrap()),
                });
            } else {
                let expr = cparser.consume_expr()?;
                let alias: Option<Token> =
                    optional_with_expected!(self, cparser, skw!(As), Identifier { dollar: false });
                projections.push(SqlProjection::Expr {
                    expr,
                    alias: alias.map(|t| t.extract_identifier().unwrap()),
                });
            }
            if !cparser.match_next(&sym!(Comma)) {
                break;
            }
        }
        Ok(projections)
    }

    fn sql_select_from(&mut self, cparser: &mut Parser) -> ParseResult<Option<SqlFrom>> {
        if cparser.match_next(&skw!(From)) {
            return Ok(Some(self.sql_select_from_join(cparser)?));
        }
        Ok(None)
    }

    fn sql_select_from_join(&mut self, cparser: &mut Parser) -> ParseResult<SqlFrom> {
        let mut from_group: Vec<SqlFrom> = vec![];

        loop {
            let left = self.sql_select_from_source(cparser)?;
            from_group.push(left);
            while cparser.match_next_one_of(&[
                skw!(Left),
                skw!(Right),
                skw!(Inner),
                skw!(Cross),
                skw!(Join),
            ]) {
                // If the next token is a join keyword, then it must be a join from
                let peek = cparser.peek_bw(1);
                if peek.tok_type != SqlKeyword(Join) {
                    cparser.expected(&skw!(Join))?;
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
                let right = self.sql_select_from_source(cparser)?;
                let join_constraint: Option<Box<Expr>> = if cparser.match_next(&skw!(On)) {
                    Some(cparser.consume_expr()?)
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
            if !cparser.match_next(&sym!(Comma)) {
                break;
            }
        }

        Ok(SqlFrom::Group { values: from_group })
    }

    fn sql_select_where(&mut self, cparser: &mut Parser) -> ParseResult<Option<Box<Expr>>> {
        if cparser.match_next(&skw!(Where)) {
            return Ok(Some(cparser.consume_expr()?));
        }
        Ok(None)
    }

    fn sql_select_group_by(&mut self, cparser: &mut Parser) -> ParseResult<Option<Vec<Expr>>> {
        if cparser.match_next(&skw!(Group)) {
            cparser.expected(&skw!(By))?;
            let mut groups: Vec<Expr> = vec![];

            loop {
                let sql_expr = cparser.consume_expr()?;
                groups.push(*sql_expr);
                if !cparser.match_next(&sym!(Comma)) {
                    break;
                }
            }
            Ok(Some(groups))
        } else {
            Ok(None)
        }
    }

    fn sql_select_from_source(&mut self, cparser: &mut Parser) -> ParseResult<SqlFrom> {
        if cparser.match_next(&sym!(LeftParen)) {
            if cparser.cmp_tok(&skw!(Select)) {
                let subquery = Box::new(self.sql_select_inner(cparser)?);
                cparser.expected(&sym!(RightParen))?;
                let alias: Option<Token> =
                    optional_with_expected!(self, cparser, skw!(As), Identifier { dollar: false });
                return Ok(SqlFrom::Select {
                    subquery,
                    alias: alias.map(|t| t.extract_identifier().unwrap()),
                });
            }
            // If the next token is a left paren, then it must be either a select statement or a recursive "from" clause
            let parsed = self.sql_select_from_join(cparser)?;
            cparser.expected(&sym!(RightParen))?;
            Ok(parsed)
        } else if let Some(collection) = self.sql_collection_identifier(cparser)? {
            return Ok(SqlFrom::Source(SqlSource::Collection(collection)));
        } else {
            let expr = cparser.consume_expr()?;
            cparser.expected(&skw!(As))?;
            let identifier = cparser.expected(&Identifier { dollar: false })?.clone();
            return Ok(SqlFrom::Source(SqlSource::Expr(SqlExpressionSource {
                expr,
                alias: identifier.extract_identifier().unwrap(),
            })));
        }
    }
}
