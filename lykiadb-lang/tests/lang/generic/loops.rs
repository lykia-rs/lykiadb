use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    loop_0: {
        "loop {}" =>  {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Loop",
                "body": {
                  "@type": "Stmt::Block",
                  "body": []
                },
                "condition": null,
                "post": null
              }
            ]
        }
    },
    loop_1: {
        "loop { print(1); }" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Loop",
                "body": {
                  "@type": "Stmt::Block",
                  "body": [
                    {
                      "@type": "Stmt::Expression",
                      "expr": {
                        "@type": "Expr::Call",
                        "args": [
                          {
                            "@type": "Expr::Literal",
                            "raw": "1",
                            "value": {
                              "Num": 1.0
                            }
                          }
                        ],
                        "callee": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "print"
                          }
                        }
                      }
                    }
                  ]
                },
                "condition": null,
                "post": null
              }
            ]
        }
    },
    for_0: {
        "for (var $i = 0; $i < 10; $i = $i + 1) {}" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Block",
                "body": [
                  {
                    "@type": "Stmt::Declaration",
                    "dst": {
                      "@type": "Identifier",
                      "kind": "IdentifierKind::ForcedVariable",
                      "name": "$i"
                    },
                    "expr": {
                      "@type": "Expr::Literal",
                      "raw": "0",
                      "value": {
                        "Num": 0.0
                      }
                    }
                  },
                  {
                    "@type": "Stmt::Loop",
                    "body": {
                      "@type": "Stmt::Block",
                      "body": []
                    },
                    "condition": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::ForcedVariable",
                          "name": "$i"
                        }
                      },
                      "operation": {
                        "@type": "Less"
                      },
                      "right": {
                        "@type": "Expr::Literal",
                        "raw": "10",
                        "value": {
                          "Num": 10.0
                        }
                      }
                    },
                    "post": {
                      "@type": "Stmt::Expression",
                      "expr": {
                        "@type": "Expr::Assignment",
                        "dst": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::ForcedVariable",
                          "name": "$i"
                        },
                        "expr": {
                          "@type": "Expr::Binary",
                          "left": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::ForcedVariable",
                              "name": "$i"
                            }
                          },
                          "operation": {
                            "@type": "Add"
                          },
                          "right": {
                            "@type": "Expr::Literal",
                            "raw": "1",
                            "value": {
                              "Num": 1.0
                            }
                          }
                        }
                      }
                    }
                  }
                ]
              }
            ]
          }
    },
    for_1: {
        "for (var $i = 0; $i < 10; $i = $i + 1) { print($i); }" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Block",
                "body": [
                  {
                    "@type": "Stmt::Declaration",
                    "dst": {
                      "@type": "Identifier",
                      "kind": "IdentifierKind::ForcedVariable",
                      "name": "$i"
                    },
                    "expr": {
                      "@type": "Expr::Literal",
                      "raw": "0",
                      "value": {
                        "Num": 0.0
                      }
                    }
                  },
                  {
                    "@type": "Stmt::Loop",
                    "body": {
                      "@type": "Stmt::Block",
                      "body": [
                        {
                          "@type": "Stmt::Expression",
                          "expr": {
                            "@type": "Expr::Call",
                            "args": [
                              {
                                "@type": "Expr::Variable",
                                "name": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::ForcedVariable",
                                  "name": "$i"
                                }
                              }
                            ],
                            "callee": {
                              "@type": "Expr::Variable",
                              "name": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "print"
                              }
                            }
                          }
                        }
                      ]
                    },
                    "condition": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::ForcedVariable",
                          "name": "$i"
                        }
                      },
                      "operation": {
                        "@type": "Less"
                      },
                      "right": {
                        "@type": "Expr::Literal",
                        "raw": "10",
                        "value": {
                          "Num": 10.0
                        }
                      }
                    },
                    "post": {
                      "@type": "Stmt::Expression",
                      "expr": {
                        "@type": "Expr::Assignment",
                        "dst": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::ForcedVariable",
                          "name": "$i"
                        },
                        "expr": {
                          "@type": "Expr::Binary",
                          "left": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::ForcedVariable",
                              "name": "$i"
                            }
                          },
                          "operation": {
                            "@type": "Add"
                          },
                          "right": {
                            "@type": "Expr::Literal",
                            "raw": "1",
                            "value": {
                              "Num": 1.0
                            }
                          }
                        }
                      }
                    }
                  }
                ]
              }
            ]
          }
    },
    for_infinite: {
        "for (;;) {}" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Loop",
                "body": {
                  "@type": "Stmt::Block",
                  "body": []
                },
                "condition": null,
                "post": null
              }
            ]
          }
    },
    while_infinite: {
        "while (true) {}" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Loop",
                "body": {
                  "@type": "Stmt::Block",
                  "body": []
                },
                "condition": {
                  "@type": "Expr::Literal",
                  "raw": "true",
                  "value": {
                    "Bool": true
                  }
                },
                "post": null
              }
            ]
          }
    }
}
