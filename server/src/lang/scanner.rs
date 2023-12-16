use crate::lang::ast::Literal::*;
use crate::lang::token::Symbol::*;
use crate::lang::token::TokenType::{Eof, Identifier};
use crate::lang::token::*;
use crate::sym;
use std::iter::{Enumerate, Peekable};
use std::rc::Rc;
use std::str::Chars;

pub struct Scanner<'a> {
    chars: Peekable<Enumerate<Chars<'a>>>,
    line: u32,
}

#[derive(Debug, Clone)]
pub enum ScanError {
    UnexpectedCharacter { span: Span },
    UnterminatedString { span: Span },
    MalformedNumberLiteral { span: Span },
}

impl<'a> Scanner<'a> {
    pub fn scan(source: &str) -> Result<Vec<Token>, ScanError> {
        let mut scanner = Scanner {
            chars: source.chars().enumerate().peekable(),
            line: 0,
        };
        scanner.scan_tokens()
    }

    // TODO(vck): remove this
    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.chars.peek().unwrap().1 != expected {
            return false;
        }
        self.chars.next();
        true
    }

    // TODO(vck): remove this
    fn peek(&self, offset: usize) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.chars.clone().nth(offset).unwrap().1
    }

    fn advance(&mut self) -> (usize, char) {
        self.chars.next().unwrap()
    }

    // TODO(vck): remove this
    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.chars.clone().peek().is_none()
    }

    fn scan_string(&mut self, start: usize) -> Result<Token, ScanError> {
        self.advance(); // consume the opening "
        let mut raw_str = String::new();
        while self.peek(0) != '"' && !self.is_at_end() {
            raw_str.push(self.advance().1);
        }

        if self.is_at_end() {
            return Err(ScanError::UnterminatedString {
                span: Span {
                    start: start,
                    end: start + raw_str.len(),
                    line: self.line,
                    line_end: self.line,
                },
            });
        }

        self.advance();

        let len = raw_str.len();
        Ok(Token {
            tok_type: TokenType::Str,
            literal: Some(Str(Rc::new(raw_str.clone()))),
            lexeme: Some(raw_str),
            span: Span {
                start: start,
                end: start + len + 2,
                line: self.line,
                line_end: self.line,
            },
        })
    }

    fn scan_number(&mut self, start: usize) -> Result<Token, ScanError> {
        let mut raw_str = String::new();
        while self.peek(0).is_ascii_digit() {
            raw_str.push(self.advance().1);
        }

        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            raw_str.push(self.advance().1);
            while self.peek(0).is_ascii_digit() {
                raw_str.push(self.advance().1);
            }
        }

        if self.peek(0).to_ascii_lowercase() == 'e' {
            raw_str.push(self.advance().1);

            if self.peek(0) == '-' || self.peek(0) == '+' {
                raw_str.push(self.advance().1);
            }
            if self.is_at_end() || !self.peek(0).is_ascii_digit() {
                return Err(ScanError::MalformedNumberLiteral {
                    span: Span {
                        start: start,
                        end: start + raw_str.len(),
                        line: self.line,
                        line_end: self.line,
                    },
                });
            }
            while self.peek(0).is_ascii_digit() {
                raw_str.push(self.advance().1);
            }
        }

        let parsed = raw_str.parse::<f64>().unwrap();
        let len = raw_str.len();

        Ok(Token {
            tok_type: TokenType::Num,
            literal: Some(Num(parsed)),
            lexeme: Some(raw_str),
            span: Span {
                start: start,
                end: start + len,
                line: self.line,
                line_end: self.line,
            },
        })
    }

    fn scan_identifier(&mut self, start: usize, is_dollar: bool) -> Result<Token, ScanError> {
        let mut raw_str = String::new();
        raw_str.push(self.advance().1); // consume the first char
        while self.peek(0).is_alphabetic() || self.peek(0) == '_' {
            raw_str.push(self.advance().1);
        }
        let span = Span {
            start: start,
            end: start + raw_str.len(),
            line: self.line,
            line_end: self.line,
        };

        if CASE_SNS_KEYWORDS.contains_key(&raw_str) {
            Ok(Token {
                tok_type: CASE_SNS_KEYWORDS.get(&raw_str).unwrap().clone(),
                literal: None,
                lexeme: Some(raw_str),
                span,
            })
        } else if CASE_INS_KEYWORDS.contains_key(&raw_str.to_ascii_uppercase()) {
            Ok(Token {
                tok_type: CASE_INS_KEYWORDS
                    .get(&raw_str.to_ascii_uppercase())
                    .unwrap()
                    .clone(),
                literal: None,
                lexeme: Some(raw_str),
                span,
            })
        } else {
            Ok(Token {
                tok_type: Identifier { dollar: is_dollar },
                literal: Some(Str(Rc::new(raw_str.clone()))),
                lexeme: Some(raw_str),
                span,
            })
        }
    }

    fn scan_slash(&mut self, start: usize) -> Option<Token> {
        self.advance();
        if self.match_next('/') {
            while !self.is_at_end() && self.peek(0) != '\n' {
                self.advance();
            }
            None
        } else {
            Some(Token {
                tok_type: sym!(Slash),
                literal: None,
                lexeme: Some("/".to_owned()),
                span: Span {
                    start: start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            })
        }
    }

    fn scan_double_token(&mut self, start: usize, c: char) -> Token {
        self.advance();
        if self.match_next('=') {
            Token {
                tok_type: match c {
                    '!' => sym!(BangEqual),
                    '=' => sym!(EqualEqual),
                    '<' => sym!(LessEqual),
                    '>' => sym!(GreaterEqual),
                    _ => unreachable!(), // TODO(vck): fix
                },
                literal: None,
                lexeme: Some(c.to_string() + "="),
                span: Span {
                    start: start,
                    end: start + 2,
                    line: self.line,
                    line_end: self.line,
                },
            }
        } else {
            Token {
                tok_type: match c {
                    '!' => sym!(Bang),
                    '=' => sym!(Equal),
                    '<' => sym!(Less),
                    '>' => sym!(Greater),
                    _ => unreachable!(), // TODO(vck): fix
                },
                literal: None,
                lexeme: Some(c.to_string()),
                span: Span {
                    start: start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            }
        }
    }

    fn scan_other(&mut self, start: usize, other: char) -> Result<Token, ScanError> {
        self.advance();
        if let Some(sym) = SYMBOLS.get(&other) {
            Ok(Token {
                tok_type: sym.clone(),
                literal: None,
                lexeme: Some(other.to_string()),
                span: Span {
                    start: start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            })
        } else {
            return Err(ScanError::UnexpectedCharacter {
                span: Span {
                    start: start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            });
        }
    }

    fn scan_eof(&mut self, start: usize) -> Token {
        Token {
            tok_type: Eof,
            literal: None,
            lexeme: None,
            span: Span {
                start,
                end: start,
                line: self.line,
                line_end: self.line,
            },
        }
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>, ScanError> {
        let mut tokens: Vec<Token> = vec![];

        while !self.is_at_end() {
            let (start, c) = self.chars.peek().unwrap();
            let start_idx = *start;
            let start_char = *c;
            let matched = match c {
                '\n' => {
                    self.line += 1;
                    self.advance();
                    None
                }
                ' ' | '\r' | '\t' => {
                    self.advance();
                    None
                }
                '"' => Some(self.scan_string(start_idx)?),
                '0'..='9' => Some(self.scan_number(start_idx)?),
                'A'..='Z' | 'a'..='z' | '_' => Some(self.scan_identifier(start_idx, false)?),
                '$' => Some(self.scan_identifier(start_idx, true)?),
                '!' | '=' | '<' | '>' => Some(self.scan_double_token(start_idx, start_char)),
                '/' => self.scan_slash(start_idx),
                _ => Some(self.scan_other(start_idx, start_char)?),
            };
            if let Some(token) = matched {
                tokens.push(token);
            }
        }

        if tokens.is_empty() {
            tokens.push(self.scan_eof(0));
        } else {
            tokens.push(self.scan_eof(tokens.last().unwrap().span.end + 1));
        }

        Ok(tokens)
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
                        end: 1,
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
                        end: 2,
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
                        end: 3,
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
                        end: 4,
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
                        end: 5,
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
                        end: 6,
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
                        end: 7,
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
                        end: 8,
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
                        end: 9,
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
                        end: 10,
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
                        end: 11,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 12,
                        end: 12,
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
                        start: 12,
                        end: 25,
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
                        end: 30,
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
                        end: 36,
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
                        end: 3,
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
                        end: 6,
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
                        end: 12,
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
                        end: 17,
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
                        end: 21,
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
                        end: 25,
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
                        end: 28,
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
                        end: 34,
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
                        end: 43,
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
                        end: 50,
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
                        end: 56,
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
                        end: 61,
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
                        end: 65,
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
                        end: 71,
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
                        end: 76,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 77,
                        end: 77,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_sql_keywords() {
        assert_tokens(
            "Begin Transaction Rollback Commit Where Having Asc Desc Order By Explain Is Not Null Offset Like Limit And Or Join Inner Outer Right Left On Create Insert Update Delete Drop Into Values Index Collection Select From As Cross Default Group Key Of Only Primary References Set System Unique Read Write", vec![
            Token {tok_type: skw!(SqlKeyword::Begin), literal: None, lexeme: lexm!("Begin"), span: Span { line: 0, start: 0, line_end: 0, end: 5 }},
            Token {tok_type: skw!(SqlKeyword::Transaction), literal: None, lexeme: lexm!("Transaction") , span: Span { line: 0, start: 6, line_end: 0, end: 17 }},
            Token {tok_type: skw!(SqlKeyword::Rollback), literal: None, lexeme: lexm!("Rollback"), span: Span { line: 0, start: 18, line_end: 0, end: 26 }},
            Token {tok_type: skw!(SqlKeyword::Commit), literal: None , lexeme: lexm!("Commit"), span: Span {  line: 0, start: 27, line_end: 0, end: 33 }},
            Token {tok_type: skw!(SqlKeyword::Where), literal: None, lexeme: lexm!("Where"), span: Span { line: 0, start: 34, line_end: 0, end: 39 }},
            Token {tok_type: skw!(SqlKeyword::Having), literal: None , lexeme: lexm!("Having"), span: Span { line: 0, start: 40, line_end: 0, end: 46 }},
            Token {tok_type: skw!(SqlKeyword::Asc), literal: None, lexeme: lexm!("Asc") , span: Span { line: 0, start: 47, line_end: 0, end: 50 }},
            Token {tok_type: skw!(SqlKeyword::Desc), literal: None, lexeme: lexm!("Desc"), span: Span { line: 0, start: 51, line_end: 0, end: 55 }},
            Token {tok_type: skw!(SqlKeyword::Order), literal: None, lexeme: lexm!("Order"), span: Span { line: 0, start: 56, line_end: 0, end: 61 }},
            Token {tok_type: skw!(SqlKeyword::By), literal: None, lexeme: lexm!("By"), span: Span { line: 0, start: 62, line_end: 0, end: 64 }},
            Token {tok_type: skw!(SqlKeyword::Explain), literal: None, lexeme: lexm!("Explain"), span: Span { line: 0, start: 65, line_end: 0, end: 72 }},
            Token {tok_type: skw!(SqlKeyword::Is), literal: None, lexeme: lexm!("Is"), span: Span { line: 0, start: 73, line_end: 0, end: 75 }},
            Token {tok_type: skw!(SqlKeyword::Not), literal: None, lexeme: lexm!("Not"), span: Span { line: 0, start: 76, line_end: 0, end: 79 }},
            Token {tok_type: skw!(SqlKeyword::Null), literal: None, lexeme: lexm!("Null"), span: Span { line: 0, start: 80, line_end: 0, end: 84 }},
            Token {tok_type: skw!(SqlKeyword::Offset), literal: None, lexeme: lexm!("Offset"), span: Span { line: 0, start: 85, line_end: 0, end: 91 }},
            Token {tok_type: skw!(SqlKeyword::Like), literal: None, lexeme: lexm!("Like"), span: Span { line: 0, start: 92, line_end: 0, end: 96 }},
            Token {tok_type: skw!(SqlKeyword::Limit), literal: None, lexeme: lexm!("Limit"), span: Span { line: 0, start: 97, line_end: 0, end: 102 }},
            Token {tok_type: skw!(SqlKeyword::And), literal: None, lexeme: lexm!("And"), span: Span { line: 0, start: 103, line_end: 0, end: 106 }},
            Token {tok_type: skw!(SqlKeyword::Or), literal: None, lexeme: lexm!("Or"), span: Span { line: 0, start: 107, line_end: 0, end: 109 }},
            Token {tok_type: skw!(SqlKeyword::Join), literal: None, lexeme: lexm!("Join") , span: Span { line: 0, start: 110, line_end: 0, end: 114 }},
            Token {tok_type: skw!(SqlKeyword::Inner), literal: None, lexeme: lexm!("Inner") , span: Span { line: 0, start: 115, line_end: 0, end: 120 }},
            Token {tok_type: skw!(SqlKeyword::Outer), literal: None, lexeme: lexm!("Outer"), span: Span { line: 0, start: 121, line_end: 0, end: 126 }},
            Token {tok_type: skw!(SqlKeyword::Right), literal: None, lexeme: lexm!("Right"), span: Span { line: 0, start: 127, line_end: 0, end: 132 }},
            Token {tok_type: skw!(SqlKeyword::Left), literal: None, lexeme: lexm!("Left"), span: Span { line: 0, start: 133, line_end: 0, end: 137 }},
            Token {tok_type: skw!(SqlKeyword::On), literal: None, lexeme: lexm!("On") , span: Span { line: 0, start: 138, line_end: 0, end: 140 }},
            Token {tok_type: skw!(SqlKeyword::Create), literal: None, lexeme: lexm!("Create"), span: Span { line: 0, start: 141, line_end: 0, end: 147 }},
            Token {tok_type: skw!(SqlKeyword::Insert), literal: None, lexeme: lexm!("Insert"), span: Span { line: 0, start: 148, line_end: 0, end: 154 }},
            Token {tok_type: skw!(SqlKeyword::Update), literal: None, lexeme: lexm!("Update"), span: Span { line: 0, start: 155, line_end: 0, end: 161 }},
            Token {tok_type: skw!(SqlKeyword::Delete), literal: None, lexeme: lexm!("Delete"), span: Span { line: 0, start: 162, line_end: 0, end: 168 }},
            Token {tok_type: skw!(SqlKeyword::Drop), literal: None, lexeme: lexm!("Drop") , span: Span { line: 0, start: 169, line_end: 0, end: 173 }},
            Token {tok_type: skw!(SqlKeyword::Into), literal: None, lexeme: lexm!("Into") , span: Span { line: 0, start: 174, line_end: 0, end: 178 }},
            Token {tok_type: skw!(SqlKeyword::Values), literal: None , lexeme: lexm!("Values"), span: Span { line: 0, start: 179, line_end: 0, end: 185 }},
            Token {tok_type: skw!(SqlKeyword::Index), literal: None, lexeme: lexm!("Index"), span: Span { line: 0, start: 186, line_end: 0, end: 191 }},
            Token {tok_type: skw!(SqlKeyword::Collection), literal: None, lexeme: lexm!("Collection"), span: Span { line: 0, start: 192, line_end: 0, end: 202 }},
            Token {tok_type: skw!(SqlKeyword::Select), literal: None, lexeme: lexm!("Select"), span: Span { line: 0, start: 203, line_end: 0, end: 209 }},
            Token {tok_type: skw!(SqlKeyword::From), literal: None, lexeme: lexm!("From"), span: Span { line: 0, start: 210, line_end: 0, end: 214 }},
            Token {tok_type: skw!(SqlKeyword::As), literal: None, lexeme: lexm!("As"), span: Span { line: 0, start: 215, line_end: 0, end: 217 }},
            Token {tok_type: skw!(SqlKeyword::Cross), literal: None, lexeme: lexm!("Cross"), span: Span { line: 0, start: 218, line_end: 0, end: 223 }},
            Token {tok_type: skw!(SqlKeyword::Default), literal: None, lexeme: lexm!("Default"), span: Span { line: 0, start: 224, line_end: 0, end: 231 }},
            Token {tok_type: skw!(SqlKeyword::Group), literal: None, lexeme: lexm!("Group"), span: Span { line: 0, start: 232, line_end: 0, end: 237 }},
            Token {tok_type: skw!(SqlKeyword::Key), literal: None, lexeme: lexm!("Key"), span: Span { line: 0, start: 238, line_end: 0, end: 241 }},
            Token {tok_type: skw!(SqlKeyword::Of), literal: None, lexeme: lexm!("Of"), span: Span { line: 0, start: 242, line_end: 0, end: 244 }},
            Token {tok_type: skw!(SqlKeyword::Only), literal: None, lexeme: lexm!("Only"), span: Span { line: 0, start: 245, line_end: 0, end: 249 }},
            Token {tok_type: skw!(SqlKeyword::Primary), literal: None, lexeme: lexm!("Primary"), span: Span { line: 0, start: 250, line_end: 0, end: 257 }},
            Token {tok_type: skw!(SqlKeyword::References), literal: None, lexeme: lexm!("References"), span: Span { line: 0, start: 258, line_end: 0, end: 268 }},
            Token {tok_type: skw!(SqlKeyword::Set), literal: None, lexeme: lexm!("Set"), span: Span { line: 0, start: 269, line_end: 0, end: 272 }},
            Token {tok_type: skw!(SqlKeyword::System), literal: None, lexeme: lexm!("System"), span: Span { line: 0, start: 273, line_end: 0, end: 279 }},
            Token {tok_type: skw!(SqlKeyword::Unique), literal: None, lexeme: lexm!("Unique") , span: Span { line: 0, start: 280, line_end: 0, end: 286 }},
            Token {tok_type: skw!(SqlKeyword::Read), literal: None, lexeme: lexm!("Read"), span: Span { line: 0, start: 287, line_end: 0, end: 291 }},
            Token {tok_type: skw!(SqlKeyword::Write), literal: None, lexeme: lexm!("Write") , span: Span { line: 0, start: 292, line_end: 0, end: 297 }},
            Token {tok_type: Eof, literal: None, lexeme: None, span: Span { line: 0, start: 298, line_end: 0, end: 298 }},
        ]);
    }
}
