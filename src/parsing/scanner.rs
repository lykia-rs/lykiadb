use crate::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
}

impl<'a> Scanner<'a> {
    fn addToken(&mut self, token: TokenType) {
        self.tokens.push(Token {
            tok_type: token,
            lexeme: "",
            literal: "",
            line:2
        });
    }

    pub fn scanTokens(&mut self) {
        let mut start: u32 = 0;
        let mut current: usize = 0;
        let mut line: u32 = 0;
        let chars: Vec<_> = self.source.chars().collect();
        for c in chars {
            match c {
                '(' => self.addToken(TokenType::LeftParen),
                ')' => self.addToken(TokenType::RightParen),
                _ => (),
            }
        }
    }
}