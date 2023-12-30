#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users where id = 1;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "where": {
                                    "type": "Expr::Binary",
                                    "operation": "IsEqual",
                                    "left": {
                                        "type": "Expr::Variable",
                                        "name": "id",
                                    },
                                    "right": {
                                        "type": "Expr::Literal",
                                        "value": "Num(1.0)",
                                        "raw": "1"
                                    }
                                }
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    multi_0: {
        "SELECT * from users where id > 100 and name = 'John';" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "where": {
                                    "type": "Expr::Logical",
                                    "operation": "And",
                                    "left": {
                                        "type": "Expr::Binary",
                                        "operation": "Greater",
                                        "left": {
                                            "type": "Expr::Variable",
                                            "name": "id",
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(100.0)",
                                            "raw": "100"
                                        }
                                    },
                                    "right": {
                                        "type": "Expr::Binary",
                                        "operation": "IsEqual",
                                        "left": {
                                            "type": "Expr::Variable",
                                            "name": "name",
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Str(\"John\")",
                                            "raw": "John"
                                        }
                                    }
                                }
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    multi_1: {
        "SELECT * from users where (id > 100 and name = 'John') or (id < 10 and name = 'Jane');" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "where": {
                                    "type": "Expr::Logical",
                                    "operation": "Or",
                                    "left": {
                                        "type": "Expr::Grouping",
                                        "expr": {
                                            "type": "Expr::Logical",
                                            "operation": "And",
                                            "left": {
                                                "type": "Expr::Binary",
                                                "operation": "Greater",
                                                "left": {
                                                    "type": "Expr::Variable",
                                                    "name": "id",
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Num(100.0)",
                                                    "raw": "100"
                                                }
                                            },
                                            "right": {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                    "type": "Expr::Variable",
                                                    "name": "name",
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Str(\"John\")",
                                                    "raw": "John"
                                                }
                                            }
                                        }
                                    },
                                    "right": {
                                        "type": "Expr::Grouping",
                                        "expr": {
                                            "type": "Expr::Logical",
                                            "operation": "And",
                                            "left": {
                                                "type": "Expr::Binary",
                                                "operation": "Less",
                                                "left": {
                                                    "type": "Expr::Variable",
                                                    "name": "id",
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Num(10.0)",
                                                    "raw": "10"
                                                }
                                            },
                                            "right": {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                    "type": "Expr::Variable",
                                                    "name": "name",
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Str(\"Jane\")",
                                                    "raw": "Jane"
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}
