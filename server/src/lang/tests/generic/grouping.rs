#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    group_0: {
        "(1 + 2) * (3 / (4 - 7));" =>         {
            "@type": "Stmt::Program",
            "body": [
              {
                "@type": "Stmt::Expression",
                "expr": {
                  "@type": "Expr::Binary",
                  "left": {
                    "@type": "Expr::Grouping",
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
                  },
                  "operation": {
                    "@type": "Multiply"
                  },
                  "right": {
                    "@type": "Expr::Grouping",
                    "expr": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Literal",
                        "raw": "3",
                        "value": {
                          "Num": 3.0
                        }
                      },
                      "operation": {
                        "@type": "Divide"
                      },
                      "right": {
                        "@type": "Expr::Grouping",
                        "expr": {
                          "@type": "Expr::Binary",
                          "left": {
                            "@type": "Expr::Literal",
                            "raw": "4",
                            "value": {
                              "Num": 4.0
                            }
                          },
                          "operation": {
                            "@type": "Subtract"
                          },
                          "right": {
                            "@type": "Expr::Literal",
                            "raw": "7",
                            "value": {
                              "Num": 7.0
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
