use std::collections::HashMap;

/// A single block inside a `@test` body.
#[derive(Debug, PartialEq, Clone)]
pub enum Block {
    Input(String),
    Expect(String),
    ExpectErr(String),
}

/// A flat, resolved test case ready for a `TestHandler`.
#[derive(Debug, PartialEq, Clone)]
pub struct TestCase {
    pub name: String,
    pub flags: HashMap<String, String>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum SuiteItem {
    Test(TestCase),
    Group {
        name: String,
        flags: HashMap<String, String>,
        children: Vec<SuiteItem>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedEof { context: String },
    UnexpectedToken { position: usize, expected: String, got: String },
}

pub type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct TestLangParser {
    chars: Vec<char>,
    pos: usize,
}

impl TestLangParser {
    pub fn new(input: &str) -> Self {
        TestLangParser { chars: input.chars().collect(), pos: 0 }
    }

    fn cur(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.cur();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn starts_with(&self, s: &str) -> bool {
        let tail = &self.chars[self.pos..];
        let needle: Vec<char> = s.chars().collect();
        tail.starts_with(&needle)
    }

    fn eat(&mut self, s: &str) {
        self.pos += s.chars().count();
    }

    fn skip_ws(&mut self) {
        while matches!(self.cur(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.advance();
        }
    }

    fn skip_line(&mut self) {
        while !matches!(self.cur(), Some('\n') | None) {
            self.advance();
        }
    }

    fn parse_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.cur() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn expect_char(&mut self, expected: char) -> ParseResult<()> {
        self.skip_ws();
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(ParseError::UnexpectedToken {
                position: self.pos,
                expected: expected.to_string(),
                got: c.to_string(),
            }),
            None => Err(ParseError::UnexpectedEof {
                context: format!("expected '{expected}'"),
            }),
        }
    }

    /// Parse content inside `{ … }` with balanced inner braces.
    /// The opening `{` has already been consumed.
    fn parse_braced(&mut self) -> ParseResult<String> {
        let mut s = String::new();
        let mut depth = 0usize;
        loop {
            match self.advance() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside braced block".into(),
                    })
                }
                Some('{') => {
                    depth += 1;
                    s.push('{');
                }
                Some('}') => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    s.push('}');
                }
                Some(c) => s.push(c),
            }
        }
        Ok(s)
    }

    /// `@set(key = "value")` the `@set` keyword has already been consumed.
    fn parse_set(&mut self) -> ParseResult<(String, String)> {
        self.expect_char('(')?;
        self.skip_ws();
        let key = self.parse_ident();
        self.skip_ws();
        self.expect_char('=')?;
        self.skip_ws();
        self.expect_char('"')?;
        let mut value = String::new();
        loop {
            match self.advance() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside @set string value".into(),
                    })
                }
                Some('"') => break,
                Some(c) => value.push(c),
            }
        }
        self.skip_ws();
        self.expect_char(')')?;
        Ok((key, value))
    }

    /// Body of a `@test` block.  The opening `{` has already been consumed.
    fn parse_test_body(&mut self) -> ParseResult<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut cur_input = String::new();
        let mut depth = 0usize;

        loop {
            match self.cur() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside @test body".into(),
                    })
                }

                Some('@') => {
                    self.advance();
                    if self.starts_with("expect_err") {
                        self.eat("expect_err");
                        if !cur_input.trim().is_empty() {
                            blocks.push(Block::Input(trim_code(&cur_input)));
                            cur_input.clear();
                        }
                        self.expect_char('{')?;
                        let content = self.parse_braced()?;
                        blocks.push(Block::ExpectErr(content));
                    } else if self.starts_with("expect") {
                        self.eat("expect");
                        if !cur_input.trim().is_empty() {
                            blocks.push(Block::Input(trim_code(&cur_input)));
                            cur_input.clear();
                        }
                        self.expect_char('{')?;
                        let content = self.parse_braced()?;
                        blocks.push(Block::Expect(content));
                    } else {
                        cur_input.push('@');
                    }
                }

                Some('{') => {
                    depth += 1;
                    self.advance();
                    cur_input.push('{');
                }

                Some('}') => {
                    if depth == 0 {
                        self.advance();
                        if !cur_input.trim().is_empty() {
                            blocks.push(Block::Input(trim_code(&cur_input)));
                        }
                        return Ok(blocks);
                    }
                    depth -= 1;
                    self.advance();
                    cur_input.push('}');
                }

                Some(c) => {
                    self.advance();
                    cur_input.push(c);
                }
            }
        }
    }

    /// Body of a `@group` block.  The opening `{` has already been consumed.
    fn parse_group_body(
        &mut self,
    ) -> ParseResult<(HashMap<String, String>, Vec<SuiteItem>)> {
        let mut flags: HashMap<String, String> = HashMap::new();
        let mut children: Vec<SuiteItem> = Vec::new();

        loop {
            self.skip_ws();
            match self.cur() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside @group body".into(),
                    })
                }

                Some('#') => self.skip_line(),

                Some('@') => {
                    self.advance();
                    let kw = self.parse_ident();
                    match kw.as_str() {
                        "set" => {
                            let (k, v) = self.parse_set()?;
                            flags.insert(k, v);
                        }
                        "test" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let test_blocks = self.parse_test_body()?;
                            children.push(SuiteItem::Test(TestCase {
                                name,
                                flags: HashMap::new(),
                                blocks: test_blocks,
                            }));
                        }
                        "group" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let (sub_flags, sub_children) =
                                self.parse_group_body()?;
                            children.push(SuiteItem::Group {
                                name,
                                flags: sub_flags,
                                children: sub_children,
                            });
                        }
                        other => {
                            return Err(ParseError::UnexpectedToken {
                                position: self.pos,
                                expected: "set, test, or group".into(),
                                got: other.to_string(),
                            })
                        }
                    }
                }

                Some('}') => {
                    self.advance();
                    return Ok((flags, children));
                }

                Some(c) => {
                    return Err(ParseError::UnexpectedToken {
                        position: self.pos,
                        expected: "@directive or }".into(),
                        got: c.to_string(),
                    })
                }
            }
        }
    }

    /// Top-level entry point.  Parses a sequence of `@test` / `@group` items.
    pub fn parse(&mut self) -> ParseResult<Vec<SuiteItem>> {
        let mut items: Vec<SuiteItem> = Vec::new();

        loop {
            self.skip_ws();
            match self.cur() {
                None => break,

                Some('#') => self.skip_line(),

                Some('@') => {
                    self.advance();
                    let kw = self.parse_ident();
                    match kw.as_str() {
                        "test" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let blocks = self.parse_test_body()?;
                            items.push(SuiteItem::Test(TestCase {
                                name,
                                flags: HashMap::new(),
                                blocks,
                            }));
                        }
                        "group" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let (grp_flags, children) =
                                self.parse_group_body()?;
                            items.push(SuiteItem::Group {
                                name,
                                flags: grp_flags,
                                children,
                            });
                        }
                        other => {
                            return Err(ParseError::UnexpectedToken {
                                position: self.pos,
                                expected: "test or group".into(),
                                got: other.to_string(),
                            })
                        }
                    }
                }

                Some(c) => {
                    return Err(ParseError::UnexpectedToken {
                        position: self.pos,
                        expected: "@directive or EOF".into(),
                        got: c.to_string(),
                    })
                }
            }
        }

        Ok(items)
    }
}

