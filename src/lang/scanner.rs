use crate::lang::token::Symbol::*;
use crate::lang::token::TokenType::{Eof, Identifier};
use crate::lang::token::*;
use crate::runtime::types::RV::*;
use crate::sym;
use std::rc::Rc;

pub struct Scanner {
    chars: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

#[derive(Debug, Clone)]
pub enum ScanError {
    UnexpectedCharacter { span: Span },
    UnterminatedString { span: Span },
    MalformedNumberLiteral { span: Span },
}

impl Scanner {
    pub fn scan(source: &str) -> Result<Vec<Token>, ScanError> {
        let mut scanner = Scanner {
            chars: source.chars().collect(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 0,
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
            literal: None,
            lexeme: Some(lexeme.to_string()),
            span: Span {
                start: self.start,
                end: self.current - 1,
                line: self.line,
                line_end: self.line,
            },
        });
    }

    fn finalize(&mut self) {
        self.tokens.push(Token {
            tok_type: Eof,
            literal: None,
            lexeme: None,
            span: Span {
                start: self.current,
                end: self.current,
                line: self.line,
                line_end: self.line,
            },
        });
    }

    fn add_double_token(
        &mut self,
        lexeme_prefix: &str,
        expected_second: char,
        token_single: TokenType,
        token_double: TokenType,
    ) {
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
            return Err(ScanError::UnterminatedString {
                span: Span {
                    start: self.start + 1,
                    end: self.current,
                    line: self.line,
                    line_end: self.line,
                },
            });
        }

        self.advance();

        let span: String = self.chars[self.start + 1..self.current - 1]
            .iter()
            .collect();

        self.tokens.push(Token {
            tok_type: TokenType::Str,
            literal: Some(Str(Rc::new(span.clone()))),
            lexeme: Some(span),
            span: Span {
                start: self.start + 1,
                end: self.current - 1,
                line: self.line,
                line_end: self.line,
            },
        });
        Ok(())
    }

    fn number(&mut self) -> Result<(), ScanError> {
        while self.peek(0).is_ascii_digit() {
            self.advance();
        }

        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance();
            while self.peek(0).is_ascii_digit() {
                self.advance();
            }
        }

        if self.peek(0).to_ascii_lowercase() == 'e' {
            self.advance();
            if self.peek(0) == '-' || self.peek(0) == '+' {
                self.advance();
            }
            if self.is_at_end() || !self.peek(0).is_ascii_digit() {
                return Err(ScanError::MalformedNumberLiteral {
                    span: Span {
                        start: self.start,
                        end: self.current,
                        line: self.line,
                        line_end: self.line,
                    },
                });
            }
            while self.peek(0).is_ascii_digit() {
                self.advance();
            }
        }

