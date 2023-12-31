#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    loop_0: {
        "loop {}" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Loop",
                    "condition": null,
                    "post": null,
                    "body": {
                        "type": "Stmt::Block",
                        "body": [],
                    }
                }
            ]
        }
    },
    loop_1: {
        "loop { print(1); }" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Loop",
                    "condition": null,
                    "post": null,
                    "body": {
                        "type": "Stmt::Block",
                        "body": [
                            {
                                "type": "Stmt::Expression",
                                "expr": {
                                    "type": "Expr::Call",
                                    "callee": {
                                        "type": "Expr::Variable",
                                        "name": "print"
                                    },
                                    "args": [
                                        {
                                            "type": "Expr::Literal",
                                            "value": "Num(1.0)",
                                            "raw": "1"
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                }
            ]
        }
    },
    for_0: {
        "for (var $i = 0; $i < 10; $i = $i + 1) {}" => {
            "type": "Stmt::Program",
            "body":         [
                {
                  "type": "Stmt::Block",
                  "body": [
                    {
                        "type": "Stmt::Declaration",
                        "variable": "$i",
                        "expr": {
                            "raw": "0",
                            "type": "Expr::Literal",
                            "value": "Num(0.0)"
                        },
                    },
                    {
                      "type": "Stmt::Loop",
                      "body": {
                        "type": "Stmt::Block",
                        "body": [],
                      },
                      "condition": {
                        "left": {
                            "name": "$i",
                            "type": "Expr::Variable"
                        },
                        "operation": "Less",
                        "right": {
                            "raw": "10",
                            "type": "Expr::Literal",
                            "value": "Num(10.0)"
                        },
                        "type": "Expr::Binary"
                      },
                      "post": {
                        "type": "Stmt::Expression",
                        "expr": {
                          "dst": "$i",
                          "type": "Expr::Assignment",
                          "expr": {
                            "type": "Expr::Binary",
                            "left": {
                                "name": "$i",
                                "type": "Expr::Variable"
                            },
                            "operation": "Add",
                            "right": {
                                "raw": "1",
                                "type": "Expr::Literal",
                                "value": "Num(1.0)"
                            },
                          },
                        },
                      },
                    }
                  ]
                }
            ]
        }
    },
    for_1: {
        "for (var $i = 0; $i < 10; $i = $i + 1) { print($i); }" => {
            "type": "Stmt::Program",
            "body": [
                {
                  "type": "Stmt::Block",
                  "body": [
                    {
                        "type": "Stmt::Declaration",
                        "variable": "$i",
                        "expr": {
                            "raw": "0",
                            "type": "Expr::Literal",
                            "value": "Num(0.0)"
                        },
                    },
                    {
                      "type": "Stmt::Loop",
                      "body": {
                        "type": "Stmt::Block",
                        "body": [
                            {
                                "type": "Stmt::Expression",
                                "expr": {
                                    "type": "Expr::Call",
                                    "callee": {
                                        "type": "Expr::Variable",
                                        "name": "print"
                                    },
                                    "args": [
                                        {
                                            "type": "Expr::Variable",
                                            "name": "$i"
                                        }
                                    ]
                                }
                            }
                        ],
                      },
                      "condition": {
                        "left": {
                            "name": "$i",
                            "type": "Expr::Variable"
                        },
                        "operation": "Less",
                        "right": {
                            "raw": "10",
                            "type": "Expr::Literal",
                            "value": "Num(10.0)"
                        },
                        "type": "Expr::Binary"
                      },
                      "post": {
                        "type": "Stmt::Expression",
                        "expr": {
                          "dst": "$i",
                          "type": "Expr::Assignment",
                          "expr": {
                            "type": "Expr::Binary",
                            "left": {
                                "name": "$i",
                                "type": "Expr::Variable"
                            },
                            "operation": "Add",
                            "right": {
                                "raw": "1",
                                "type": "Expr::Literal",
                                "value": "Num(1.0)"
                            },
                          },
                        },
                      },
                    }
                  ]
                }
            ]
        }
    },
    for_infinite: {
        "for (;;) {}" => {
            "type": "Stmt::Program",
            "body":  [
                {
                    "type": "Stmt::Loop",
                    "body": {
                        "type": "Stmt::Block",
                        "body": [],
                    },
                    "condition": null,
                    "post": null,
                }
            ]
        }
    },
    while_infinite: {
        "while (true) {}" => {
            "type": "Stmt::Program",
            "body":  [
                {
                    "type": "Stmt::Loop",
                    "body": {
                        "type": "Stmt::Block",
                        "body": [],
                    },
                    "condition": {
                        "type": "Expr::Literal",
                        "value": "Bool(true)",
                        "raw": "true"
                    },
                    "post": null,
                }
            ]
        }
    }
}
