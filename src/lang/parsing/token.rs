use std::rc::Rc;
use phf::phf_map;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum RV {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Object(FxHashMap<String, RV>),
    Array(Vec<RV>),
    Callable(Option<usize>, Rc<Function>),
    Undefined,
    NaN,
    Null,
}

impl Eq for RV {}

impl<'de> Deserialize<'de> for RV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::String(s) => Ok(RV::Str(Rc::new(s))),
            serde_json::Value::Number(n) => Ok(RV::Num(n.as_f64().unwrap())),
            serde_json::Value::Bool(b) => Ok(RV::Bool(b)),
            serde_json::Value::Null => Ok(RV::Null),
            _ => Ok(RV::Undefined)
        }
    }
}

impl<'se> Serialize for RV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            RV::Str(s) => serializer.serialize_str(s),
            RV::Num(n) => serializer.serialize_f64(*n),
            RV::Bool(b) => serializer.serialize_bool(*b),
            RV::Undefined => serializer.serialize_none(),
            RV::NaN => serializer.serialize_none(),
            RV::Null => serializer.serialize_none(),
            RV::Callable(_, _) => serializer.serialize_none(),
            RV::Array(_) => serializer.serialize_none(),
            RV::Object(_) => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Symbol {
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Bang,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
    Str,
    Num,
    Null,
    False,
    True,
    //
    Identifier {
        dollar: bool,
    },
    //
    Symbol(Symbol),
    Keyword(Keyword),
    SqlKeyword(SqlKeyword),
    //
    Eof
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Keyword {
    And,
    Or,
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
    Is,
    Not,
    Null,
    Offset,
    Like,
    Limit,
    And,
    Or,
    //
    Join,
    Inner,
    Outer,
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
    Table,
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
}

#[macro_export] macro_rules! kw {
    ($val: expr) => {
        TokenType::Keyword($val)
    }
}

#[macro_export] macro_rules! skw {
    ($val: expr) => {
        TokenType::SqlKeyword($val)
    }
}

#[macro_export] macro_rules! sym {
    ($val: expr) => {
        TokenType::Symbol($val)
    }
}

pub static SYMBOLS: phf::Map<char, TokenType> = phf_map! {
    '(' => sym!(Symbol::LeftParen),
    ')' => sym!(Symbol::RightParen),
    '{' => sym!(Symbol::LeftBrace),
    '}' => sym!(Symbol::RightBrace),
    ',' => sym!(Symbol::Comma),
    '.' => sym!(Symbol::Dot),
    '-' => sym!(Symbol::Minus),
    '+' => sym!(Symbol::Plus),
    ';' => sym!(Symbol::Semicolon),
    '*' => sym!(Symbol::Star),
};

pub static CASE_SNS_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => kw!(Keyword::And),
    "class" => kw!(Keyword::Class),
    "else" => kw!(Keyword::Else),
    "for" => kw!(Keyword::For),
    "fun" => kw!(Keyword::Fun),
    "if" => kw!(Keyword::If),
    "or" => kw!(Keyword::Or),
    "break" => kw!(Keyword::Break),
    "continue" => kw!(Keyword::Continue),
    "return" => kw!(Keyword::Return),
    "super" => kw!(Keyword::Super),
    "this" => kw!(Keyword::This),
    "var" => kw!(Keyword::Var),
    "while" => kw!(Keyword::While),
    "loop" => kw!(Keyword::Loop),
    //
    "null" =>  TokenType::Null,
    "false" => TokenType::False,
    "true" => TokenType::True,
};

pub static CASE_INS_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    //
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
    "NULL" => skw!(SqlKeyword::Null),
    "OFFSET" => skw!(SqlKeyword::Offset),
    "LIMIT" => skw!(SqlKeyword::Limit),
    "LIKE" => skw!(SqlKeyword::Like),
    "JOIN" => skw!(SqlKeyword::Join),
    "INNER" => skw!(SqlKeyword::Inner),
    "OUTER" => skw!(SqlKeyword::Outer),
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
    "TABLE" => skw!(SqlKeyword::Table),
    "UNIQUE" => skw!(SqlKeyword::Unique),
    "READ" => skw!(SqlKeyword::Read),
    "WRITE" => skw!(SqlKeyword::Write),
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub tok_type: TokenType,
    pub lexeme: Option<Rc<String>>,
    pub literal: Option<RV>,
    pub line: u32
}