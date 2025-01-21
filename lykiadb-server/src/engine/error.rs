use std::fmt::{Display, Formatter, Result};

use crate::{plan::PlannerError, value::environment::EnvironmentError};

use super::interpreter::InterpretError;
use lykiadb_lang::{ast::Span, parser::ParseError, tokenizer::scanner::ScanError, LangError};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    Lang(LangError),
    Interpret(InterpretError),
    Environment(EnvironmentError),
    Plan(PlannerError),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub fn report_error(
    source_name: &str,
    source: &str,
    error: ExecutionError,
    mut writer: impl std::io::Write,
) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    // Generate & choose some colours for each of our elements
    let out = Color::Fixed(81);

    let mut print = |message: &str, hint: &str, span: Span| {
        Report::build(ReportKind::Error, source_name, 12)
            .with_code(3)
            .with_message(format!("{} at line {}", message, span.line + 1))
            .with_label(
                Label::new((source_name, span.start..span.end))
                    .with_message(hint)
                    .with_color(out),
            )
            .finish()
            .write((source_name, Source::from(&source)), &mut writer)
            .unwrap();
    };

    match error {
        ExecutionError::Lang(LangError::Scan(ScanError::UnexpectedCharacter { span })) => {
            print("Unexpected character", "Remove this character", span);
        }
        ExecutionError::Lang(LangError::Scan(ScanError::UnterminatedString { span })) => {
            print(
                "Unterminated string",
                "Terminate the string with a double quote (\").",
                span,
            );
        }
        ExecutionError::Lang(LangError::Scan(ScanError::MalformedNumberLiteral { span })) => {
            print(
                "Malformed number literal",
                "Make sure that number literal is up to spec.",
                span,
            );
        }
        ExecutionError::Lang(LangError::Parse(ParseError::MissingToken { token, expected })) => {
            print(
                "Missing token",
                &format!(
                    "Add a {:?} token after \"{}\".",
                    expected,
                    token.lexeme.unwrap()
                ),
                token.span,
            );
        }
        ExecutionError::Lang(LangError::Parse(ParseError::NoTokens)) => {
            print("There is nothing to parse", "", Span::default());
        }
        ExecutionError::Lang(LangError::Parse(ParseError::InvalidAssignmentTarget { left })) => {
            print(
                "Invalid assignment target",
                &format!("No values can be assigned to {}", left.lexeme.unwrap()),
                left.span,
            );
        }
        ExecutionError::Lang(LangError::Parse(ParseError::UnexpectedToken { token })) => {
            print(
                "Unexpected token",
                &format!(
                    "Unexpected token {}.",
                    token.lexeme.unwrap_or("None".to_string())
                ),
                token.span,
            );
        }
        ExecutionError::Interpret(InterpretError::ArityMismatch {
            span,
            expected,
            found,
        }) => {
            print(
                "Function arity mismatch",
                &format!(
                    "Function expects {} arguments, while provided {}.",
                    expected, found
                ),
                span,
            );
        }
        ExecutionError::Interpret(InterpretError::UnexpectedStatement { span }) => {
            print("Unexpected statement", "Remove this.", span);
        }
        ExecutionError::Interpret(InterpretError::NotCallable { span }) => {
            print(
                "Not callable",
                "Expression does not yield a callable.",
                span,
            );
        }
        ExecutionError::Interpret(InterpretError::PropertyNotFound { property, span }) => {
            print(
                &format!(
                    "Property {} not found in the evaluated expression",
                    property
                ),
                "Check if that field is present in the expression.",
                span,
            );
        }
        ExecutionError::Plan(PlannerError::DuplicateObjectInScope { previous, .. }) => {
            print(
                "Duplicate object in scope",
                &format!("Object {} is already defined in the scope.", previous.name),
                previous.span,
            );
        }
        ExecutionError::Plan(PlannerError::SubqueryNotAllowed(span)) => {
            print(
                "Subquery not allowed",
                "Subqueries are not allowed in this context.",
                span,
            );
        }
        ExecutionError::Environment(EnvironmentError::Other { message })
        | ExecutionError::Interpret(InterpretError::Other { message }) => {
            print(&message, "", Span::default());
        }
        _ => {}
    };
}
#[cfg(test)]
mod tests {

    use super::*;
    use lykiadb_lang::{
        ast::{Identifier, Literal},
        kw, sym,
        tokenizer::token::{Keyword, Symbol, Token, TokenType},
    };

    fn capture_error_output(filename: &str, source: &str, error: ExecutionError) -> String {
        let mut output = vec![];
        report_error(filename, source, error, &mut output);
        String::from_utf8(output).unwrap()
    }

