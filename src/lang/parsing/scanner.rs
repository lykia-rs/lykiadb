use std::rc::Rc;
use crate::lang::parsing::error::scan_err;
use crate::lang::parsing::token::*;
use crate::lang::parsing::token::LiteralValue::{Num, Str};
use crate::lang::parsing::token::Symbol::*;
use crate::lang::parsing::token::TokenType::*;
use crate::sym;

pub struct Scanner {
    chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32
}

impl Scanner {
    pub fn scan(source: &str) -> Vec<Token> {
        let mut scanner = Scanner {
            chars: source.chars().collect(),
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

    fn add_token(&mut self, lexeme: &str, token: TokenType) {
        self.tokens.push(Token {
            tok_type: token,
            lexeme: Some(Rc::new(lexeme.to_string())),
            literal: None,
            line: self.line
        });
    }

    fn add_str_literal(&mut self, value: &str) {
        self.tokens.push(Token {
            tok_type: TokenType::Str,
            lexeme: Some(Rc::new(value.to_string())),
            literal: Some(Str(Rc::new(value.to_string()))),
            line: self.line
        });
    }

    fn add_num_literal(&mut self, value: &str) {
        self.tokens.push(Token {
            tok_type: TokenType::Num,
            lexeme: Some(Rc::new(value.to_string())),
            literal: Some(Num(value.parse::<f64>().unwrap())),
            line: self.line
        });
    }

    fn add_identifier(&mut self, value: &str) {
        self.tokens.push(Token {
            tok_type: Identifier,
            lexeme: Some(Rc::new(value.to_string())),
            literal: Some(Str(Rc::new(value.to_string()))),
            line: self.line
        });
    }

    fn add_double_token(&mut self, lexeme_prefix: &str, expected_second: char, token_single: TokenType, token_double: TokenType) {
        if self.match_next(expected_second) {
            let concat = lexeme_prefix.to_string() + &expected_second.to_string();
            self.add_token(&concat, token_double);
        } else {
            self.add_token(lexeme_prefix, token_single);
        };
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
            let err_span: String = self.chars[self.start + 1..self.current -1].iter().collect();
            scan_err(&format!("Unterminated string '{}'", err_span), self.line);
        }

        self.advance();

        let span: String = self.chars[self.start + 1..self.current -1].iter().collect();
        self.add_str_literal(&span);
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
                let err_span: String = self.chars[self.start..self.current].iter().collect();
                scan_err(&format!("Malformed number literal '{}'", err_span), self.line);
            }
            while self.peek(0).is_ascii_digit() { self.advance(); }
        }

        let span: String = self.chars[self.start..self.current].iter().collect();
        self.add_num_literal(&span);
    }

    fn identifier(&mut self, is_safe: bool) {
        while self.peek(0).is_alphanumeric() {
            self.advance();
        }
        let span: String = self.chars[self.start..self.current].iter().collect();
        if CASE_SNS_KEYWORDS.contains_key(&span) {
            self.add_token(&span, CASE_SNS_KEYWORDS.get(&span).unwrap().clone());
        } else if CASE_INS_KEYWORDS.contains_key(&span.to_ascii_uppercase()) {
            self.add_token(&span,CASE_INS_KEYWORDS.get(&span.to_ascii_uppercase()).unwrap().clone());
        } else {
            self.add_identifier(&span);
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(&c.to_string(), sym!(LeftParen)),
            ')' => self.add_token(&c.to_string(),sym!(RightParen)),
            '{' => self.add_token(&c.to_string(),sym!(LeftBrace)),
            '}' => self.add_token(&c.to_string(),sym!(RightBrace)),
            ',' => self.add_token(&c.to_string(),sym!(Comma)),
            '.' => self.add_token(&c.to_string(),sym!(Dot)),
            '-' => self.add_token(&c.to_string(),sym!(Minus)),
            '+' => self.add_token(&c.to_string(),sym!(Plus)),
            ';' => self.add_token(&c.to_string(),sym!(Semicolon)),
            '*' => self.add_token(&c.to_string(),sym!(Star)),
            '!' => self.add_double_token(&c.to_string(),'=', sym!(Bang), sym!(BangEqual)),
            '=' => self.add_double_token(&c.to_string(),'=', sym!(Equal), sym!(EqualEqual)),
            '<' => self.add_double_token(&c.to_string(),'=', sym!(Less), sym!(LessEqual)),
            '>' => self.add_double_token(&c.to_string(),'=', sym!(Greater), sym!(GreaterEqual)),
            '/' => {
                if self.match_next('/') {
                    while !self.is_at_end() && self.peek(0) != '\n' {
                        self.advance();
                    }
                } else {
                    self.add_token(&c.to_string(),sym!(Slash));
                }
            },
            ' ' => (),
            '\r' => (),
            '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => self.number(),
            'A'..='z' => self.identifier(false),
            '$' => self.identifier(true),
            _ => scan_err(&format!("Unexpected character '{}'", c), self.line),
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(" ", Eof);
    }
}
