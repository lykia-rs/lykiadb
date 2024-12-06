use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    print_50: {
        "$hello(50);" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Expression",
                "expr": {
                  "@type": "Expr::Call",
                  "args": [
                    {
                      "@type": "Expr::Literal",
                      "raw": "50",
                      "value": {
                        "Num": 50.0
                      }
                    }
                  ],
                  "callee": {
                    "@type": "Expr::Variable",
                    "name": {
                      "@type": "Identifier",
                      "dollar": true,
                      "name": "$hello"
                    }
                  }
                }
              }
            ]
          }
    }
}
