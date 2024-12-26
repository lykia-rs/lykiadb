use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

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
    }
}