/// Recursively flatten a suite tree into a flat list of `TestCase`s, merging
/// flags from enclosing `@group`s into each test.
pub(crate) fn flatten_items(
    items: &[SuiteItem],
    inherited: &HashMap<String, String>,
    prefix: &str,
) -> Vec<TestCase> {
    let mut out = Vec::new();
    for item in items {
        match item {
            SuiteItem::Test(tc) => {
                let mut flags = inherited.clone();
                flags.extend(tc.flags.clone());
                let full_name = if prefix.is_empty() {
                    tc.name.clone()
                } else {
                    format!("{}.{}", prefix, tc.name)
                };
                out.push(TestCase { name: full_name, flags, blocks: tc.blocks.clone() });
            }
            SuiteItem::Group { name, flags, children, .. } => {
                let mut merged = inherited.clone();
                merged.extend(flags.clone());
                let child_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}.{name}")
                };
                out.extend(flatten_items(children, &merged, &child_prefix));
            }
        }
    }
    out
}

/// Strip common leading whitespace (dedent) and trim outer blank lines.
pub fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();

    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let dedented = lines
        .iter()
        .map(|l| if l.len() >= min_indent { &l[min_indent..] } else { l.trim_start() })
        .collect::<Vec<_>>()
        .join("\n");

    dedented.trim().to_string()
}

