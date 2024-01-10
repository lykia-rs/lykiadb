#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    bool_false: {
        "false;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Bool": false
                        },
                        "raw": "false"
                    }
                }
            ]
        }
    },
    bool_true: {
        "true;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Bool": true
                        },
                        "raw": "true"
                    }
                }
            ]
        }
    },
    undef: {
        "undefined;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": "Undefined",
                        "raw": "undefined"
                    }
                }
            ]
        }
    },
    null: {
        "null;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": "Null",
                        "raw": "null"
                    }
                }
            ]
        }
    }
}
