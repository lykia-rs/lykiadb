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
                                "projection": [{
                                    "collection": null
                                }]
                            },
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
                                "projection": [{
                                    "collection": "users"
                                }]
                            },
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
                                "projection": [{
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "id"
                                    },
                                    "alias": null
                                },
                                {
                                    "expr": {
                                        "type": "Expr::Get",
                                        "object": {
                                            "type": "Expr::Variable",
                                            "name": "users"
                                        },
                                        "name": "name"
                                    },
                                    "alias": "username"
                                }]
                            },
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}