fn trim_code(s: &str) -> String {
    s.trim_matches('\n').trim_matches('\r').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_items(src: &str) -> Vec<SuiteItem> {
        TestLangParser::new(src).parse().expect("parse failed")
    }

    fn flat(src: &str) -> Vec<TestCase> {
        let items = TestLangParser::new(src).parse().expect("parse failed");
        flatten_items(&items, &HashMap::new(), "")
    }

    fn flags_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn dedent_strips_common_indent() {
        assert_eq!(dedent("\n    hello\n    world\n"), "hello\nworld");
    }

    #[test]
    fn dedent_preserves_relative_indent() {
        assert_eq!(dedent("\n    - outer\n      - inner\n"), "- outer\n  - inner");
    }

    #[test]
    fn dedent_handles_empty() {
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("   "), "");
    }

    #[test]
    fn parse_minimal_test() {
        let items = parse_items("@test foo { some code }");
        assert_eq!(
            items,
            vec![SuiteItem::Test(TestCase {
                name: "foo".into(),
                flags: HashMap::new(),
                // trim_code strips surrounding newlines; spaces are dedented at run time
                blocks: vec![Block::Input(" some code ".into())],
            })]
        );
    }

    #[test]
    fn parse_test_with_expect() {
        let src = "@test greet {\n    testutil::print(\"hello\");\n    @expect {\n        hello\n    }\n}";
        let cases = flat(src);
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "greet");
        assert!(matches!(&cases[0].blocks[0], Block::Input(_)));
        assert!(matches!(&cases[0].blocks[1], Block::Expect(_)));
    }

    #[test]
    fn parse_test_with_expect_err() {
        let src = "@test bad {\n    do_error();\n    @expect_err {\n        SomeError\n    }\n}";
        let cases = flat(src);
        assert!(matches!(&cases[0].blocks[1], Block::ExpectErr(_)));
    }

    #[test]
    fn parse_test_interleaved_blocks() {
        let src = "@test multi {\n    code1();\n    @expect_err { err1 }\n    code2();\n    @expect_err { err2 }\n}";
        let cases = flat(src);
        assert_eq!(cases[0].blocks.len(), 4);
    }

    #[test]
    fn parse_nested_braces_in_code() {
        let src = "@test loops {\n    for (var $i = 0; $i < 3; $i = $i + 1) {\n        print($i);\n    }\n    @expect {\n        0\n        1\n        2\n    }\n}";
        let cases = flat(src);
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains("for"));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn parse_group_flags_inherited() {
        let src = "@group suite {\n    @set(run = \"plan\")\n    @test q1 { code @expect { result } }\n    @test q2 { code @expect { result } }\n}";
        let cases = flat(src);
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].flags.get("run"), Some(&"plan".to_string()));
        assert_eq!(cases[1].flags.get("run"), Some(&"plan".to_string()));
    }

    #[test]
    fn parse_nested_group_flags_merged() {
        let src = "@group outer {\n    @set(run = \"plan\")\n    @group inner {\n        @set(extra = \"yes\")\n        @test t { code @expect { x } }\n    }\n}";
        let cases = flat(src);
        assert_eq!(cases[0].flags, flags_map(&[("run", "plan"), ("extra", "yes")]));
    }

    #[test]
    fn parse_nested_group_flag_override() {
        let src = "@group outer {\n    @set(run = \"plan\")\n    @group inner {\n        @set(run = \"interpreter\")\n        @test t { code @expect { x } }\n    }\n}";
        let cases = flat(src);
        assert_eq!(cases[0].flags.get("run"), Some(&"interpreter".to_string()));
    }

    #[test]
    fn parse_top_level_comment() {
        let src = "# comment\n@test name { code @expect { result } }";
        assert_eq!(flat(src).len(), 1);
    }

    #[test]
    fn parse_multiple_top_level_tests() {
        let src = "@test first { a @expect { 1 } }\n@test second { b @expect { 2 } }";
        let cases = flat(src);
        assert_eq!(cases[0].name, "first");
        assert_eq!(cases[1].name, "second");
    }

    #[test]
    fn parse_error_unexpected_token() {
        assert!(matches!(
            TestLangParser::new("garbage").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_unknown_directive_in_group() {
        assert!(matches!(
            TestLangParser::new("@group g { @unknown { } }").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_eof_in_braced() {
        assert!(matches!(
            TestLangParser::new("@test t { code @expect {").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn parse_error_eof_in_test_body() {
        assert!(matches!(
            TestLangParser::new("@test t { code without closing brace").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }
}
