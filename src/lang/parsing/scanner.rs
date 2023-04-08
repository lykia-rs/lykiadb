use crate::lang::parsing::token::{Equality, Helper, KEYWORDS, Operator, Token, TokenType};
use crate::lang::parsing::token::LiteralValue::{Num, Str};

pub struct Scanner<'a> {
    chars: Vec<char>,
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32
}

impl<'a> Scanner<'a> {
    pub fn scan(source: &'a str) -> Vec<Token> {
        let mut scanner = Scanner {
            chars: source.chars().collect(),
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0
        };
        scanner.scan_tokens();
        scanner.tokens
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.chars[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self, offset: usize) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.chars[self.current + offset]
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        c
    }

    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token {
            tok_type: token,
            lexeme: None,
            literal: None,
            line: self.line
        });
    }

    fn add_str_literal(&mut self, value: String) {
        self.tokens.push(Token {
            tok_type: TokenType::Str,
            lexeme: Some(value.clone()),
            literal: Some(Str(value)),
            line: self.line
        });
    }

    fn add_num_literal(&mut self, value: String) {
        self.tokens.push(Token {
            tok_type: TokenType::Num,
            lexeme: Some(value.clone()),
            literal: Some(Num(value.parse::<f32>().unwrap())),
            line: self.line
        });
    }

    fn add_identifier(&mut self, value: String) {
        self.tokens.push(Token {
            tok_type: TokenType::Identifier,
            lexeme: Some(value.clone()),
            literal: Some(Str(value)),
            line: self.line
        });
    }

    fn add_double_token(&mut self, expected_second: char, token_single: TokenType, token_double: TokenType) {
        let tok_type = if self.match_next(expected_second) {
            token_double
        } else {
            token_single
        };
        self.add_token(tok_type);
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn string(&mut self) {
        while self.peek(0) != '"' && !self.is_at_end() {
            if self.peek(0) == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            panic!("Unterminated string.");
        }

        self.advance();

        let span = self.chars[self.start + 1..self.current -1].iter().collect();
        self.add_str_literal(span);
    }

    fn number(&mut self) {
        while self.peek(0).is_ascii_digit() { self.advance(); }

        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance();
            while self.peek(0).is_ascii_digit() { self.advance(); }
        }

        if self.peek(0).to_ascii_lowercase() == 'e' {
            self.advance();
            if self.peek(0) == '-' || self.peek(0) == '+' {
                self.advance();
            }
            if self.is_at_end() || !self.peek(0).is_ascii_digit() {
                panic!("Malformed number.");
            }
            while self.peek(0).is_ascii_digit() { self.advance(); }
        }

        let span = self.chars[self.start..self.current].iter().collect();
        self.add_num_literal(span);
    }

    fn identifier(&mut self) {
        while self.peek(0).is_alphanumeric() {
            self.advance();
        }
        let span: String = self.chars[self.start..self.current].iter().collect();
        if KEYWORDS.contains_key(&span) {
            self.add_token(KEYWORDS.get(&span).unwrap().clone());
        } else {
            self.add_identifier(span);
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::Helper(Helper::LeftParen)),
            ')' => self.add_token(TokenType::Helper(Helper::RightParen)),
            '{' => self.add_token(TokenType::Helper(Helper::LeftBrace)),
            '}' => self.add_token(TokenType::Helper(Helper::RightBrace)),
            ',' => self.add_token(TokenType::Helper(Helper::Comma)),
            '.' => self.add_token(TokenType::Operator(Operator::Dot)),
            '-' => self.add_token(TokenType::Operator(Operator::Minus)),
            '+' => self.add_token(TokenType::Operator(Operator::Plus)),
            ';' => self.add_token(TokenType::Helper(Helper::Semicolon)),
            '*' => self.add_token(TokenType::Operator(Operator::Star)),
            '!' => self.add_double_token('=', TokenType::Operator(Operator::Bang), TokenType::Equality(Equality::BangEqual)),
            '=' => self.add_double_token('=', TokenType::Equality(Equality::Equal), TokenType::Equality(Equality::EqualEqual)),
            '<' => self.add_double_token('=', TokenType::Equality(Equality::Less), TokenType::Equality(Equality::LessEqual)),
            '>' => self.add_double_token('=', TokenType::Equality(Equality::Greater), TokenType::Equality(Equality::GreaterEqual)),
            '/' => {
                if self.match_next('/') {
                    while !self.is_at_end() && self.peek(0) != '\n' {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Operator(Operator::Slash));
                }
            },
            ' ' => (),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => self.number(),
            'A'..='z' => self.identifier(),
            _ => panic!("Unexpected character."),
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(TokenType::EOF);
    }
}
