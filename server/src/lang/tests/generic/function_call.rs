#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    print_50: {
        "print(50);" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Call",
                        "callee": {
                            "type": "Expr::Variable",
                            "name": "print"
                        },
                        "args": [
                            {
                                "type": "Expr::Literal",
                                "value": "Num(50.0)",
                                "raw": "50"
                            }
                        ]
                    }
                }
            ]
        }
    }
}
