use serde::{Deserialize, Serialize};

use crate::ast::{Literal::*, Span};
use crate::sym;
use crate::tokenizer::token::Symbol::*;
use crate::tokenizer::token::TokenType::{Eof, Identifier};
use crate::tokenizer::token::*;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;
use std::sync::Arc;

pub struct Scanner<'a> {
    chars: Peekable<Enumerate<Chars<'a>>>,
    line: u32,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ScanError {
    UnexpectedCharacter { span: Span },
    UnterminatedString { span: Span },
    MalformedNumberLiteral { span: Span },
}

impl Scanner<'_> {
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

    fn scan_string(&mut self, start: usize, c: char) -> Result<Token, ScanError> {
        self.advance(); // consume the opening "
        let mut raw_str = String::new();
        while self.peek(0) != c && !self.is_at_end() {
            raw_str.push(self.advance().1);
        }

        if self.is_at_end() {
            return Err(ScanError::UnterminatedString {
                span: Span {
                    start,
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
            literal: Some(Str(Arc::new(raw_str.clone()))),
            lexeme: Some(raw_str),
            span: Span {
                start,
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
                        start,
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
                start,
                end: start + len,
                line: self.line,
                line_end: self.line,
            },
        })
    }

    fn scan_identifier(&mut self, start: usize, prev: Option<&Token>) -> Result<Token, ScanError> {
        let mut raw_str = String::new();
        while self.peek(0).is_alphabetic()
            || self.peek(0) == '_'
            || self.peek(0) == '$'
            || self.peek(0) == '\\'
            || self.peek(0).is_ascii_digit()
        {
            raw_str.push(self.advance().1);
        }
        let span = Span {
            start,
            end: start + raw_str.len(),
            line: self.line,
            line_end: self.line,
        };

        let is_escaped_identifier = raw_str.starts_with('\\');

        let is_coerced_identifier = is_escaped_identifier || {
            if let Some(prev) = prev {
                matches!(prev.tok_type, TokenType::Symbol(Dot))
            } else {
                false
            }
        };

        if !is_coerced_identifier && GENERIC_KEYWORDS.contains_key(&raw_str) {
            Ok(Token {
                tok_type: GENERIC_KEYWORDS.get(&raw_str).unwrap().clone(),
                literal: None,
                lexeme: Some(raw_str),
                span,
            })
        } else if !is_coerced_identifier && SQL_KEYWORDS.contains_key(&raw_str.to_ascii_uppercase())
        {
            Ok(Token {
                tok_type: SQL_KEYWORDS
                    .get(&raw_str.to_ascii_uppercase())
                    .unwrap()
                    .clone(),
                literal: None,
                lexeme: Some(raw_str),
                span,
            })
        } else {
            let literal = if is_escaped_identifier {
                Arc::new(raw_str[1..].to_string())
            } else {
                Arc::new(raw_str.clone())
            };

            Ok(Token {
                tok_type: Identifier {
                    dollar: literal.starts_with('$'),
                },
                literal: Some(Str(literal)),
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
                    start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            })
        }
    }

    fn scan_double_token(&mut self, start: usize, c: char) -> Token {
        self.advance();

        if self.match_next(':') && c == ':' {
            Token {
                tok_type: sym!(DoubleColon),
                literal: None,
                lexeme: Some("::".to_owned()),
                span: Span {
                    start,
                    end: start + 2,
                    line: self.line,
                    line_end: self.line,
                },
            }
        } else if self.match_next('&') && c == '&' {
            Token {
                tok_type: sym!(LogicalAnd),
                literal: None,
                lexeme: Some("&&".to_owned()),
                span: Span {
                    start,
                    end: start + 2,
                    line: self.line,
                    line_end: self.line,
                },
            }
        } else if self.match_next('|') && c == '|' {
            Token {
                tok_type: sym!(LogicalOr),
                literal: None,
                lexeme: Some("||".to_owned()),
                span: Span {
                    start,
                    end: start + 2,
                    line: self.line,
                    line_end: self.line,
                },
            }
        } else if self.match_next('=') {
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
                    start,
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
                    ':' => sym!(Colon),
                    _ => unreachable!(), // TODO(vck): fix
                },
                literal: None,
                lexeme: Some(c.to_string()),
                span: Span {
                    start,
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
                    start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            })
        } else {
            Err(ScanError::UnexpectedCharacter {
                span: Span {
                    start,
                    end: start + 1,
                    line: self.line,
                    line_end: self.line,
                },
            })
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
                '"' | '\'' | '`' => Some(self.scan_string(start_idx, start_char)?),
                '0'..='9' => Some(self.scan_number(start_idx)?),
                'A'..='Z' | 'a'..='z' | '_' | '$' | '\\' => {
                    Some(self.scan_identifier(start_idx, tokens.last())?)
                }
                '!' | '=' | '<' | '>' | '|' | '&' | ':' => {
                    Some(self.scan_double_token(start_idx, start_char))
                }
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
pub mod test_helpers {
    #[macro_export]
    macro_rules! lexm {
        ($a: literal) => {
            Some($a.to_owned())
        };
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::token::TokenType::Eof;
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
                    literal: Some(Str(Arc::new("hello world".to_string()))),
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
                    literal: Some(Str(Arc::new("helloIdentifier".to_string()))),
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
            "$myCustomVariable $my_custom_variable myCustomFunction my_custom_function \\for \\$edge_case",
            vec![
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Arc::new("$myCustomVariable".to_string()))),
                    lexeme: lexm!("$myCustomVariable"),
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 17,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Arc::new("$my_custom_variable".to_string()))),
                    lexeme: lexm!("$my_custom_variable"),
                    span: Span {
                        line: 0,
                        start: 18,
                        end: 37,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Arc::new("myCustomFunction".to_string()))),
                    lexeme: lexm!("myCustomFunction"),
                    span: Span {
                        line: 0,
                        start: 38,
                        end: 54,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Arc::new("my_custom_function".to_string()))),
                    lexeme: lexm!("my_custom_function"),
                    span: Span {
                        line: 0,
                        start: 55,
                        end: 73,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Arc::new("for".to_string()))),
                    lexeme: lexm!("\\for"),
                    span: Span {
                        line: 0,
                        start: 74,
                        end: 78,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Arc::new("$edge_case".to_string()))),
                    lexeme: lexm!("\\$edge_case"),
                    span: Span {
                        line: 0,
                        start: 79,
                        end: 90,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    literal: None,
                    lexeme: None,
                    span: Span {
                        line: 0,
                        start: 91,
                        end: 91,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_string_literals() {
        assert_tokens(
            "`abc` 'abc' \"abc\" `'abc'` '`abc`' `\"abc\"` \"`abc`\"",
            vec![
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("abc"),
                    literal: Some(Str(Arc::new("abc".to_string()))),
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 5,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("abc"),
                    literal: Some(Str(Arc::new("abc".to_string()))),
                    span: Span {
                        line: 0,
                        start: 6,
                        end: 11,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("abc"),
                    literal: Some(Str(Arc::new("abc".to_string()))),
                    span: Span {
                        line: 0,
                        start: 12,
                        end: 17,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("'abc'"),
                    literal: Some(Str(Arc::new("'abc'".to_string()))),
                    span: Span {
                        line: 0,
                        start: 18,
                        end: 25,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("`abc`"),
                    literal: Some(Str(Arc::new("`abc`".to_string()))),
                    span: Span {
                        line: 0,
                        start: 26,
                        end: 33,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("\"abc\""),
                    literal: Some(Str(Arc::new("\"abc\"".to_string()))),
                    span: Span {
                        line: 0,
                        start: 34,
                        end: 41,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    lexeme: lexm!("`abc`"),
                    literal: Some(Str(Arc::new("`abc`".to_string()))),
                    span: Span {
                        line: 0,
                        start: 42,
                        end: 49,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 50,
                        end: 50,
                        line_end: 0,
                    },
                },
            ],
        )
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
                    literal: Some(Num(1.797_693_134_862_315_7E308)),
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
            "&&  || class else for function if break continue return super this var while loop",
            vec![
                Token {
                    tok_type: sym!(Symbol::LogicalAnd),
                    lexeme: lexm!("&&"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 0,
                        end: 2,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: sym!(Symbol::LogicalOr),
                    lexeme: lexm!("||"),
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
                    lexeme: lexm!("function"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 22,
                        end: 30,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::If),
                    lexeme: lexm!("if"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 31,
                        end: 33,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Break),
                    lexeme: lexm!("break"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 34,
                        end: 39,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Continue),
                    lexeme: lexm!("continue"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 40,
                        end: 48,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Return),
                    lexeme: lexm!("return"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 49,
                        end: 55,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Super),
                    lexeme: lexm!("super"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 56,
                        end: 61,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::This),
                    lexeme: lexm!("this"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 62,
                        end: 66,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Var),
                    lexeme: lexm!("var"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 67,
                        end: 70,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::While),
                    lexeme: lexm!("while"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 71,
                        end: 76,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Loop),
                    lexeme: lexm!("loop"),
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 77,
                        end: 81,
                        line_end: 0,
                    },
                },
                Token {
                    tok_type: Eof,
                    lexeme: None,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 82,
                        end: 82,
                        line_end: 0,
                    },
                },
            ],
        );
    }

    #[test]
    fn test_sql_keywords() {
        assert_tokens(
            "Begin Transaction Rollback Commit Where Having Asc Desc Order By Explain Is Not      Offset Like Limit And Or Join Inner Right Left On Create Insert Update Delete Drop Into Values Index Collection Select From As Cross Default Group Key Of Only Primary References Set System Unique Read Write", vec![
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
            Token {tok_type: skw!(SqlKeyword::Offset), literal: None, lexeme: lexm!("Offset"), span: Span { line: 0, start: 85, line_end: 0, end: 91 }},
            Token {tok_type: skw!(SqlKeyword::Like), literal: None, lexeme: lexm!("Like"), span: Span { line: 0, start: 92, line_end: 0, end: 96 }},
            Token {tok_type: skw!(SqlKeyword::Limit), literal: None, lexeme: lexm!("Limit"), span: Span { line: 0, start: 97, line_end: 0, end: 102 }},
            Token {tok_type: skw!(SqlKeyword::And), literal: None, lexeme: lexm!("And"), span: Span { line: 0, start: 103, line_end: 0, end: 106 }},
            Token {tok_type: skw!(SqlKeyword::Or), literal: None, lexeme: lexm!("Or"), span: Span { line: 0, start: 107, line_end: 0, end: 109 }},
            Token {tok_type: skw!(SqlKeyword::Join), literal: None, lexeme: lexm!("Join") , span: Span { line: 0, start: 110, line_end: 0, end: 114 }},
            Token {tok_type: skw!(SqlKeyword::Inner), literal: None, lexeme: lexm!("Inner") , span: Span { line: 0, start: 115, line_end: 0, end: 120 }},
            Token {tok_type: skw!(SqlKeyword::Right), literal: None, lexeme: lexm!("Right"), span: Span { line: 0, start: 121, line_end: 0, end: 126 }},
            Token {tok_type: skw!(SqlKeyword::Left), literal: None, lexeme: lexm!("Left"), span: Span { line: 0, start: 127, line_end: 0, end: 131 }},
            Token {tok_type: skw!(SqlKeyword::On), literal: None, lexeme: lexm!("On") , span: Span { line: 0, start: 132, line_end: 0, end: 134 }},
            Token {tok_type: skw!(SqlKeyword::Create), literal: None, lexeme: lexm!("Create"), span: Span { line: 0, start: 135, line_end: 0, end: 141 }},
            Token {tok_type: skw!(SqlKeyword::Insert), literal: None, lexeme: lexm!("Insert"), span: Span { line: 0, start: 142, line_end: 0, end: 148 }},
            Token {tok_type: skw!(SqlKeyword::Update), literal: None, lexeme: lexm!("Update"), span: Span { line: 0, start: 149, line_end: 0, end: 155 }},
            Token {tok_type: skw!(SqlKeyword::Delete), literal: None, lexeme: lexm!("Delete"), span: Span { line: 0, start: 156, line_end: 0, end: 162 }},
            Token {tok_type: skw!(SqlKeyword::Drop), literal: None, lexeme: lexm!("Drop") , span: Span { line: 0, start: 163, line_end: 0, end: 167 }},
            Token {tok_type: skw!(SqlKeyword::Into), literal: None, lexeme: lexm!("Into") , span: Span { line: 0, start: 168, line_end: 0, end: 172 }},
            Token {tok_type: skw!(SqlKeyword::Values), literal: None , lexeme: lexm!("Values"), span: Span { line: 0, start: 173, line_end: 0, end: 179 }},
            Token {tok_type: skw!(SqlKeyword::Index), literal: None, lexeme: lexm!("Index"), span: Span { line: 0, start: 180, line_end: 0, end: 185 }},
            Token {tok_type: skw!(SqlKeyword::Collection), literal: None, lexeme: lexm!("Collection"), span: Span { line: 0, start: 186, line_end: 0, end: 196 }},
            Token {tok_type: skw!(SqlKeyword::Select), literal: None, lexeme: lexm!("Select"), span: Span { line: 0, start: 197, line_end: 0, end: 203 }},
            Token {tok_type: skw!(SqlKeyword::From), literal: None, lexeme: lexm!("From"), span: Span { line: 0, start: 204, line_end: 0, end: 208 }},
            Token {tok_type: skw!(SqlKeyword::As), literal: None, lexeme: lexm!("As"), span: Span { line: 0, start: 209, line_end: 0, end: 211 }},
            Token {tok_type: skw!(SqlKeyword::Cross), literal: None, lexeme: lexm!("Cross"), span: Span { line: 0, start: 212, line_end: 0, end: 217 }},
            Token {tok_type: skw!(SqlKeyword::Default), literal: None, lexeme: lexm!("Default"), span: Span { line: 0, start: 218, line_end: 0, end: 225 }},
            Token {tok_type: skw!(SqlKeyword::Group), literal: None, lexeme: lexm!("Group"), span: Span { line: 0, start: 226, line_end: 0, end: 231 }},
            Token {tok_type: skw!(SqlKeyword::Key), literal: None, lexeme: lexm!("Key"), span: Span { line: 0, start: 232, line_end: 0, end: 235 }},
            Token {tok_type: skw!(SqlKeyword::Of), literal: None, lexeme: lexm!("Of"), span: Span { line: 0, start: 236, line_end: 0, end: 238 }},
            Token {tok_type: skw!(SqlKeyword::Only), literal: None, lexeme: lexm!("Only"), span: Span { line: 0, start: 239, line_end: 0, end: 243 }},
            Token {tok_type: skw!(SqlKeyword::Primary), literal: None, lexeme: lexm!("Primary"), span: Span { line: 0, start: 244, line_end: 0, end: 251 }},
            Token {tok_type: skw!(SqlKeyword::References), literal: None, lexeme: lexm!("References"), span: Span { line: 0, start: 252, line_end: 0, end: 262 }},
            Token {tok_type: skw!(SqlKeyword::Set), literal: None, lexeme: lexm!("Set"), span: Span { line: 0, start: 263, line_end: 0, end: 266 }},
            Token {tok_type: skw!(SqlKeyword::System), literal: None, lexeme: lexm!("System"), span: Span { line: 0, start: 267, line_end: 0, end: 273 }},
            Token {tok_type: skw!(SqlKeyword::Unique), literal: None, lexeme: lexm!("Unique") , span: Span { line: 0, start: 274, line_end: 0, end: 280 }},
            Token {tok_type: skw!(SqlKeyword::Read), literal: None, lexeme: lexm!("Read"), span: Span { line: 0, start: 281, line_end: 0, end: 285 }},
            Token {tok_type: skw!(SqlKeyword::Write), literal: None, lexeme: lexm!("Write") , span: Span { line: 0, start: 286, line_end: 0, end: 291 }},
            Token {tok_type: Eof, literal: None, lexeme: None, span: Span { line: 0, start: 292, line_end: 0, end: 292 }},
        ]);
    }
}
