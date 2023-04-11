use phf::phf_map;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(String),
    Num(f32),
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
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
    Str,
    Num,
    Identifier,
    Nil,
    False,
    True,
    //
    EOF
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
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "var" => TokenType::Var,
    "while" => TokenType::While,
    //
    "nil" =>  TokenType::Nil,
    "false" => TokenType::False,
    "true" => TokenType::True,
};

#[derive(Debug, Clone)]
pub struct Token {
    pub tok_type: TokenType,
    pub lexeme: Option<String>,
    pub literal: Option<LiteralValue>,
    pub line: u32
}