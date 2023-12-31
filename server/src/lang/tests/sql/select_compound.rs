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
                            "compound": [{
                                "operation": "Union",
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
                                    "where": null,
                                    "group_by": null,
                                    "having": null
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
                            "compound": [{
                                "operation": "Intersect",
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
                            "compound": [{
                                "operation": "Except",
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
                                    "where": null,
                                    "group_by": null,
                                    "having": null
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
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [{
                                "operation": "Union",
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
                                    "where": null,
                                    "group_by": null,
                                    "having": null
                                },
                            },{
                                "operation": "Except",
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
                                    "where": null,
                                    "group_by": null,
                                    "having": null
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