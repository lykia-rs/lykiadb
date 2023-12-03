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
            span: Span {
                line: self.line,
                start: self.start,
                lexeme: Rc::new(lexeme.to_string()),
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
            let err_span: String = self.chars[self.start + 1..self.current].iter().collect();
            return Err(ScanError::UnterminatedString {
                span: Span {
                    start: self.start + 1,
                    lexeme: Rc::new(err_span),
                    line: self.line,
                },
            });
        }

        self.advance();

        let span: String = self.chars[self.start + 1..self.current - 1]
            .iter()
            .collect();
        let value = Rc::new(span.to_string());

        self.tokens.push(Token {
            tok_type: TokenType::Str,
            literal: Some(Str(value.clone())),
            span: Span {
                line: self.line,
                start: self.start + 1,
                lexeme: value,
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
                let err_span: String = self.chars[self.start..self.current].iter().collect();
                return Err(ScanError::MalformedNumberLiteral {
                    span: Span {
                        start: self.start,
                        lexeme: Rc::new(err_span),
                        line: self.line,
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
            span: Span {
                start: self.start,
                lexeme: Rc::new(span),
                line: self.line,
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
            let value = Rc::new(span.to_string());
            self.tokens.push(Token {
                tok_type: Identifier { dollar: is_dollar },
                literal: Some(Str(value.clone())),
                span: Span {
                    start: self.start,
                    lexeme: value,
                    line: self.line,
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
                            line: self.line,
                            lexeme: Rc::new(String::from(c)),
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

        self.add_token(" ", Eof);
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

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 0,
                        lexeme: lexm!("("),
                    },
                },
                Token {
                    tok_type: sym!(RightParen),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 1,
                        lexeme: lexm!(")"),
                    },
                },
                Token {
                    tok_type: sym!(LeftBrace),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 2,
                        lexeme: lexm!("{"),
                    },
                },
                Token {
                    tok_type: sym!(RightBrace),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 3,
                        lexeme: lexm!("}"),
                    },
                },
                Token {
                    tok_type: sym!(Semicolon),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 4,
                        lexeme: lexm!(";"),
                    },
                },
                Token {
                    tok_type: sym!(Comma),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 5,
                        lexeme: lexm!(","),
                    },
                },
                Token {
                    tok_type: sym!(Plus),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 6,
                        lexeme: lexm!("+"),
                    },
                },
                Token {
                    tok_type: sym!(Minus),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 7,
                        lexeme: lexm!("-"),
                    },
                },
                Token {
                    tok_type: sym!(Star),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 8,
                        lexeme: lexm!("*"),
                    },
                },
                Token {
                    tok_type: sym!(Slash),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 9,
                        lexeme: lexm!("/"),
                    },
                },
                Token {
                    tok_type: sym!(Dot),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 10,
                        lexeme: lexm!("."),
                    },
                },
                Token {
                    tok_type: Eof,

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 10,
                        lexeme: lexm!(" "),
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
                    span: Span {
                        line: 0,
                        start: 0,
                        lexeme: lexm!("123"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,
                    literal: Some(Num(123.456)),
                    span: Span {
                        line: 0,
                        start: 4,
                        lexeme: lexm!("123.456"),
                    },
                },
                Token {
                    tok_type: TokenType::Str,
                    literal: Some(Str(Rc::new("hello world".to_string()))),
                    span: Span {
                        line: 0,
                        start: 13,
                        lexeme: lexm!("hello world"),
                    },
                },
                Token {
                    tok_type: TokenType::True,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 26,
                        lexeme: lexm!("true"),
                    },
                },
                Token {
                    tok_type: TokenType::False,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 31,
                        lexeme: lexm!("false"),
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("helloIdentifier".to_string()))),
                    span: Span {
                        line: 0,
                        start: 37,
                        lexeme: lexm!("helloIdentifier"),
                    },
                },
                Token {
                    tok_type: Eof,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 37,
                        lexeme: lexm!(" "),
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
                    span: Span {
                        line: 0,
                        start: 0,
                        lexeme: lexm!("$myPreciseVariable"),
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: true },
                    literal: Some(Str(Rc::new("$my_precise_variable".to_string()))),
                    span: Span {
                        line: 0,
                        start: 19,
                        lexeme: lexm!("$my_precise_variable"),
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("myPreciseFunction".to_string()))),
                    span: Span {
                        line: 0,
                        start: 40,
                        lexeme: lexm!("myPreciseFunction"),
                    },
                },
                Token {
                    tok_type: TokenType::Identifier { dollar: false },
                    literal: Some(Str(Rc::new("my_precise_function".to_string()))),
                    span: Span {
                        line: 0,
                        start: 58,
                        lexeme: lexm!("my_precise_function"),
                    },
                },
                Token {
                    tok_type: Eof,
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 58,
                        lexeme: lexm!(" "),
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

                    literal: Some(Num(0.0)),
                    span: Span {
                        line: 0,
                        start: 0,
                        lexeme: lexm!("0"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(1.0)),
                    span: Span {
                        line: 0,
                        start: 2,
                        lexeme: lexm!("1"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(2.0)),
                    span: Span {
                        line: 0,
                        start: 4,
                        lexeme: lexm!("2"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(3.0)),
                    span: Span {
                        line: 0,
                        start: 6,
                        lexeme: lexm!("3"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(4.0)),
                    span: Span {
                        line: 0,
                        start: 8,
                        lexeme: lexm!("4"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(5.0)),
                    span: Span {
                        line: 0,
                        start: 10,
                        lexeme: lexm!("5"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(6.0)),
                    span: Span {
                        line: 0,
                        start: 12,
                        lexeme: lexm!("6"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(7.0)),
                    span: Span {
                        line: 0,
                        start: 14,
                        lexeme: lexm!("7"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(8.0)),
                    span: Span {
                        line: 0,
                        start: 16,
                        lexeme: lexm!("8"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(9.0)),
                    span: Span {
                        line: 0,
                        start: 18,
                        lexeme: lexm!("9"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(10.0)),
                    span: Span {
                        line: 0,
                        start: 20,
                        lexeme: lexm!("10"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(100.0)),
                    span: Span {
                        line: 0,
                        start: 23,
                        lexeme: lexm!("100"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(500.0)),
                    span: Span {
                        line: 0,
                        start: 27,
                        lexeme: lexm!("500"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(1000.0)),
                    span: Span {
                        line: 0,
                        start: 31,
                        lexeme: lexm!("1000"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(1.7976931348623157E+308)),
                    span: Span {
                        line: 0,
                        start: 36,
                        lexeme: lexm!("1.7976931348623157E+308"),
                    },
                },
                Token {
                    tok_type: TokenType::Num,

                    literal: Some(Num(1.7976931348623157E-308)),
                    span: Span {
                        line: 0,
                        start: 60,
                        lexeme: lexm!("1.7976931348623157E-308"),
                    },
                },
                Token {
                    tok_type: Eof,

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 60,
                        lexeme: lexm!(" "),
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
                    literal: None,
                    span: Span {
                        line: 0,
                        start: 0,
                        lexeme: lexm!("and"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Or),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 4,
                        lexeme: lexm!("or"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Class),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 7,
                        lexeme: lexm!("class"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Else),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 13,
                        lexeme: lexm!("else"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::For),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 18,
                        lexeme: lexm!("for"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Fun),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 22,
                        lexeme: lexm!("fun"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::If),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 26,
                        lexeme: lexm!("if"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Break),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 29,
                        lexeme: lexm!("break"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Continue),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 35,
                        lexeme: lexm!("continue"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Return),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 44,
                        lexeme: lexm!("return"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Super),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 51,
                        lexeme: lexm!("super"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::This),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 57,
                        lexeme: lexm!("this"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Var),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 62,
                        lexeme: lexm!("var"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::While),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 66,
                        lexeme: lexm!("while"),
                    },
                },
                Token {
                    tok_type: kw!(Keyword::Loop),

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 72,
                        lexeme: lexm!("loop"),
                    },
                },
                Token {
                    tok_type: Eof,

                    literal: None,
                    span: Span {
                        line: 0,
                        start: 72,
                        lexeme: lexm!(" "),
                    },
                },
            ],
        );
    }

    #[test]
    fn test_sql_keywords() {
        assert_tokens(
            "Begin Transaction Rollback Commit Where Having Asc Desc Order By Explain Is Not Null Offset Like Limit And Or Join Inner Outer Right Left On Create Insert Update Delete Drop Into Values Index Table Select From As Cross Default Group Key Of Only Primary References Set System Unique Read Write", vec![
            Token {tok_type: skw!(SqlKeyword::Begin), literal: None, span: Span { line: 0, start: 0, lexeme: lexm!("Begin")} },
            Token {tok_type: skw!(SqlKeyword::Transaction), literal: None, span: Span { line: 0, start: 6, lexeme: lexm!("Transaction") } },
            Token {tok_type: skw!(SqlKeyword::Rollback), literal: None, span: Span { line: 0, start: 18 , lexeme: lexm!("Rollback") }},
            Token {tok_type: skw!(SqlKeyword::Commit), literal: None,span: Span {  line: 0, start: 27 , lexeme: lexm!("Commit") }},
            Token {tok_type: skw!(SqlKeyword::Where), literal: None, span: Span { line: 0, start: 34 , lexeme: lexm!("Where") }},
            Token {tok_type: skw!(SqlKeyword::Having), literal: None, span: Span { line: 0, start: 40 , lexeme: lexm!("Having") }},
            Token {tok_type: skw!(SqlKeyword::Asc), literal: None, span: Span { line: 0, start: 47 , lexeme: lexm!("Asc") }},
            Token {tok_type: skw!(SqlKeyword::Desc), literal: None, span: Span { line: 0, start: 51 , lexeme: lexm!("Desc") }},
            Token {tok_type: skw!(SqlKeyword::Order), literal: None, span: Span { line: 0, start: 56 , lexeme: lexm!("Order") }},
            Token {tok_type: skw!(SqlKeyword::By), literal: None, span: Span { line: 0, start: 62 , lexeme: lexm!("By") }},
            Token {tok_type: skw!(SqlKeyword::Explain), literal: None, span: Span { line: 0, start: 65 , lexeme: lexm!("Explain") }},
            Token {tok_type: skw!(SqlKeyword::Is), literal: None, span: Span { line: 0, start: 73 , lexeme: lexm!("Is") }},
            Token {tok_type: skw!(SqlKeyword::Not), literal: None, span: Span { line: 0, start: 76, lexeme: lexm!("Not")  }},
            Token {tok_type: skw!(SqlKeyword::Null), literal: None, span: Span { line: 0, start: 80 , lexeme: lexm!("Null") }},
            Token {tok_type: skw!(SqlKeyword::Offset), literal: None, span: Span { line: 0, start: 85 , lexeme: lexm!("Offset") }},
            Token {tok_type: skw!(SqlKeyword::Like), literal: None, span: Span { line: 0, start: 92 , lexeme: lexm!("Like") }},
            Token {tok_type: skw!(SqlKeyword::Limit), literal: None, span: Span { line: 0, start: 97 , lexeme: lexm!("Limit") }},
            Token {tok_type: skw!(SqlKeyword::And), literal: None, span: Span { line: 0, start: 103, lexeme: lexm!("And")}},
            Token {tok_type: skw!(SqlKeyword::Or), literal: None, span: Span { line: 0, start: 107, lexeme: lexm!("Or")}},
            Token {tok_type: skw!(SqlKeyword::Join), literal: None, span: Span { line: 0, start: 110, lexeme: lexm!("Join") }},
            Token {tok_type: skw!(SqlKeyword::Inner), literal: None, span: Span { line: 0, start: 115, lexeme: lexm!("Inner") }},
            Token {tok_type: skw!(SqlKeyword::Outer), literal: None, span: Span { line: 0, start: 121, lexeme: lexm!("Outer") }},
            Token {tok_type: skw!(SqlKeyword::Right), literal: None, span: Span { line: 0, start: 127, lexeme: lexm!("Right") }},
            Token {tok_type: skw!(SqlKeyword::Left), literal: None, span: Span { line: 0, start: 133, lexeme: lexm!("Left") }},
            Token {tok_type: skw!(SqlKeyword::On), literal: None, span: Span { line: 0, start: 138, lexeme: lexm!("On") }},
            Token {tok_type: skw!(SqlKeyword::Create), literal: None, span: Span { line: 0, start: 141, lexeme: lexm!("Create") }},
            Token {tok_type: skw!(SqlKeyword::Insert), literal: None, span: Span { line: 0, start: 148, lexeme: lexm!("Insert") }},
            Token {tok_type: skw!(SqlKeyword::Update), literal: None, span: Span { line: 0, start: 155, lexeme: lexm!("Update") }},
            Token {tok_type: skw!(SqlKeyword::Delete), literal: None, span: Span { line: 0, start: 162, lexeme: lexm!("Delete") }},
            Token {tok_type: skw!(SqlKeyword::Drop), literal: None, span: Span { line: 0, start: 169, lexeme: lexm!("Drop") }},
            Token {tok_type: skw!(SqlKeyword::Into), literal: None, span: Span { line: 0, start: 174, lexeme: lexm!("Into") }},
            Token {tok_type: skw!(SqlKeyword::Values), literal: None, span: Span { line: 0, start: 179 , lexeme: lexm!("Values") }},
            Token {tok_type: skw!(SqlKeyword::Index), literal: None, span: Span { line: 0, start: 186, lexeme: lexm!("Index")}},
            Token {tok_type: skw!(SqlKeyword::Table), literal: None, span: Span { line: 0, start: 192, lexeme: lexm!("Table") }},
            Token {tok_type: skw!(SqlKeyword::Select), literal: None, span: Span { line: 0, start: 198, lexeme: lexm!("Select") }},
            Token {tok_type: skw!(SqlKeyword::From), literal: None, span: Span { line: 0, start: 205, lexeme: lexm!("From") }},
            Token {tok_type: skw!(SqlKeyword::As), literal: None, span: Span { line: 0, start: 210, lexeme: lexm!("As") }},
            Token {tok_type: skw!(SqlKeyword::Cross), literal: None, span: Span { line: 0, start: 213, lexeme: lexm!("Cross") }},
            Token {tok_type: skw!(SqlKeyword::Default), literal: None, span: Span { line: 0, start: 219, lexeme: lexm!("Default") }},
            Token {tok_type: skw!(SqlKeyword::Group), literal: None, span: Span { line: 0, start: 227, lexeme: lexm!("Group") }},
            Token {tok_type: skw!(SqlKeyword::Key), literal: None, span: Span { line: 0, start: 233, lexeme: lexm!("Key") }},
            Token {tok_type: skw!(SqlKeyword::Of), literal: None, span: Span { line: 0, start: 237, lexeme: lexm!("Of") }},
            Token {tok_type: skw!(SqlKeyword::Only), literal: None, span: Span { line: 0, start: 240, lexeme: lexm!("Only") }},
            Token {tok_type: skw!(SqlKeyword::Primary), literal: None, span: Span { line: 0, start: 245, lexeme: lexm!("Primary") }},
            Token {tok_type: skw!(SqlKeyword::References), literal: None, span: Span { line: 0, start: 253, lexeme: lexm!("References")  }},
            Token {tok_type: skw!(SqlKeyword::Set), literal: None, span: Span { line: 0, start: 264, lexeme: lexm!("Set") }},
            Token {tok_type: skw!(SqlKeyword::System), literal: None, span: Span { line: 0, start: 268, lexeme: lexm!("System") }},
            Token {tok_type: skw!(SqlKeyword::Unique), literal: None, span: Span { line: 0, start: 275, lexeme: lexm!("Unique") }},
            Token {tok_type: skw!(SqlKeyword::Read), literal: None, span: Span { line: 0, start: 282, lexeme: lexm!("Read") }},
            Token {tok_type: skw!(SqlKeyword::Write), literal: None, span: Span { line: 0, start: 287, lexeme: lexm!("Write")  }},
            Token {tok_type: Eof, literal: None, span: Span { line: 0, start: 287, lexeme: lexm!(" ")} }
        ]);
    }
}
