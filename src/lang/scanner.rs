use std::rc::Rc;
use crate::lang::token::*;
use crate::lang::token::Symbol::*;
use crate::lang::token::TokenType::{Eof, Identifier};
use crate::runtime::types::RV::*;
use crate::sym;

pub struct Scanner {
    chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32
}

#[derive(Debug)]
pub enum ScanError {
    UnexpectedCharacter { line: u32, chr: char },
    UnterminatedString { line: u32, string: String },
    MalformedNumberLiteral { line: u32, string: String }
}

impl Scanner {
    pub fn scan(source: &str) -> Result<Vec<Token>, ScanError> {
        let mut scanner = Scanner {
            chars: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0
        };
        scanner.scan_tokens()?;
        Ok(scanner.tokens)
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

    fn string(&mut self) -> Result<(), ScanError> {
        while self.peek(0) != '"' && !self.is_at_end() {
            if self.peek(0) == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            let err_span: String = self.chars[self.start + 1..self.current -1].iter().collect();
            return Err(ScanError::UnterminatedString {line: self.line, string: err_span});
        }

        self.advance();

        let span: String = self.chars[self.start + 1..self.current -1].iter().collect();
        self.add_str_literal(&span);
        Ok(())
    }

    fn number(&mut self) -> Result<(), ScanError> {
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
                return Err(ScanError::MalformedNumberLiteral {line: self.line, string: err_span});
            }
            while self.peek(0).is_ascii_digit() { self.advance(); }
        }

