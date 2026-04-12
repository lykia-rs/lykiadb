use std::collections::HashMap;

/// A single block inside a `@test` body.
#[derive(Debug, PartialEq, Clone)]
pub enum Block {
    Input(String),
    ExpectValue(String),
    ExpectOutput(String),
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
    UnexpectedEof {
        context: String,
    },
    UnexpectedToken {
        position: usize,
        expected: String,
        got: String,
    },
    NoAssertions {
        name: String,
    },
}

pub type ParseResult<T> = Result<T, ParseError>;

pub(crate) struct TestLangParser {
    chars: Vec<char>,
    pos: usize,
}

impl TestLangParser {
    pub fn new(input: &str) -> Self {
        TestLangParser {
            chars: input.chars().collect(),
            pos: 0,
        }
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
                    });
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

    /// Consume a double-quoted string literal after the opening `"` has been
    /// consumed, pushing the full literal (including surrounding quotes) onto
    /// `out`.  Handles `\"` and `\\` escapes so inner quotes don't terminate
    /// the scan early.
    fn scan_string_literal(&mut self, out: &mut String) -> ParseResult<()> {
        out.push('"');
        loop {
            match self.advance() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside string literal".into(),
                    });
                }
                Some('\\') => {
                    out.push('\\');
                    match self.advance() {
                        None => {
                            return Err(ParseError::UnexpectedEof {
                                context: "inside string escape".into(),
                            });
                        }
                        Some(c) => out.push(c),
                    }
                }
                Some('"') => {
                    out.push('"');
                    break;
                }
                Some(c) => out.push(c),
            }
        }
        Ok(())
    }

    /// Consume from the current position to the end of the line (exclusive),
    /// pushing everything onto `out`.  Call after the comment opener (`#` or
    /// `//`) has already been pushed onto `out`.
    fn scan_line_comment(&mut self, out: &mut String) {
        while !matches!(self.cur(), Some('\n') | None) {
            out.push(self.advance().unwrap());
        }
    }

    /// Parse `(key = "value")` : the keyword before it has already been consumed.
    fn parse_annotation_args(&mut self) -> ParseResult<(String, String)> {
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
                        context: "inside annotation value".into(),
                    });
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
    /// `@expect output { … }` → output match; `@expect error { … }` → error match.
    /// At least one expect block is required.
    fn parse_test_body(&mut self, name: &str) -> ParseResult<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut cur_input = String::new();
        let mut depth = 0usize;

        loop {
            match self.cur() {
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside @test body".into(),
                    });
                }

                Some('@') => {
                    self.advance();
                    let kw = self.parse_ident();
                    if kw == "expect" {
                        if !cur_input.trim().is_empty() {
                            blocks.push(Block::Input(trim_code(&cur_input)));
                            cur_input.clear();
                        }
                        self.skip_ws();
                        if self.cur() != Some('{') {
                            let qualifier = self.parse_ident();
                            if qualifier == "error" {
                                self.expect_char('{')?;
                                blocks.push(Block::ExpectErr(self.parse_braced()?));
                            }
                            else if qualifier == "output" {
                                self.expect_char('{')?;
                                blocks.push(Block::ExpectOutput(self.parse_braced()?));
                            }
                            else {
                                return Err(ParseError::UnexpectedToken {
                                    position: self.pos,
                                    expected: "error, output or {".into(),
                                    got: qualifier,
                                });
                            }
                        } else {
                            self.expect_char('{')?;
                            blocks.push(Block::ExpectValue(self.parse_braced()?));
                        }
                    } else if kw == "test" || kw == "group" {
                        return Err(ParseError::UnexpectedToken {
                            position: self.pos,
                            expected: "@expect or code".into(),
                            got: format!("@{kw}"),
                        });
                    } else {
                        cur_input.push('@');
                        cur_input.push_str(&kw);
                    }
                }

                Some('"') => {
                    self.advance();
                    self.scan_string_literal(&mut cur_input)?;
                }

                Some('#') => {
                    self.advance();
                    cur_input.push('#');
                    self.scan_line_comment(&mut cur_input);
                }

                Some('/') if self.chars.get(self.pos + 1) == Some(&'/') => {
                    self.advance();
                    self.advance();
                    cur_input.push_str("//");
                    self.scan_line_comment(&mut cur_input);
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
                        if !blocks
                            .iter()
                            .any(|b| matches!(b, Block::ExpectValue(_) |Block::ExpectOutput(_) | Block::ExpectErr(_)))
                        {
                            return Err(ParseError::NoAssertions {
                                name: name.to_string(),
                            });
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

    /// Parse a sequence of `@set` / `@test` / `@group` items.
    /// If `is_toplevel`, reads until EOF; otherwise reads until balanced `}`.
    /// `@set(k="v")` annotations accumulate and are consumed by the next
    /// `@test` or `@group`, making them usable on both.
    fn parse_items_body(&mut self, is_toplevel: bool) -> ParseResult<Vec<SuiteItem>> {
        let mut items: Vec<SuiteItem> = Vec::new();
        let mut pending: HashMap<String, String> = HashMap::new();

        loop {
            self.skip_ws();
            match self.cur() {
                None if is_toplevel => break,
                None => {
                    return Err(ParseError::UnexpectedEof {
                        context: "inside @group body".into(),
                    });
                }

                Some('#') => self.skip_line(),

                Some('@') => {
                    self.advance();
                    let kw = self.parse_ident();
                    match kw.as_str() {
                        "set" => {
                            let (k, v) = self.parse_annotation_args()?;
                            pending.insert(k, v);
                        }
                        "test" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let blocks = self.parse_test_body(&name)?;
                            items.push(SuiteItem::Test(TestCase {
                                name,
                                flags: std::mem::take(&mut pending),
                                blocks,
                            }));
                        }
                        "group" => {
                            self.skip_ws();
                            let name = self.parse_ident();
                            self.expect_char('{')?;
                            let children = self.parse_items_body(false)?;
                            items.push(SuiteItem::Group {
                                name,
                                flags: std::mem::take(&mut pending),
                                children,
                            });
                        }
                        other => {
                            return Err(ParseError::UnexpectedToken {
                                position: self.pos,
                                expected: "set, test, or group".into(),
                                got: other.to_string(),
                            });
                        }
                    }
                }

                Some('}') if !is_toplevel => {
                    self.advance();
                    return Ok(items);
                }

                Some(c) => {
                    return Err(ParseError::UnexpectedToken {
                        position: self.pos,
                        expected: "@directive or EOF".into(),
                        got: c.to_string(),
                    });
                }
            }
        }

        Ok(items)
    }

    pub fn parse(&mut self) -> ParseResult<Vec<SuiteItem>> {
        self.parse_items_body(true)
    }
}

