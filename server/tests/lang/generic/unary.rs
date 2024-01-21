use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    minus_1: {
        "-1;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Unary",
                        "operation": {
                            "@type": "Subtract"
                        },
                        "expr": {
                            "@type": "Expr::Literal",
                            "value": {
                                "Num": 1.0
                            },
                            "raw": "1"
                        }
                    }
                }
            ]
        }
    },
    not_true: {
        "!true;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Unary",
                        "operation": {
                            "@type": "Not"
                        },
                        "expr": {
                            "@type": "Expr::Literal",
                            "value": {
                                "Bool": true
                            },
                            "raw": "true"
                        }
                    }
                }
            ]
        }
    }
}
