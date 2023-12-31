#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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

    collection: {
        "SELECT users.* from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": "users"
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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
    mixed_0: {
        "SELECT id, users.name as username from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "id"
                                    },
                                    "alias": null
                                },
                                {
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Get",
                                        "object": {
                                            "type": "Expr::Variable",
                                            "name": "users"
                                        },
                                        "name": "name"
                                    },
                                    "alias": "username"
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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
    mixed_1: {
        "SELECT 5 as five, \"text\" as some_text from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Literal",
                                        "value": "Num(5.0)",
                                        "raw": "5"
                                    },
                                    "alias": "five"
                                },
                                {
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Literal",
                                        "value": "Str(\"text\")",
                                        "raw": "text"
                                    },
                                    "alias": "some_text"
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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
    mixed_2: {
        "SELECT 5 + 27 as addition, 4 / 2 as division from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(5.0)",
                                            "raw": "5"
                                        },
                                        "operation": "Add",
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(27.0)",
                                            "raw": "27"
                                        }
                                    },
                                    "alias": "addition"
                                },
                                {
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(4.0)",
                                            "raw": "4"
                                        },
                                        "operation": "Divide",
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(2.0)",
                                            "raw": "2"
                                        }
                                    },
                                    "alias": "division"
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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
    mixed_no_from: {
        "SELECT 5 + 27 as addition, 4 / 2 as division;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": null,
                                "projection": [{
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(5.0)",
                                            "raw": "5"
                                        },
                                        "operation": "Add",
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(27.0)",
                                            "raw": "27"
                                        }
                                    },
                                    "alias": "addition"
                                },
                                {
                                    "type": "Expr",
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(4.0)",
                                            "raw": "4"
                                        },
                                        "operation": "Divide",
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(2.0)",
                                            "raw": "2"
                                        }
                                    },
                                    "alias": "division"
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
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
