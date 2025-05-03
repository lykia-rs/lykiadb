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
                    "kind": "IdentifierKind::Variable",
                    "name": "$a"
                  },
                  "parameters": [
                    [
                      {
                        "@type": "Identifier",
                        "kind": "IdentifierKind::Variable",
                        "name": "$x"
                      },
                      null
                    ],
                    [
                      {
                        "@type": "Identifier",
                        "kind": "IdentifierKind::Variable",
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
                  "kind": "IdentifierKind::Variable",
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
