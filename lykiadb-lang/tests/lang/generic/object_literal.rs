use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    empty_expr: {
        "{};" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {}
                }
              }
            }
          ]
        }
    },
    empty_declare: {
        "var $obj = {};" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Declaration",
              "dst": {
                "@type": "Identifier",
                "dollar": true,
                "name": "$obj"
              },
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {}
                }
              }
            }
          ]
        }
    },
    plain_expr: {
        "{ a : 1, b : `q` };" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {
                    "a": {
                      "@type": "Expr::Literal",
                      "raw": "1",
                      "value": {
                        "Num": 1.0
                      }
                    },
                    "b": {
                      "@type": "Expr::Literal",
                      "raw": "q",
                      "value": {
                        "Str": "q"
                      }
                    }
                  }
                }
              }
            }
          ]
        }
    },
    plain_declare: {
        "var $obj = { a : 1, b : `q` };" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Declaration",
              "dst": {
                "@type": "Identifier",
                "dollar": true,
                "name": "$obj"
              },
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {
                    "a": {
                      "@type": "Expr::Literal",
                      "raw": "1",
                      "value": {
                        "Num": 1.0
                      }
                    },
                    "b": {
                      "@type": "Expr::Literal",
                      "raw": "q",
                      "value": {
                        "Str": "q"
                      }
                    }
                  }
                }
              }
            }
          ]
        }
    },
    plain_expr_literal_keys: {
        "{ 'a': 1, b : `q`, 5: `d` };" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {
                    "a": {
                      "@type": "Expr::Literal",
                      "raw": "1",
                      "value": {
                        "Num": 1.0
                      }
                    },
                    "b": {
                      "@type": "Expr::Literal",
                      "raw": "q",
                      "value": {
                        "Str": "q"
                      }
                    },
                    "5": {
                      "@type": "Expr::Literal",
                      "raw": "d",
                      "value": {
                        "Str": "d"
                      }
                    }
                  }
                }
              }
            }
          ]
        }
    },
    some_nesting: {
        "{ 'a b c' : 1, b : { c : 2, d : 3 } };" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Literal",
                "raw": "",
                "value": {
                  "Object": {
                    "a b c": {
                      "@type": "Expr::Literal",
                      "raw": "1",
                      "value": {
                        "Num": 1.0
                      }
                    },
                    "b": {
                      "@type": "Expr::Literal",
                      "raw": "",
                      "value": {
                        "Object": {
                          "c": {
                            "@type": "Expr::Literal",
                            "raw": "2",
                            "value": {
                              "Num": 2.0
                            }
                          },
                          "d": {
                            "@type": "Expr::Literal",
                            "raw": "3",
                            "value": {
                              "Num": 3.0
                            }
                          }
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
