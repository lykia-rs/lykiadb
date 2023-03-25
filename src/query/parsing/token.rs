use phf::phf_map;

#[derive(Debug, Clone)]
pub enum Keyword {
    Identifier,
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
    //
    True,
    False,
    Nil,
}

#[derive(Debug, Clone)]
pub enum Operator{
    Bang,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
}

#[derive(Debug, Clone)]
pub enum Equality{
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone)]
pub enum Helper {
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Helper(Helper),
    Equality(Equality),
    Operator(Operator),
    Keyword(Keyword),
    String,
    Number,
    //
    EOF
}

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::Keyword(Keyword::And),
    "class" => TokenType::Keyword(Keyword::Class),
    "else" => TokenType::Keyword(Keyword::Else),
    "false" => TokenType::Keyword(Keyword::False),
    "for" => TokenType::Keyword(Keyword::For),
    "fun" => TokenType::Keyword(Keyword::Fun),
    "if" => TokenType::Keyword(Keyword::If),
    "nil" =>  TokenType::Keyword(Keyword::Nil),
    "or" => TokenType::Keyword(Keyword::Or),
    "print" => TokenType::Keyword(Keyword::Print),
    "return" => TokenType::Keyword(Keyword::Return),
    "super" => TokenType::Keyword(Keyword::Super),
    "this" => TokenType::Keyword(Keyword::This),
    "true" => TokenType::Keyword(Keyword::True),
    "var" => TokenType::Keyword(Keyword::Var),
    "while" => TokenType::Keyword(Keyword::While)
};

#[derive(Debug)]
pub enum Literal {
    Str(String),
    Num(f32),
}

#[derive(Debug)]
pub struct Token {
    pub tok_type: TokenType,
    pub lexeme: Option<String>,
    pub literal: Option<Literal>,
    pub line: u32
}

impl Token {
    fn to_string(&self) -> String {
        format!("{:?}, {:?}, {:?}", self.tok_type, self.lexeme, self.literal)
    }
}