/// Recursively flatten a suite tree into a flat list of `TestCase`s, merging
/// flags from enclosing `@group`s into each test.
pub(crate) fn flatten_items(
    items: &[SuiteItem],
    inherited: &HashMap<String, String>,
    prefix: &str,
) -> Vec<TestCase> {
    items
        .iter()
        .flat_map(|item| match item {
            SuiteItem::Test(tc) => {
                let mut flags = inherited.clone();
                flags.extend(tc.flags.clone());
                let full_name = if prefix.is_empty() {
                    tc.name.clone()
                } else {
                    format!("{prefix}.{}", tc.name)
                };
                vec![TestCase {
                    name: full_name,
                    flags,
                    blocks: tc.blocks.clone(),
                }]
            }
            SuiteItem::Group {
                name,
                flags,
                children,
            } => {
                let mut merged = inherited.clone();
                merged.extend(flags.clone());
                let child_prefix = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}.{name}")
                };
                flatten_items(children, &merged, &child_prefix)
            }
        })
        .collect()
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
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim_start()
            }
        })
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
        flatten_items(
            &TestLangParser::new(src).parse().expect("parse failed"),
            &HashMap::new(),
            "",
        )
    }

    fn flags_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn dedent_strips_common_indent() {
        assert_eq!(
            dedent(
                "
    hello
    world
"
            ),
            "hello\nworld"
        );
    }

    #[test]
    fn dedent_preserves_relative_indent() {
        assert_eq!(
            dedent(
                "
    - outer
      - inner
"
            ),
            "- outer\n  - inner"
        );
    }

    #[test]
    fn dedent_handles_empty() {
        assert_eq!(dedent(""), "");
        assert_eq!(dedent("   "), "");
    }

    #[test]
    fn dedent_all_blank_lines() {
        assert_eq!(dedent("\n\n\n"), "");
    }

    #[test]
    fn dedent_single_non_blank_line() {
        assert_eq!(dedent("    hello"), "hello");
    }

    #[test]
    fn parse_test_with_expect() {
        let cases = flat(
            r#"
            @test greet {
                print("hello");
                @expect output {
                    hello
                }
            }
        "#,
        );
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "greet");
        assert!(matches!(&cases[0].blocks[0], Block::Input(_)));
        assert!(matches!(&cases[0].blocks[1], Block::ExpectOutput(_)));
    }

    #[test]
    fn parse_test_expect_only() {
        let items = parse_items("@test foo { @expect output { result } }");
        assert_eq!(
            items,
            vec![SuiteItem::Test(TestCase {
                name: "foo".into(),
                flags: HashMap::new(),
                blocks: vec![Block::ExpectOutput(" result ".into())],
            })]
        );
    }

    #[test]
    fn parse_test_with_expect_error() {
        let cases = flat(
            r#"
            @test bad {
                do_error();
                @expect error {
                    SomeError
                }
            }
        "#,
        );
        assert!(matches!(&cases[0].blocks[1], Block::ExpectErr(_)));
    }

    #[test]
    fn parse_test_interleaved_blocks() {
        let cases = flat(
            r#"
            @test multi {
                code1();
                @expect error { err1 }
                code2();
                @expect error { err2 }
            }
        "#,
        );
        assert_eq!(cases[0].blocks.len(), 4);
    }

    #[test]
    fn parse_test_only_expect_error_no_input() {
        let cases = flat("@test e { @expect error { boom } }");
        assert_eq!(cases[0].blocks.len(), 1);
        assert!(matches!(&cases[0].blocks[0], Block::ExpectErr(_)));
    }

    #[test]
    fn parse_nested_braces_in_code() {
        let cases = flat(
            r#"
            @test loops {
                for (var $i = 0; $i < 3; $i = $i + 1) {
                    print($i);
                }
                @expect output {
                    0
                    1
                    2
                }
            }
        "#,
        );
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains("for"));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn parse_multiple_top_level_tests() {
        let cases = flat(
            r#"
            @test first { a @expect output { 1 } }
            @test second { b @expect output { 2 } }
        "#,
        );
        assert_eq!(cases[0].name, "first");
        assert_eq!(cases[1].name, "second");
    }

    #[test]
    fn parse_top_level_comment() {
        let cases = flat(
            r#"
            # comment
            @test name { code @expect output { result } }
        "#,
        );
        assert_eq!(cases.len(), 1);
    }

    #[test]
    fn set_annotates_test_directly() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @test t { @expect output { x } }
        "#,
        );
        assert_eq!(cases[0].flags.get("run"), Some(&"plan".to_string()));
    }

    #[test]
    fn set_annotates_group_flags_inherited_by_tests() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @group suite {
                @test q1 { code @expect output { result } }
                @test q2 { code @expect output { result } }
            }
        "#,
        );
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].flags.get("run"), Some(&"plan".to_string()));
        assert_eq!(cases[1].flags.get("run"), Some(&"plan".to_string()));
    }

    #[test]
    fn set_inside_group_annotates_next_item_only() {
        let cases = flat(
            r#"
            @group suite {
                @set(run = "plan")
                @test q1 { code @expect output { result } }
                @test q2 { code @expect output { result } }
            }
        "#,
        );
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].flags.get("run"), Some(&"plan".to_string()));
        assert!(cases[1].flags.get("run").is_none());
    }

    #[test]
    fn set_does_not_bleed_to_next_item() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @test first { @expect output { x } }
            @test second { @expect output { y } }
        "#,
        );
        assert_eq!(cases[0].flags.get("run"), Some(&"plan".to_string()));
        assert!(cases[1].flags.get("run").is_none());
    }

    #[test]
    fn multiple_sets_accumulate() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @set(extra = "yes")
            @test t { @expect output { x } }
        "#,
        );
        assert_eq!(
            cases[0].flags,
            flags_map(&[("run", "plan"), ("extra", "yes")])
        );
    }

    #[test]
    fn nested_group_flags_merged() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @group outer {
                @set(extra = "yes")
                @group inner {
                    @test t { code @expect output { x } }
                }
            }
        "#,
        );
        assert_eq!(
            cases[0].flags,
            flags_map(&[("run", "plan"), ("extra", "yes")])
        );
    }

    #[test]
    fn nested_group_flag_override() {
        let cases = flat(
            r#"
            @set(run = "plan")
            @group outer {
                @set(run = "interpreter")
                @group inner {
                    @test t { code @expect output { x } }
                }
            }
        "#,
        );
        assert_eq!(cases[0].flags.get("run"), Some(&"interpreter".to_string()));
    }

    #[test]
    fn group_name_prefix_in_test_names() {
        let cases = flat(
            r#"
            @group outer {
                @group inner {
                    @test t { @expect output { x } }
                }
            }
        "#,
        );
        assert_eq!(cases[0].name, "outer.inner.t");
    }

    #[test]
    fn no_assertions_error() {
        assert!(matches!(
            TestLangParser::new("@test foo { some code }").parse(),
            Err(ParseError::NoAssertions { name }) if name == "foo"
        ));
    }

    #[test]
    fn no_assertions_error_empty_body() {
        assert!(matches!(
            TestLangParser::new("@test foo { }").parse(),
            Err(ParseError::NoAssertions { .. })
        ));
    }

    #[test]
    fn parse_error_test_nested_in_test() {
        assert!(matches!(
            TestLangParser::new("@test outer { @test inner { @expect output { x } } @expect output { y } }")
                .parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_group_nested_in_test() {
        assert!(matches!(
            TestLangParser::new("@test outer { @group g { } @expect output { y } }").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn unknown_expect_qualifier() {
        assert!(matches!(
            TestLangParser::new("@test t { @expect unknown { x } }").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_unexpected_token_at_top_level() {
        assert!(matches!(
            TestLangParser::new("garbage").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_unknown_directive() {
        assert!(matches!(
            TestLangParser::new("@unknown { }").parse(),
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
            TestLangParser::new("@test t { @expect output {").parse(),
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

    #[test]
    fn parse_error_eof_in_group_body() {
        assert!(matches!(
            TestLangParser::new("@group g { @test t { @expect output { x } }").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn parse_error_eof_in_annotation_value() {
        assert!(matches!(
            TestLangParser::new("@set(run = \"plan").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn parse_error_bad_char_in_group() {
        assert!(matches!(
            TestLangParser::new("@group g { bad }").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn parse_error_expect_char_eof() {
        assert!(matches!(
            TestLangParser::new("@test").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }

    #[test]
    fn parse_error_expect_char_wrong() {
        assert!(matches!(
            TestLangParser::new("@test t x").parse(),
            Err(ParseError::UnexpectedToken { .. })
        ));
    }

    #[test]
    fn at_sign_in_code_preserved() {
        let cases = flat("@test t { @notadirective @expect output { x } }");
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains("@notadirective"));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn flatten_empty_group() {
        let items = parse_items("@group empty {}");
        assert!(flatten_items(&items, &HashMap::new(), "").is_empty());
    }

    #[test]
    fn flatten_top_level_prefix_empty() {
        assert_eq!(flat("@test t { @expect output { x } }")[0].name, "t");
    }

    #[test]
    fn string_open_brace_not_counted() {
        // Unbalanced `{` inside a string should not confuse the depth counter.
        let cases = flat(
            r#"
            @test t {
                var $s = "open brace: {";
                @expect output { ok }
            }
        "#,
        );
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains(r#""open brace: {""#));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn string_close_brace_not_counted() {
        // Unbalanced `}` inside a string must not terminate the test body early.
        let cases = flat(
            r#"
            @test t {
                var $s = "close brace: }";
                @expect output { ok }
            }
        "#,
        );
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains(r#""close brace: }""#));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn string_balanced_braces_preserved() {
        let cases = flat(r#"@test t { var $s = "{}"; @expect output { ok } }"#);
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains(r#""{}""#));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn string_escaped_quote_with_brace_not_counted() {
        // `\"` inside a string must not end the literal, so the `{` that
        // follows it remains invisible to the depth counter.
        let cases = flat("@test t { var $s = \"say \\\"hello {\\\"\"; @expect output { ok } }");
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains('{'));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn hash_comment_braces_not_counted() {
        let cases = flat("@test t { code(); # { unbalanced brace\n @expect output { ok } }");
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains("# { unbalanced brace"));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn double_slash_comment_braces_not_counted() {
        let cases = flat("@test t { code(); // } unbalanced\n @expect output { ok } }");
        if let Block::Input(code) = &cases[0].blocks[0] {
            assert!(code.contains("// } unbalanced"));
        } else {
            panic!("expected Input block");
        }
    }

    #[test]
    fn parse_error_eof_in_string_literal_in_test_body() {
        assert!(matches!(
            TestLangParser::new("@test t { var $s = \"unterminated").parse(),
            Err(ParseError::UnexpectedEof { .. })
        ));
    }
}