    // Scanner Error Tests
    #[test]
    fn test_scanner_unterminated_string() {
        let source = r#"let x = "unterminated"#;
        let error = ExecutionError::Lang(LangError::Scan(ScanError::UnterminatedString {
            span: Span {
                start: 8,
                end: 21,
                line: 0,
                line_end: 0,
            },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Unterminated string"));
        assert!(output.contains("Terminate the string with a double quote"));
    }

    #[test]
    fn test_scanner_malformed_number() {
        let source = "let x = 123.456.789";
        let error = ExecutionError::Lang(LangError::Scan(ScanError::MalformedNumberLiteral {
            span: Span {
                start: 8,
                end: 19,
                line: 0,
                line_end: 0,
            },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Malformed number literal"));
        assert!(output.contains("Make sure that number literal is up to spec"));
    }

    // Parser Error Tests
    #[test]
    fn test_parser_missing_token() {
        let source = "var x = ";
        let error = ExecutionError::Lang(LangError::Parse(ParseError::MissingToken {
            token: Token {
                tok_type: kw!(Keyword::Var),
                lexeme: Some("var".to_string()),
                span: Span {
                    start: 0,
                    end: 3,
                    line: 0,
                    line_end: 0,
                },
                literal: None,
            },
            expected: TokenType::Identifier { dollar: true },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Missing token"));
        assert!(output.contains("Add a Identifier { dollar: true } token after \"var\"."));
    }

    #[test]
    fn test_parser_no_tokens() {
        let source = "";
        let error = ExecutionError::Lang(LangError::Parse(ParseError::NoTokens));

        let output = capture_error_output("test.txt", source, error);

        assert!(output.contains("There is nothing to parse"));
    }

    #[test]
    fn test_parser_unexpected_token() {
        let source = "let x = ;";
        let error = ExecutionError::Lang(LangError::Parse(ParseError::UnexpectedToken {
            token: Token {
                tok_type: sym!(Symbol::Semicolon),
                lexeme: Some(";".to_string()),
                span: Span {
                    start: 8,
                    end: 9,
                    line: 0,
                    line_end: 0,
                },
                literal: None,
            },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Unexpected token"));
        assert!(output.contains("Unexpected token ;"));
    }

    #[test]
    fn test_interpreter_unexpected_statement() {
        let source = "break;";
        let error = ExecutionError::Interpret(InterpretError::UnexpectedStatement {
            span: Span {
                start: 0,
                end: 5,
                line: 0,
                line_end: 0,
            },
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Unexpected statement"));
        assert!(output.contains("Remove this"));
    }

    #[test]
    fn test_parser_invalid_assignment() {
        let source = "5 = 10";
        let error = ExecutionError::Lang(LangError::Parse(ParseError::InvalidAssignmentTarget {
            left: Token {
                tok_type: TokenType::Num,
                lexeme: Some("5".to_string()),
                span: Span {
                    start: 0,
                    end: 1,
                    line: 0,
                    line_end: 0,
                },
                literal: Some(Literal::Num(5.0)),
            },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Invalid assignment target"));
        assert!(output.contains("No values can be assigned to 5"));
    }

    // Planner Error Tests
    #[test]
    fn test_planner_duplicate_object() {
        let source = "Select * from users, users;";
        let error = ExecutionError::Plan(PlannerError::DuplicateObjectInScope {
            previous: Identifier::new("users", false),
            ident: Identifier::new("users", false),
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Duplicate object in scope"));
        assert!(output.contains("Object users is already defined in the scope"));
    }

    #[test]
    fn test_planner_subquery_not_allowed() {
        let source = "SELECT * FROM users inner join orders on users.id = (SELECT id FROM users);";
        let error = ExecutionError::Plan(PlannerError::SubqueryNotAllowed(Span {
            start: 47,
            end: 70,
            line: 0,
            line_end: 0,
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Subquery not allowed"));
        assert!(output.contains("Subqueries are not allowed in this context"));
    }

    // Interpreter Error Tests
    #[test]
    fn test_interpreter_arity_mismatch() {
        let source = "function test(a, b) {}; test(1);";
        let error = ExecutionError::Interpret(InterpretError::ArityMismatch {
            span: Span {
                start: 24,
                end: 29,
                line: 0,
                line_end: 0,
            },
            expected: 2,
            found: 1,
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Function arity mismatch"));
        assert!(output.contains("Function expects 2 arguments, while provided 1"));
    }

    #[test]
    fn test_interpreter_not_callable() {
        let source = "let x = 5; x();";
        let error = ExecutionError::Interpret(InterpretError::NotCallable {
            span: Span {
                start: 12,
                end: 15,
                line: 0,
                line_end: 0,
            },
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Not callable"));
        assert!(output.contains("Expression does not yield a callable"));
    }

    // Environment Error Tests
    #[test]
    fn test_environment_variable_not_found() {
        let source = "io::print(undefined_var);";
        let error = ExecutionError::Environment(EnvironmentError::Other {
            message: "Variable 'undefined_var' not found in current scope".to_string(),
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Variable 'undefined_var' not found in current scope"));
    }

    #[test]
    fn test_scan_error_reporting() {
        let source = "let x = @";
        let error = ExecutionError::Lang(LangError::Scan(ScanError::UnexpectedCharacter {
            span: Span {
                start: 8,
                end: 9,
                line: 0,
                line_end: 0,
            },
        }));

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Unexpected character"));
        assert!(output.contains("Remove this character"));
        assert!(output.contains("at line 1"));
    }

    #[test]
    fn test_interpret_error_reporting() {
        let source = "obj.nonexistent";
        let error = ExecutionError::Interpret(InterpretError::PropertyNotFound {
            property: "nonexistent".to_string(),
            span: Span {
                start: 4,
                end: 14,
                line: 0,
                line_end: 0,
            },
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Property nonexistent not found"));
        assert!(output.contains("Check if that field is present"));
    }

    #[test]
    fn test_environment_error_reporting() {
        let source = "";
        let error = ExecutionError::Environment(EnvironmentError::Other {
            message: "Variable not found".to_string(),
        });

        let output = capture_error_output("test.txt", source, error);
        assert!(output.contains("Variable not found"));
    }
}
