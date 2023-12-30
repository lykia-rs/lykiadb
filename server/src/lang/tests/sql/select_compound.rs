#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    single_union: {
        "SELECT * FROM users UNION SELECT * FROM users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
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
                                "where": null
                            },
                            "compound": [{
                                "operation": "Union",
                                "core": {
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
                                    "where": null
                                },
                            }],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    single_intersect: {
        "SELECT * FROM users INTERSECT SELECT * FROM users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
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
                                "where": null
                            },
                            "compound": [{
                                "operation": "Intersect",
                                "core": {
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
                                    "where": null
                                },
                            }],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    single_except: {
        "SELECT * FROM users EXCEPT SELECT * FROM users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
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
                                "where": null
                            },
                            "compound": [{
                                "operation": "Except",
                                "core": {
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
                                    "where": null
                                },
                            }],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    union_and_except: {
        "SELECT * FROM users UNION SELECT * FROM users EXCEPT SELECT * FROM users;" => {
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
                                    "type": "Group",
                                    "subqueries": [{
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "users",
                                        "namespace": null,
                                    }],
                                },
                                "where": null
                            },
                            "compound": [{
                                "operation": "Union",
                                "core": {
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
                                    "where": null
                                },
                            },{
                                "operation": "Except",
                                "core": {
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
                                    "where": null,
                                },
                            }],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}