        let span: String = self.chars[self.start..self.current].iter().collect();
        let parsed = span.parse::<f64>().unwrap();
        self.tokens.push(Token {
            tok_type: TokenType::Num,
            literal: Some(Num(parsed)),
            lexeme: Some(span),
            span: Span {
                start: self.start,
                end: self.current,
                line: self.line,
                line_end: self.line,
            },
        });
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
            self.add_token(
                &span,
                CASE_INS_KEYWORDS
                    .get(&span.to_ascii_uppercase())
                    .unwrap()
                    .clone(),
            );
        } else {
            let value = span.to_string();
            self.tokens.push(Token {
                tok_type: Identifier { dollar: is_dollar },
                literal: Some(Str(Rc::new(value.clone()))),
                lexeme: Some(value),
                span: Span {
                    start: self.start,
                    end: self.current,
                    line: self.line,
                    line_end: self.line,
                },
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
            '!' => self.add_double_token(&c.to_string(), '=', sym!(Bang), sym!(BangEqual)),
            '=' => self.add_double_token(&c.to_string(), '=', sym!(Equal), sym!(EqualEqual)),
            '<' => self.add_double_token(&c.to_string(), '=', sym!(Less), sym!(LessEqual)),
            '>' => self.add_double_token(&c.to_string(), '=', sym!(Greater), sym!(GreaterEqual)),
            '/' => {
                if self.match_next('/') {
                    while !self.is_at_end() && self.peek(0) != '\n' {
                        self.advance();
                    }
                } else {
                    self.add_token(&c.to_string(), sym!(Slash));
                }
            }
            other => {
                if let Some(sym) = SYMBOLS.get(&other) {
                    self.add_token(&other.to_string(), sym.clone());
                } else {
                    return Err(ScanError::UnexpectedCharacter {
                        span: Span {
                            start: self.start,
                            end: self.current,
                            line: self.line,
                            line_end: self.line,
                        },
                    });
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

        self.finalize();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::lang::token::TokenType::Eof;
    use crate::{kw, lexm, skw};

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
        assert_tokens(
            "(){};,+-*/.",
            vec![
                Token {
                    tok_type: sym!(LeftParen),
                    lexeme: lexm!("("),
                    literal: None,
                    span: Span {
                        start: 0,
                        end: 0,
                        line: 0,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(RightParen),
                    lexeme: lexm!(")"),
                    literal: None,
                    span: Span {
                        start: 1,
                        end: 1,
                        line: 0,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(LeftBrace),
                    lexeme: lexm!("{"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 2,
                        end: 2,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(RightBrace),
                    lexeme: lexm!("}"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 3,
                        end: 3,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Semicolon),
                    lexeme: lexm!(";"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 4,
                        end: 4,
                        line_end: 0,
                    },
                },
                Token {
                    lexeme: lexm!(","),
                    tok_type: sym!(Comma),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 5,
                        end: 5,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Plus),
                    lexeme: lexm!("+"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 6,
                        end: 6,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Minus),
                    lexeme: lexm!("-"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 7,
                        end: 7,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Star),
                    lexeme: lexm!("*"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 8,
                        end: 8,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Slash),
                    lexeme: lexm!("/"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 9,
                        end: 9,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Dot),
                    lexeme: lexm!("."),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 10,
                        end: 10,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 11,
                        end: 11,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_mixed_arbitrary() {
        assert_tokens(
            "123 123.456 \"hello world\" true false helloIdentifier",
            vec![
                Token {
                    tok_type: TokenType::Num,
                    literal: Some(Num(123.0)),
                    lexeme: lexm!("123"),
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 3,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    literal: Some(Num(123.456)),
                    lexeme: lexm!("123.456"),
                    span: Span {
                        line: 0,
                        start: 4,
                        end: 11,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    literal: Some(Str(Rc::new("hello world".to_string()))),
                    lexeme: lexm!("hello world"),
                    span: Span {
                        line: 0,
                        start: 13,
                        end: 24,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::True,
                    literal: None,
                    lexeme: lexm!("true"),
                    span: Span {
                        line: 0,
                        start: 26,
                        end: 29,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::False,
                    literal: None,
                    lexeme: lexm!("false"),
                    span: Span {
                        line: 0,
                        start: 31,
                        end: 35,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("helloIdentifier".to_string()))),
                    lexeme: lexm!("helloIdentifier"),
                    span: Span {
                        line: 0,
                        start: 37,
                        end: 52,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    literal: None,
                    lexeme: None,
                    span: Span {
                        line: 0,
                        start: 53,
                        end: 53,
                        line_end: 0,
                    },
                },
            ],
        );
    }
    #[test]
    fn test_identifiers() {
        assert_tokens(
            "$myPreciseVariable $my_precise_variable myPreciseFunction my_precise_function",
            vec![
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Rc::new("$myPreciseVariable".to_string()))),
                    lexeme: lexm!("$myPreciseVariable"),
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 18,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Rc::new("$my_precise_variable".to_string()))),
                    lexeme: lexm!("$my_precise_variable"),
                    span: Span {
                        line: 0,
                        start: 19,
                        end: 39,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("myPreciseFunction".to_string()))),
                    lexeme: lexm!("myPreciseFunction"),
                    span: Span {
                        line: 0,
                        start: 40,
                        end: 57,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("my_precise_function".to_string()))),
                    lexeme: lexm!("my_precise_function"),
                    span: Span {
                        line: 0,
                        start: 58,
                        end: 77,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    literal: None,
                    lexeme: None,
                    span: Span {
                        start: 78,
                        end: 78,
                        line: 0,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_number_literals() {
        assert_tokens(
            "0 1 2 3 4 5 6 7 8 9 10 100 500 1000 1.7976931348623157E+308 1.7976931348623157E-308",
            vec![
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("0"),
                    literal: Some(Num(0.0)),
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 1,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("1"),
                    literal: Some(Num(1.0)),
                    span: Span {
                        line: 0,
                        start: 2,
                        end: 3,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("2"),
                    literal: Some(Num(2.0)),
                    span: Span {
                        line: 0,
                        start: 4,
                        end: 5,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("3"),
                    literal: Some(Num(3.0)),
                    span: Span {
                        line: 0,
                        start: 6,
                        end: 7,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("4"),
                    literal: Some(Num(4.0)),
                    span: Span {
                        line: 0,
                        start: 8,
                        end: 9,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("5"),
                    literal: Some(Num(5.0)),
                    span: Span {
                        line: 0,
                        start: 10,
                        end: 11,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("6"),
                    literal: Some(Num(6.0)),
                    span: Span {
                        line: 0,
                        start: 12,
                        end: 13,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("7"),
                    literal: Some(Num(7.0)),
                    span: Span {
                        line: 0,
                        start: 14,
                        end: 15,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("8"),
                    literal: Some(Num(8.0)),
                    span: Span {
                        line: 0,
                        start: 16,
                        end: 17,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("9"),
                    literal: Some(Num(9.0)),
                    span: Span {
                        line: 0,
                        start: 18,
                        end: 19,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("10"),
                    literal: Some(Num(10.0)),
                    span: Span {
                        line: 0,
                        start: 20,
                        end: 22,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("100"),
                    literal: Some(Num(100.0)),
                    span: Span {
                        line: 0,
                        start: 23,
                        end: 26,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("500"),
                    literal: Some(Num(500.0)),
                    span: Span {
                        line: 0,
                        start: 27,
                        end: 30,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("1000"),
                    literal: Some(Num(1000.0)),
                    span: Span {
                        line: 0,
                        start: 31,
                        end: 35,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("1.7976931348623157E+308"),
                    literal: Some(Num(1.7976931348623157E+308)),
                    span: Span {
                        line: 0,
                        start: 36,
                        end: 59,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    lexeme: lexm!("1.7976931348623157E-308"),
                    literal: Some(Num(1.7976931348623157E-308)),
                    span: Span {
                        line: 0,
                        start: 60,
                        end: 83,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 84,
                        end: 84,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_keywords() {
        assert_tokens(
            "and or class else for fun if break continue return super this var while loop",
            vec![
                Token {
                    tok_type: kw!(Keyword::And),
                    lexeme: lexm!("and"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 2,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Or),
                    lexeme: lexm!("or"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 4,
                        end: 5,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Class),
                    lexeme: lexm!("class"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 7,
                        end: 11,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Else),
                    lexeme: lexm!("else"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 13,
                        end: 16,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::For),
                    lexeme: lexm!("for"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 18,
                        end: 20,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Fun),
                    lexeme: lexm!("fun"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 22,
                        end: 24,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::If),
                    lexeme: lexm!("if"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 26,
                        end: 27,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Break),
                    lexeme: lexm!("break"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 29,
                        end: 33,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Continue),
                    lexeme: lexm!("continue"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 35,
                        end: 42,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Return),
                    lexeme: lexm!("return"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 44,
                        end: 49,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Super),
                    lexeme: lexm!("super"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 51,
                        end: 55,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::This),
                    lexeme: lexm!("this"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 57,
                        end: 60,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Var),
                    lexeme: lexm!("var"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 62,
                        end: 64,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::While),
                    lexeme: lexm!("while"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 66,
                        end: 70,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Loop),
                    lexeme: lexm!("loop"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 72,
                        end: 75,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 76,
                        end: 76,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_sql_keywords() {
        assert_tokens(
            "Begin Transaction Rollback Commit Where Having Asc Desc Order By Explain Is Not Null Offset Like Limit And Or Join Inner Outer Right Left On Create Insert Update Delete Drop Into Values Index Table Select From As Cross Default Group Key Of Only Primary References Set System Unique Read Write", vec![
            Token {tok_type: skw!(SqlKeyword::Begin), literal: None, lexeme: lexm!("Begin"), span: Span { line: 0, start: 0, line_end: 0, end: 4 }},
            Token {tok_type: skw!(SqlKeyword::Transaction), literal: None, lexeme: lexm!("Transaction") , span: Span { line: 0, start: 6, line_end: 0, end: 16 }},
            Token {tok_type: skw!(SqlKeyword::Rollback), literal: None, lexeme: lexm!("Rollback"), span: Span { line: 0, start: 18, line_end: 0, end: 25 }},
            Token {tok_type: skw!(SqlKeyword::Commit), literal: None , lexeme: lexm!("Commit"), span: Span {  line: 0, start: 27, line_end: 0, end: 32 }},
            Token {tok_type: skw!(SqlKeyword::Where), literal: None, lexeme: lexm!("Where"), span: Span { line: 0, start: 34, line_end: 0, end: 38 }},
            Token {tok_type: skw!(SqlKeyword::Having), literal: None , lexeme: lexm!("Having"), span: Span { line: 0, start: 40, line_end: 0, end: 45 }},
            Token {tok_type: skw!(SqlKeyword::Asc), literal: None, lexeme: lexm!("Asc") , span: Span { line: 0, start: 47, line_end: 0, end: 49 }},
            Token {tok_type: skw!(SqlKeyword::Desc), literal: None, lexeme: lexm!("Desc"), span: Span { line: 0, start: 51, line_end: 0, end: 54 }},
            Token {tok_type: skw!(SqlKeyword::Order), literal: None, lexeme: lexm!("Order"), span: Span { line: 0, start: 56, line_end: 0, end: 60 }},
            Token {tok_type: skw!(SqlKeyword::By), literal: None, lexeme: lexm!("By"), span: Span { line: 0, start: 62, line_end: 0, end: 63 }},
            Token {tok_type: skw!(SqlKeyword::Explain), literal: None, lexeme: lexm!("Explain"), span: Span { line: 0, start: 65, line_end: 0, end: 71 }},
            Token {tok_type: skw!(SqlKeyword::Is), literal: None, lexeme: lexm!("Is"), span: Span { line: 0, start: 73, line_end: 0, end: 74 }},
            Token {tok_type: skw!(SqlKeyword::Not), literal: None, lexeme: lexm!("Not"), span: Span { line: 0, start: 76, line_end: 0, end: 78 }},
            Token {tok_type: skw!(SqlKeyword::Null), literal: None, lexeme: lexm!("Null"), span: Span { line: 0, start: 80, line_end: 0, end: 83 }},
            Token {tok_type: skw!(SqlKeyword::Offset), literal: None, lexeme: lexm!("Offset"), span: Span { line: 0, start: 85, line_end: 0, end: 90 }},
            Token {tok_type: skw!(SqlKeyword::Like), literal: None, lexeme: lexm!("Like"), span: Span { line: 0, start: 92, line_end: 0, end: 95 }},
            Token {tok_type: skw!(SqlKeyword::Limit), literal: None, lexeme: lexm!("Limit"), span: Span { line: 0, start: 97, line_end: 0, end: 101 }},
            Token {tok_type: skw!(SqlKeyword::And), literal: None, lexeme: lexm!("And"), span: Span { line: 0, start: 103, line_end: 0, end: 105 }},
            Token {tok_type: skw!(SqlKeyword::Or), literal: None, lexeme: lexm!("Or"), span: Span { line: 0, start: 107, line_end: 0, end: 108 }},
            Token {tok_type: skw!(SqlKeyword::Join), literal: None, lexeme: lexm!("Join") , span: Span { line: 0, start: 110, line_end: 0, end: 113 }},
            Token {tok_type: skw!(SqlKeyword::Inner), literal: None, lexeme: lexm!("Inner") , span: Span { line: 0, start: 115, line_end: 0, end: 119 }},
            Token {tok_type: skw!(SqlKeyword::Outer), literal: None, lexeme: lexm!("Outer"), span: Span { line: 0, start: 121, line_end: 0, end: 125 }},
            Token {tok_type: skw!(SqlKeyword::Right), literal: None, lexeme: lexm!("Right"), span: Span { line: 0, start: 127, line_end: 0, end: 131 }},
            Token {tok_type: skw!(SqlKeyword::Left), literal: None, lexeme: lexm!("Left"), span: Span { line: 0, start: 133, line_end: 0, end: 136 }},
            Token {tok_type: skw!(SqlKeyword::On), literal: None, lexeme: lexm!("On") , span: Span { line: 0, start: 138, line_end: 0, end: 139 }},
            Token {tok_type: skw!(SqlKeyword::Create), literal: None, lexeme: lexm!("Create"), span: Span { line: 0, start: 141, line_end: 0, end: 146 }},
            Token {tok_type: skw!(SqlKeyword::Insert), literal: None, lexeme: lexm!("Insert"), span: Span { line: 0, start: 148, line_end: 0, end: 153 }},
            Token {tok_type: skw!(SqlKeyword::Update), literal: None, lexeme: lexm!("Update"), span: Span { line: 0, start: 155, line_end: 0, end: 160 }},
            Token {tok_type: skw!(SqlKeyword::Delete), literal: None, lexeme: lexm!("Delete"), span: Span { line: 0, start: 162, line_end: 0, end: 167 }},
            Token {tok_type: skw!(SqlKeyword::Drop), literal: None, lexeme: lexm!("Drop") , span: Span { line: 0, start: 169, line_end: 0, end: 172 }},
            Token {tok_type: skw!(SqlKeyword::Into), literal: None, lexeme: lexm!("Into") , span: Span { line: 0, start: 174, line_end: 0, end: 177 }},
            Token {tok_type: skw!(SqlKeyword::Values), literal: None , lexeme: lexm!("Values"), span: Span { line: 0, start: 179, line_end: 0, end: 184 }},
            Token {tok_type: skw!(SqlKeyword::Index), literal: None, lexeme: lexm!("Index"), span: Span { line: 0, start: 186, line_end: 0, end: 190 }},
            Token {tok_type: skw!(SqlKeyword::Table), literal: None, lexeme: lexm!("Table"), span: Span { line: 0, start: 192, line_end: 0, end: 196 }},
            Token {tok_type: skw!(SqlKeyword::Select), literal: None, lexeme: lexm!("Select"), span: Span { line: 0, start: 198, line_end: 0, end: 203 }},
            Token {tok_type: skw!(SqlKeyword::From), literal: None, lexeme: lexm!("From"), span: Span { line: 0, start: 205, line_end: 0, end: 208 }},
            Token {tok_type: skw!(SqlKeyword::As), literal: None, lexeme: lexm!("As"), span: Span { line: 0, start: 210, line_end: 0, end: 211 }},
            Token {tok_type: skw!(SqlKeyword::Cross), literal: None, lexeme: lexm!("Cross"), span: Span { line: 0, start: 213, line_end: 0, end: 217 }},
            Token {tok_type: skw!(SqlKeyword::Default), literal: None, lexeme: lexm!("Default"), span: Span { line: 0, start: 219, line_end: 0, end: 225 }},
            Token {tok_type: skw!(SqlKeyword::Group), literal: None, lexeme: lexm!("Group"), span: Span { line: 0, start: 227, line_end: 0, end: 231 }},
            Token {tok_type: skw!(SqlKeyword::Key), literal: None, lexeme: lexm!("Key"), span: Span { line: 0, start: 233, line_end: 0, end: 235 }},
            Token {tok_type: skw!(SqlKeyword::Of), literal: None, lexeme: lexm!("Of"), span: Span { line: 0, start: 237, line_end: 0, end: 238 }},
            Token {tok_type: skw!(SqlKeyword::Only), literal: None, lexeme: lexm!("Only"), span: Span { line: 0, start: 240, line_end: 0, end: 243 }},
            Token {tok_type: skw!(SqlKeyword::Primary), literal: None, lexeme: lexm!("Primary"), span: Span { line: 0, start: 245, line_end: 0, end: 251 }},
            Token {tok_type: skw!(SqlKeyword::References), literal: None, lexeme: lexm!("References"), span: Span { line: 0, start: 253, line_end: 0, end: 262 }},
            Token {tok_type: skw!(SqlKeyword::Set), literal: None, lexeme: lexm!("Set"), span: Span { line: 0, start: 264, line_end: 0, end: 266 }},
            Token {tok_type: skw!(SqlKeyword::System), literal: None, lexeme: lexm!("System"), span: Span { line: 0, start: 268, line_end: 0, end: 273 }},
            Token {tok_type: skw!(SqlKeyword::Unique), literal: None, lexeme: lexm!("Unique") , span: Span { line: 0, start: 275, line_end: 0, end: 280 }},
            Token {tok_type: skw!(SqlKeyword::Read), literal: None, lexeme: lexm!("Read"), span: Span { line: 0, start: 282, line_end: 0, end: 285 }},
            Token {tok_type: skw!(SqlKeyword::Write), literal: None, lexeme: lexm!("Write") , span: Span { line: 0, start: 287, line_end: 0, end: 291 }},
            Token {tok_type: Eof, literal: None, lexeme: None, span: Span { line: 0, start: 292, line_end: 0, end: 292 }},
        ]);
    }
}
