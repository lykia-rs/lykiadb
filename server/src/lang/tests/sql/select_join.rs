#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users join orders;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
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
                                        "constraint": null
                                    }]
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }]
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
    constraint: {
        "SELECT * from users inner join orders on users.id = orders.user_id inner join order_items on orders.id = carts.order_id;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
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
                                            "name": "order_items",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "order_id",
                                              "object": {
                                                "name": "carts",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    }]
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }]
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
