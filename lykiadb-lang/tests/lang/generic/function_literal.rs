use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    simple: {
        "function $a ($x: dtype::num, $y: dtype::num) {};" => {
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
                      {
                        "@type": "TypeAnnotation",
                        "type_expr": {
                          "@type": "Expr::Get",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "num"
                          },
                          "object": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "dtype"
                            }
                          }
                        }
                      }
                    ],
                    [
                      {
                        "@type": "Identifier",
                        "dollar": true,
                        "name": "$y"
                      },
                      {
                        "@type": "TypeAnnotation",
                        "type_expr": {
                          "@type": "Expr::Get",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "num"
                          },
                          "object": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "dtype"
                            }
                          }
                        }
                      }
                    ]
                  ],
                  "return_type": {
                    "@type": "TypeAnnotation",
                    "type_expr": {
                      "@type": "Expr::Get",
                      "name": {
                        "@type": "Identifier",
                        "dollar": false,
                        "name": "unit"
                      },
                      "object": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "dtype"
                        }
                      }
                    }
                  }
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
                "return_type": {
                  "@type": "TypeAnnotation",
                  "type_expr": {
                    "@type": "Expr::Call",
                    "args": [
                      {
                        "@type": "Expr::Get",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "unit"
                        },
                        "object": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "dtype"
                          }
                        }
                      },
                      {
                        "@type": "Expr::Get",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "unit"
                        },
                        "object": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "dtype"
                          }
                        }
                      }
                    ],
                    "callee": {
                      "@type": "Expr::Get",
                      "name": {
                        "@type": "Identifier",
                        "dollar": false,
                        "name": "callable"
                      },
                      "object": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "dtype"
                        }
                      }
                    }
                  }
                }
              }
            }
          ]
      }
  }
}
