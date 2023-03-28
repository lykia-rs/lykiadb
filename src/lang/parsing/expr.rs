use crate::lang::parsing::token::{LiteralValue, Token};

/*

    let ex = GroupingExpr {
        expression: &BinaryExpr {
            left: &LiteralExpr {
                value: LiteralValue::Str("50".to_string())
            },
            operator: Token {
                tok_type: TokenType::Literal,
                lexeme: None,
                literal: None,
                line: 5
            },
            right: &LiteralExpr {
                value: LiteralValue::Str("50".to_string())
            }
        }
    };

    println!("{:?}", ex.expression.say_hello());

 */

pub trait Expr {
    fn say_hello(&self);
}

pub struct BinaryExpr<'a> {
    pub left: &'a dyn Expr,
    pub operator: Token,
    pub right: &'a dyn Expr
}

pub struct GroupingExpr<'a> {
    pub expression: &'a dyn Expr
}

pub struct LiteralExpr {
    pub value: LiteralValue
}

pub struct UnaryExpr<'a> {
    pub operator: Token,
    pub right: &'a dyn Expr
}

impl<'a> Expr for BinaryExpr<'a> {
    fn say_hello(&self) {
        println!("binary");
    }
}

impl<'a> Expr for GroupingExpr<'a> {
    fn say_hello(&self) {
        println!("grouping");
    }
}

impl Expr for LiteralExpr {
    fn say_hello(&self) {
        println!("literal");
    }
}

impl<'a> Expr for UnaryExpr<'a> {
    fn say_hello(&self) {
        println!("unary");
    }
}