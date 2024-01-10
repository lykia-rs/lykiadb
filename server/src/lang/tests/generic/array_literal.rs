#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    empty_expr: {
        "[];" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Expression",
                "expr": {
                  "@type": "Expr::Literal",
                  "raw": "",
                  "value": {
                    "Array": []
                  }
                }
              }
            ]
          }
    },
    empty_declare: {
        "var $arr = [];" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Declaration",
                "dst": {
                  "@type": "Identifier",
                  "dollar": true,
                  "name": "$arr"
                },
                "expr": {
                  "@type": "Expr::Literal",
                  "raw": "",
                  "value": {
                    "Array": []
                  }
                }
              }
            ]
          }
    },
    plain_declare: {
        "var $arr = [1, 'abc', { \\key: `val` }];" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Declaration",
                "dst": {
                  "@type": "Identifier",
                  "dollar": true,
                  "name": "$arr"
                },
                "expr": {
                  "@type": "Expr::Literal",
                  "raw": "",
                  "value": {
                    "Array": [
                      {
                        "@type": "Expr::Literal",
                        "raw": "1",
                        "value": {
                          "Num": 1.0
                        }
                      },
                      {
                        "@type": "Expr::Literal",
                        "raw": "abc",
                        "value": {
                          "Str": "abc"
                        }
                      },
                      {
                        "@type": "Expr::Literal",
                        "raw": "",
                        "value": {
                          "Object": {
                            "key": {
                              "@type": "Expr::Literal",
                              "raw": "val",
                              "value": {
                                "Str": "val"
                              }
                            }
                          }
                        }
                      }
                    ]
                  }
                }
              }
            ]
          }
    },
    two_dimensional: {
        "var $arr = [[1, 2], [3, 4]];" => {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Declaration",
                "dst": {
                  "@type": "Identifier",
                  "dollar": true,
                  "name": "$arr"
                },
                "expr": {
                  "@type": "Expr::Literal",
                  "raw": "",
                  "value": {
                    "Array": [
                      {
                        "@type": "Expr::Literal",
                        "raw": "",
                        "value": {
                          "Array": [
                            {
                              "@type": "Expr::Literal",
                              "raw": "1",
                              "value": {
                                "Num": 1.0
                              }
                            },
                            {
                              "@type": "Expr::Literal",
                              "raw": "2",
                              "value": {
                                "Num": 2.0
                              }
                            }
                          ]
                        }
                      },
                      {
                        "@type": "Expr::Literal",
                        "raw": "",
                        "value": {
                          "Array": [
                            {
                              "@type": "Expr::Literal",
                              "raw": "3",
                              "value": {
                                "Num": 3.0
                              }
                            },
                            {
                              "@type": "Expr::Literal",
                              "raw": "4",
                              "value": {
                                "Num": 4.0
                              }
                            }
                          ]
                        }
                      }
                    ]
                  }
                }
              }
            ]
          }
    }
}
