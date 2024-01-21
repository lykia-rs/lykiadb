use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    one_plus_two: {
        "1 + 2;" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Expression",
                "expr": {
                  "@type": "Expr::Binary",
                  "left": {
                    "@type": "Expr::Literal",
                    "raw": "1",
                    "value": {
                      "Num": 1.0
                    }
                  },
                  "operation": {
                    "@type": "Add"
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "raw": "2",
                    "value": {
                      "Num": 2.0
                    }
                  }
                }
              }
            ]
          }
    }
}
