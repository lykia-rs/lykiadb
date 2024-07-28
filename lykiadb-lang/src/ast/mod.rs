use crate::tokenizer::token::Spanned;

pub mod expr;
pub mod sql;
pub mod stmt;
pub mod visitor;

pub trait AstNode: Spanned {
    fn get_id(&self) -> usize;
}
