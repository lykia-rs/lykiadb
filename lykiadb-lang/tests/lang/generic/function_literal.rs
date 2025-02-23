use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    simple: {
        "function $a ($x, $y) {};" => {
            "@type": "Stmt::Program",
            "body":         [
              {
                "@type": "Stmt::Expression",
                "expr": {
                  "@type": "Expr::Function",
                  "body": [],
                  "name": {
                    "@type": "Identifier",
                    "dollar": true,
                    "name": "$a"
                  },
                  "parameters": [
                    [
                      {
                        "@type": "Identifier",
                        "dollar": true,
                        "name": "$x"
                      },
                      null
                    ],
                    [
                      {
                        "@type": "Identifier",
                        "dollar": true,
                        "name": "$y"
                      },
                      null
                    ]
                  ],
                  "return_type": null
                }
              }
            ]
          }
    },
    hof: {
      "function $make_counter() {};" => {
          "@type": "Stmt::Program",
          "body":         [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Function",
                "body": [],
                "name": {
                  "@type": "Identifier",
                  "dollar": true,
                  "name": "$make_counter"
                },
                "parameters": [],
                "return_type": null
              }
            }
          ]
      }
  }
}
