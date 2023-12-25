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
                                }]
                            },
                            "compound": [{
                                "operation": "Union",
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
                                    }]
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
