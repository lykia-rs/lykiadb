#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    limit_5: {
        "SELECT * from users limit 5;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "limit": {
                                "count": {
                                    "type": "Expr::Literal",
                                    "value": "Num(5.0)",
                                    "raw": "5"
                                },
                                "offset": null
                            },
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    limit_5_offset_10: {
        "SELECT * from users limit 5 offset 10;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "limit": {
                                "count": {
                                    "type": "Expr::Literal",
                                    "value": "Num(5.0)",
                                    "raw": "5"
                                },
                                "offset": {
                                    "type": "Expr::Literal",
                                    "value": "Num(10.0)",
                                    "raw": "10"
                                },
                            },
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    limit_10_offset_5: {
        "SELECT * from users limit 5, 10;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "limit": {
                                "count": {
                                    "type": "Expr::Literal",
                                    "value": "Num(10.0)",
                                    "raw": "10"
                                },
                                "offset": {
                                    "type": "Expr::Literal",
                                    "value": "Num(5.0)",
                                    "raw": "5"
                                }
                            },
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}
