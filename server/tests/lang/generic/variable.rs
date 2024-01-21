use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    declare_a: {
        "var $a;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Declaration",
                    "dst": {
                        "@type": "Identifier",
                        "dollar": true,
                        "name": "$a"
                    },
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": "Undefined",
                        "raw": "undefined"
                    }
                }
            ]
        }
    },
    var_a: {
        "$a;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                            "@type": "Identifier",
                            "dollar": true,
                            "name": "$a"
                        }
                    }
                }
            ]
        }
    }
}
