use phf::phf_map;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Keyword {
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator{
    Bang,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Equality{
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Helper {
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(String),
    Num(f32),
    Bool(bool),
    Nil
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
    Helper(Helper),
    Equality(Equality),
    Operator(Operator),
    Keyword(Keyword),
    Literal,
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
    "and" => TokenType::Keyword(Keyword::And),
    "class" => TokenType::Keyword(Keyword::Class),
    "else" => TokenType::Keyword(Keyword::Else),
    "for" => TokenType::Keyword(Keyword::For),
    "fun" => TokenType::Keyword(Keyword::Fun),
    "if" => TokenType::Keyword(Keyword::If),
    "or" => TokenType::Keyword(Keyword::Or),
    "print" => TokenType::Keyword(Keyword::Print),
    "return" => TokenType::Keyword(Keyword::Return),
    "super" => TokenType::Keyword(Keyword::Super),
    "this" => TokenType::Keyword(Keyword::This),
    "var" => TokenType::Keyword(Keyword::Var),
    "while" => TokenType::Keyword(Keyword::While),
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

impl Token {
    fn to_string(&self) -> String {
        format!("{:?}, {:?}, {:?}", self.tok_type, self.lexeme, self.literal)
    }
}
