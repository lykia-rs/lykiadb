use std::rc::Rc;
use phf::phf_map;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(Rc<String>),
    Num(f64),
    Bool(bool),
    Nil
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
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
    And,
    Class,
    Else,
    Fun,
    If,
    Or,
    Print,
    Clock,
    Break,
    Continue,
    Return,
    Super,
    This,
    Var,
    While,
    For,
    Loop,
    Str,
    Num,
    Identifier,
    Nil,
    False,
    True,
    //
    Eof
}

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "clock" => TokenType::Clock,
    "break" => TokenType::Break,
    "continue" => TokenType::Continue,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "var" => TokenType::Var,
    "while" => TokenType::While,
    "loop" => TokenType::Loop,
    //
    "nil" =>  TokenType::Nil,
    "false" => TokenType::False,
    "true" => TokenType::True,
};

#[derive(Debug, Clone)]
pub struct Token {
    pub tok_type: TokenType,
    pub lexeme: Option<Rc<String>>,
    pub literal: Option<LiteralValue>,
    pub line: u32
}