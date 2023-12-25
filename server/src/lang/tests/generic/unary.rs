#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    minus_1: {
        "-1;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Unary",
                        "operation": "Subtract",
                        "expr": {
                            "type": "Expr::Literal",
                            "value": "Num(1.0)",
                            "raw": "1"
                        }
                    }
                }
            ]
        }
    }
}
