#[derive(Debug)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    //
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    //
    Identifier,
    String,
    Number,

    //
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    //
    EOF
}

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
