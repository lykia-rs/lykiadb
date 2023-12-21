#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    one_plus_two: {
        "1 + 2;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Binary",
                        "left": {
                            "type": "Expr::Literal",
                            "value": "Num(1.0)",
                            "raw": "1"
                        },
                        "operator": {
                            "Symbol": "Plus"
                        },
                        "right": {
                            "type": "Expr::Literal",
                            "value": "Num(2.0)",
                            "raw": "2"
                        }
                    }
                }
            ]
        }
    }
}
