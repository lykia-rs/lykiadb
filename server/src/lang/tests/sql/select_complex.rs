#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    single_union: {
        "SELECT * FROM a LEFT JOIN b ON b.num = a.num and b.value = \"abc\";" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "a",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Left",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "b",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Logical",
                                            "operation": "And",
                                            "left":  {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                  "name": "num",
                                                  "object": {
                                                    "name": "b",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                                "right": {
                                                  "name": "num",
                                                  "object": {
                                                    "name": "a",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                            },
                                            "right": {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                  "name": "value",
                                                  "object": {
                                                    "name": "b",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Str(\"abc\")",
                                                    "raw": "abc",
                                                },
                                            },
                                          }
                                    }]
                                },
                                "where": null
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
    triple_join: {
        "select * from users inner join orders on users.id = orders.user_id inner join baskets on baskets.order_id = orders.id;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "users",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "orders",
                                            "namespace": null,
                                        },
                                        "constraint":         {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "id",
                                              "object": {
                                                "name": "users",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "user_id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    },{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "baskets",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "order_id",
                                              "object": {
                                                "name": "baskets",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    }]
                                },
                                "where": null
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
