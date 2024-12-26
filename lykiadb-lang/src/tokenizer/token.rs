use phf::phf_map;
use serde::{Deserialize, Serialize};

use crate::ast::{Identifier, Literal, Span};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Symbol {
    Comma,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Bang,
    Dot,
    DoubleColon,
    Minus,
    Plus,
    Slash,
    Star,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    Str,
    Num,
    Undefined,
    False,
    True,
    //
    Identifier { dollar: bool },
    //
    Symbol(Symbol),
    Keyword(Keyword),
    SqlKeyword(SqlKeyword),
    //
    Eof,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Keyword {
    Class,
    Else,
    Fun,
    If,
    Break,
    Continue,
    Return,
    Super,
    This,
    Var,
    While,
    For,
    Loop,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SqlKeyword {
    /*
        Bool,
        Boolean,
        Char,
        String,
        Varchar,
        Text,
        Time,
        Int,
        Integer,
        Float,
        Double,
    */
    //
    Begin,
    Transaction,
    Rollback,
    Commit,
    //
    Where,
    Having,
    Asc,
    Desc,
    Order,
    By,
    Explain,
    Offset,
    Limit,
    And,
    Or,
    //
    Is,
    Not,
    Like,
    In,
    Between,
    //
    Join,
    Inner,
    Right,
    Left,
    On,
    //
    Create,
    Insert,
    Update,
    Delete,
    Drop,
    Into,
    Values,
    Index,
    Collection,
    //
    Select,
    From,
    As,
    //
    Cross,
    Default,
    Group,
    Key,
    Of,
    Only,
    Primary,
    References,
    Set,
    System,
    Unique,
    Read,
    Write,
    //
    Union,
    All,
    Intersect,
    Except,
    Distinct,
}

#[macro_export]
macro_rules! kw {
    ($val: expr) => {
        TokenType::Keyword($val)
    };
}

#[macro_export]
macro_rules! skw {
    ($val: expr) => {
        TokenType::SqlKeyword($val)
    };
}

#[macro_export]
macro_rules! sym {
    ($val: expr) => {
        TokenType::Symbol($val)
    };
}

pub static SYMBOLS: phf::Map<char, TokenType> = phf_map! {
    '(' => sym!(Symbol::LeftParen),
    ')' => sym!(Symbol::RightParen),
    '{' => sym!(Symbol::LeftBrace),
    '}' => sym!(Symbol::RightBrace),
    ',' => sym!(Symbol::Comma),
    ':' => sym!(Symbol::Colon),
    '.' => sym!(Symbol::Dot),
    '-' => sym!(Symbol::Minus),
    '+' => sym!(Symbol::Plus),
    ';' => sym!(Symbol::Semicolon),
    '*' => sym!(Symbol::Star),
    '[' => sym!(Symbol::LeftBracket),
    ']' => sym!(Symbol::RightBracket),
};

pub static GENERIC_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "class" => kw!(Keyword::Class),
    "else" => kw!(Keyword::Else),
    "for" => kw!(Keyword::For),
    "function" => kw!(Keyword::Fun),
    "if" => kw!(Keyword::If),
    "break" => kw!(Keyword::Break),
    "continue" => kw!(Keyword::Continue),
    "return" => kw!(Keyword::Return),
    "super" => kw!(Keyword::Super),
    "this" => kw!(Keyword::This),
    "var" => kw!(Keyword::Var),
    "while" => kw!(Keyword::While),
    "loop" => kw!(Keyword::Loop),
    //
    "undefined" => TokenType::Undefined,
    "false" => TokenType::False,
    "true" => TokenType::True,
};

pub static SQL_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    //
    "ALL" => skw!(SqlKeyword::All),
    "DISTINCT" => skw!(SqlKeyword::Distinct),
    "UNION" => skw!(SqlKeyword::Union),
    "INTERSECT" => skw!(SqlKeyword::Intersect),
    "EXCEPT" => skw!(SqlKeyword::Except),
    "BEGIN" => skw!(SqlKeyword::Begin),
    "TRANSACTION" => skw!(SqlKeyword::Transaction),
    "ROLLBACK" => skw!(SqlKeyword::Rollback),
    "COMMIT" => skw!(SqlKeyword::Commit),
    "WHERE" => skw!(SqlKeyword::Where),
    "HAVING" => skw!(SqlKeyword::Having),
    "ASC" => skw!(SqlKeyword::Asc),
    "DESC" => skw!(SqlKeyword::Desc),
    "ORDER" => skw!(SqlKeyword::Order),
    "BY" => skw!(SqlKeyword::By),
    "AND" => skw!(SqlKeyword::And),
    "OR" => skw!(SqlKeyword::Or),
    "EXPLAIN" => skw!(SqlKeyword::Explain),
    "IS" => skw!(SqlKeyword::Is),
    "NOT" => skw!(SqlKeyword::Not),
    "LIKE" => skw!(SqlKeyword::Like),
    "IN" => skw!(SqlKeyword::In),
    "BETWEEN" => skw!(SqlKeyword::Between),
    "OFFSET" => skw!(SqlKeyword::Offset),
    "LIMIT" => skw!(SqlKeyword::Limit),
    "JOIN" => skw!(SqlKeyword::Join),
    "INNER" => skw!(SqlKeyword::Inner),
    "RIGHT" => skw!(SqlKeyword::Right),
    "LEFT" => skw!(SqlKeyword::Left),
    "ON" => skw!(SqlKeyword::On),
    "CREATE" => skw!(SqlKeyword::Create),
    "INSERT" => skw!(SqlKeyword::Insert),
    "UPDATE" => skw!(SqlKeyword::Update),
    "DELETE" => skw!(SqlKeyword::Delete),
    "DROP" => skw!(SqlKeyword::Drop),
    "INTO" => skw!(SqlKeyword::Into),
    "VALUES" => skw!(SqlKeyword::Values),
    "INDEX" => skw!(SqlKeyword::Index),
    "SELECT" => skw!(SqlKeyword::Select),
    "FROM" => skw!(SqlKeyword::From),
    "AS" => skw!(SqlKeyword::As),
    "CROSS" => skw!(SqlKeyword::Cross),
    "DEFAULT" => skw!(SqlKeyword::Default),
    "GROUP" => skw!(SqlKeyword::Group),
    "KEY" => skw!(SqlKeyword::Key),
    "OF" => skw!(SqlKeyword::Of),
    "ONLY" => skw!(SqlKeyword::Only),
    "PRIMARY" => skw!(SqlKeyword::Primary),
    "REFERENCES" => skw!(SqlKeyword::References),
    "SET" => skw!(SqlKeyword::Set),
    "SYSTEM" => skw!(SqlKeyword::System),
    "COLLECTION" => skw!(SqlKeyword::Collection),
    "UNIQUE" => skw!(SqlKeyword::Unique),
    "READ" => skw!(SqlKeyword::Read),
    "WRITE" => skw!(SqlKeyword::Write),
};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub tok_type: TokenType,
    pub literal: Option<Literal>,
    pub lexeme: Option<String>,
    pub span: Span,
}

impl Token {
    pub fn extract_identifier(&self) -> Option<Identifier> {
        match &self.tok_type {
            TokenType::Identifier { dollar } => Some(Identifier {
                name: self.lexeme.clone().unwrap(),
                dollar: *dollar,
                span: self.span,
            }),
            _ => None,
        }
    }
}
