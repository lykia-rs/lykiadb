#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    declare_a: {
        "var $a;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$a",
                    "expr": {
                        "type": "Expr::Literal",
                        "value": "Undefined",
                        "raw": "undefined"
                    }
                }
            ]
        }
    },
    var_a: {
        "$a;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Variable",
                        "name": "$a"
                    }
                }
            ]
        }
    }
}
