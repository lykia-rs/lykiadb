#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    empty_expr: {
        "var $arr = [];" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$arr",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Array",
                            "value": []
                        }
                    }
                }
            ]
        }
    },
    empty_declare: {
        "var $arr = [];" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$arr",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Array",
                            "value": []
                        }
                    }
                }
            ]
        }
    },
    plain_declare: {
        "var $arr = [1, 'abc', { \\key: `val` }];" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$arr",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Array",
                            "value": [
                                {
                                    "type": "Expr::Literal",
                                    "value": "Num(1.0)",
                                    "raw": "1",
                                },
                                {
                                    "type": "Expr::Literal",
                                    "value": "Str(\"abc\")",
                                    "raw": "abc",
                                },
                                {
                                  "type": "Expr::Literal",
                                  "raw": "",
                                  "value": {
                                      "type": "Object",
                                      "value": [
                                          {
                                            "key": "key",
                                            "value": {
                                              "type": "Expr::Literal",
                                              "value": "Str(\"val\")",
                                              "raw": "val",
                                            }
                                          },
                                      ]
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
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$arr",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Array",
                            "value": [
                                {
                                    "type": "Expr::Literal",
                                    "raw": "",
                                    "value": {
                                        "type": "Array",
                                        "value": [
                                            {
                                                "type": "Expr::Literal",
                                                "value": "Num(1.0)",
                                                "raw": "1",
                                            },
                                            {
                                                "type": "Expr::Literal",
                                                "value": "Num(2.0)",
                                                "raw": "2",
                                            }
                                        ]
                                    }
                                },
                                {
                                    "type": "Expr::Literal",
                                    "raw": "",
                                    "value": {
                                        "type": "Array",
                                        "value": [
                                            {
                                                "type": "Expr::Literal",
                                                "value": "Num(3.0)",
                                                "raw": "3",
                                            },
                                            {
                                                "type": "Expr::Literal",
                                                "value": "Num(4.0)",
                                                "raw": "4",
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
