#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    implicit_all: {
        "SELECT surname from users;" => {
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
                                    "alias": null,
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "surname",
                                    },
                                    "type": "Expr"
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
    explicit_all: {
        "SELECT all surname from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "All",
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
                                    "alias": null,
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "surname",
                                    },
                                    "type": "Expr"
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
    distinct: {
        "SELECT distinct surname from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "Distinct",
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
                                    "alias": null,
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "surname",
                                    },
                                    "type": "Expr"
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
