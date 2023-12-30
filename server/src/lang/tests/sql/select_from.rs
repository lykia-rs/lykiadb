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
        "SELECT * from users, admins;" => {
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
                                    },
                                    {
                                        "type": "Collection",
                                        "alias": null,
                                        "name": "admins",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
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
    multi_1: {
        "SELECT * from users u, admins as a;" => {
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
                                        "alias": "u",
                                        "name": "users",
                                        "namespace": null,
                                    },
                                    {
                                        "type": "Collection",
                                        "alias": "a",
                                        "name": "admins",
                                        "namespace": null,
                                    }],
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
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
    multi_2: {
        "SELECT * from (SELECT * from users) u;" => {
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
                                        "type": "Select",
                                        "alias": "u",
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
                                                "compound": [],
                                                "limit": null,
                                                "order_by": null
                                            }
                                        }
                                    }],
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
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
