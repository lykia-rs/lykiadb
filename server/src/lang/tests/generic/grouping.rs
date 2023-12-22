#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    group_0: {
        "(1 + 2) * (3 / (4 - 7));" =>
        {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Binary",
                        "operation": "Multiply",
                        "left": {
                            "type": "Expr::Grouping",
                            "expr": {
                                "type": "Expr::Binary",
                                "operation": "Add",
                                "left": {
                                    "raw": "1",
                                    "type": "Expr::Literal",
                                    "value": "Num(1.0)"
                                },
                                "right": {
                                    "raw": "2",
                                    "type": "Expr::Literal",
                                    "value": "Num(2.0)"
                                }
                            }
                        },
                        "right": {
                            "type": "Expr::Grouping",
                            "expr": {
                                "type": "Expr::Binary",
                                "operation": "Divide",
                                "left": {
                                    "raw": "3",
                                    "type": "Expr::Literal",
                                    "value": "Num(3.0)"
                                },
                                "right": {
                                    "type": "Expr::Grouping",
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "operation": "Subtract",
                                        "left": {
                                            "raw": "4",
                                            "type": "Expr::Literal",
                                            "value": "Num(4.0)"
                                        },
                                        "right": {
                                            "raw": "7",
                                            "type": "Expr::Literal",
                                            "value": "Num(7.0)"
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
