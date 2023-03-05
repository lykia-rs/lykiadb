use crate::query::parsing::token::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a String,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(raw: &'a String) -> Self {
        Scanner {
            source: raw,
            tokens: vec![]
        }
    }

    fn addToken(&mut self, token: TokenType) {
        let s = String::from("");
        self.tokens.push(Token {
            tok_type: token,
            lexeme: s.clone(),
            literal: s,
            line: 1
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