        let span: String = self.chars[self.start..self.current].iter().collect();
        self.add_num_literal(&span);
        Ok(())
    }

    fn identifier(&mut self, is_dollar: bool) {
        while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == '_' {
            self.advance();
        }
        let span: String = self.chars[self.start..self.current].iter().collect();
        if CASE_SNS_KEYWORDS.contains_key(&span) {
            self.add_token(&span, CASE_SNS_KEYWORDS.get(&span).unwrap().clone());
        } else if CASE_INS_KEYWORDS.contains_key(&span.to_ascii_uppercase()) {
            self.add_token(&span,CASE_INS_KEYWORDS.get(&span.to_ascii_uppercase()).unwrap().clone());
        } else {
            self.tokens.push(Token {
                tok_type: Identifier { dollar: is_dollar },
                lexeme: Some(Rc::new(span.to_string())),
                literal: Some(Str(Rc::new(span.to_string()))),
                line: self.line
            });
        }
    }

    fn scan_token(&mut self) -> Result<(), ScanError> {
        let c = self.advance();
        match c {
            '\n' => self.line += 1,
            ' ' | '\r' | '\t' => (),
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            'A'..='z' => self.identifier(false),
            '$' => self.identifier(true),
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
            other => {
                if let Some(sym) = SYMBOLS.get(&other) {
                    self.add_token(&other.to_string(), sym.clone());
                } else {
                    return Err(ScanError::UnexpectedCharacter {line: self.line, chr: c})
                }
            }
        }
        Ok(())
    }

    fn scan_tokens(&mut self) -> Result<(), ScanError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.add_token(" ", Eof);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{kw, lexm, skw};
    use crate::lang::token::TokenType::Eof;

    use super::*;

    fn assert_tokens(source: &str, expected_tokens: Vec<Token>) {
        let tokens = Scanner::scan(source).unwrap();
        assert_eq!(tokens.len(), expected_tokens.len());
        // assert that the elements are equal
        for (token, expected) in tokens.iter().zip(expected_tokens.iter()) {
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_single_char_tokens() {
        assert_tokens("(){};,+-*/.", vec![
            Token {tok_type: sym!(LeftParen), lexeme: lexm!("("), literal: None, line: 0},
            Token {tok_type: sym!(RightParen), lexeme: lexm!(")"), literal: None, line: 0},
            Token {tok_type: sym!(LeftBrace), lexeme: lexm!("{"), literal: None, line: 0},
            Token {tok_type: sym!(RightBrace), lexeme: lexm!("}"), literal: None, line: 0},
            Token {tok_type: sym!(Semicolon), lexeme: lexm!(";"), literal: None, line: 0},
            Token {tok_type: sym!(Comma), lexeme: lexm!(","), literal: None, line: 0},
            Token {tok_type: sym!(Plus), lexeme: lexm!("+"), literal: None, line: 0},
            Token {tok_type: sym!(Minus), lexeme: lexm!("-"), literal: None, line: 0},
            Token {tok_type: sym!(Star), lexeme: lexm!("*"), literal: None, line: 0},
            Token {tok_type: sym!(Slash), lexeme: lexm!("/"), literal: None, line: 0},
            Token {tok_type: sym!(Dot), lexeme: lexm!("."), literal: None, line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }

    #[test]
    fn test_mixed_arbitrary() {
        assert_tokens("123 123.456 \"hello world\" true false helloIdentifier", vec![
            Token {tok_type: TokenType::Num, lexeme: lexm!("123"), literal: Some(Num(123.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("123.456"), literal: Some(Num(123.456)), line: 0},
            Token {tok_type: TokenType::Str, lexeme: lexm!("hello world"), literal: Some(Str(Rc::new("hello world".to_string()))), line: 0},
            Token {tok_type: TokenType::True, lexeme: lexm!("true"), literal: None, line: 0},
            Token {tok_type: TokenType::False, lexeme: lexm!("false"), literal: None, line: 0},
            Token {tok_type: TokenType::Identifier { dollar: false }, lexeme: lexm!("helloIdentifier"), literal: Some(Str(Rc::new("helloIdentifier".to_string()))), line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }
    #[test]
    fn test_identifiers() {
        assert_tokens("$myPreciseVariable $my_precise_variable myPreciseFunction my_precise_function", vec![
            Token {tok_type: TokenType::Identifier { dollar: true }, lexeme: lexm!("$myPreciseVariable"), literal: Some(Str(Rc::new("$myPreciseVariable".to_string()))), line: 0},
            Token {tok_type: TokenType::Identifier { dollar: true }, lexeme: lexm!("$my_precise_variable"), literal: Some(Str(Rc::new("$my_precise_variable".to_string()))), line: 0},
            Token {tok_type: TokenType::Identifier { dollar: false }, lexeme: lexm!("myPreciseFunction"), literal: Some(Str(Rc::new("myPreciseFunction".to_string()))), line: 0},
            Token {tok_type: TokenType::Identifier { dollar: false }, lexeme: lexm!("my_precise_function"), literal: Some(Str(Rc::new("my_precise_function".to_string()))), line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }

    #[test]
    fn test_number_literals() {
        assert_tokens("0 1 2 3 4 5 6 7 8 9 10 100 500 1000 1.7976931348623157E+308 1.7976931348623157E-308", vec![
            Token {tok_type: TokenType::Num, lexeme: lexm!("0"), literal: Some(Num(0.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("1"), literal: Some(Num(1.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("2"), literal: Some(Num(2.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("3"), literal: Some(Num(3.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("4"), literal: Some(Num(4.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("5"), literal: Some(Num(5.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("6"), literal: Some(Num(6.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("7"), literal: Some(Num(7.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("8"), literal: Some(Num(8.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("9"), literal: Some(Num(9.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("10"), literal: Some(Num(10.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("100"), literal: Some(Num(100.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("500"), literal: Some(Num(500.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("1000"), literal: Some(Num(1000.0)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("1.7976931348623157E+308"), literal: Some(Num(1.7976931348623157E+308)), line: 0},
            Token {tok_type: TokenType::Num, lexeme: lexm!("1.7976931348623157E-308"), literal: Some(Num(1.7976931348623157E-308)), line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }

    #[test]
    fn test_keywords() {
        assert_tokens("and or class else for fun if break continue return super this var while loop", vec![
            Token {tok_type: kw!(Keyword::And), lexeme: lexm!("and"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Or), lexeme: lexm!("or"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Class), lexeme: lexm!("class"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Else), lexeme: lexm!("else"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::For), lexeme: lexm!("for"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Fun), lexeme: lexm!("fun"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::If), lexeme: lexm!("if"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Break), lexeme: lexm!("break"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Continue), lexeme: lexm!("continue"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Return), lexeme: lexm!("return"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Super), lexeme: lexm!("super"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::This), lexeme: lexm!("this"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Var), lexeme: lexm!("var"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::While), lexeme: lexm!("while"), literal: None, line: 0},
            Token {tok_type: kw!(Keyword::Loop), lexeme: lexm!("loop"), literal: None, line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }

    #[test]
    fn test_sql_keywords() {
        assert_tokens(
            "Begin Transaction Rollback Commit Where Having Asc Desc Order By Explain Is Not Null Offset Like Limit And Or Join Inner Outer Right Left On Create Insert Update Delete Drop Into Values Index Table Select From As Cross Default Group Key Of Only Primary References Set System Unique Read Write", vec![
            Token {tok_type: skw!(SqlKeyword::Begin), lexeme: lexm!("Begin"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Transaction), lexeme: lexm!("Transaction"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Rollback), lexeme: lexm!("Rollback"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Commit), lexeme: lexm!("Commit"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Where), lexeme: lexm!("Where"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Having), lexeme: lexm!("Having"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Asc), lexeme: lexm!("Asc"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Desc), lexeme: lexm!("Desc"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Order), lexeme: lexm!("Order"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::By), lexeme: lexm!("By"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Explain), lexeme: lexm!("Explain"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Is), lexeme: lexm!("Is"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Not), lexeme: lexm!("Not"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Null), lexeme: lexm!("Null"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Offset), lexeme: lexm!("Offset"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Like), lexeme: lexm!("Like"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Limit), lexeme: lexm!("Limit"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::And), lexeme: lexm!("And"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Or), lexeme: lexm!("Or"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Join), lexeme: lexm!("Join"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Inner), lexeme: lexm!("Inner"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Outer), lexeme: lexm!("Outer"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Right), lexeme: lexm!("Right"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Left), lexeme: lexm!("Left"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::On), lexeme: lexm!("On"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Create), lexeme: lexm!("Create"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Insert), lexeme: lexm!("Insert"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Update), lexeme: lexm!("Update"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Delete), lexeme: lexm!("Delete"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Drop), lexeme: lexm!("Drop"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Into), lexeme: lexm!("Into"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Values), lexeme: lexm!("Values"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Index), lexeme: lexm!("Index"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Table), lexeme: lexm!("Table"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Select), lexeme: lexm!("Select"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::From), lexeme: lexm!("From"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::As), lexeme: lexm!("As"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Cross), lexeme: lexm!("Cross"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Default), lexeme: lexm!("Default"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Group), lexeme: lexm!("Group"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Key), lexeme: lexm!("Key"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Of), lexeme: lexm!("Of"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Only), lexeme: lexm!("Only"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Primary), lexeme: lexm!("Primary"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::References), lexeme: lexm!("References"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Set), lexeme: lexm!("Set"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::System), lexeme: lexm!("System"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Unique), lexeme: lexm!("Unique"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Read), lexeme: lexm!("Read"), literal: None, line: 0},
            Token {tok_type: skw!(SqlKeyword::Write), lexeme: lexm!("Write"), literal: None, line: 0},
            Token {tok_type: Eof, lexeme: lexm!(" "), literal: None, line: 0}
        ]);
    }
}