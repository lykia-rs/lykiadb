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
    }
}
