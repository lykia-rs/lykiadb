#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    no_order: {
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
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    single_order: {
        "SELECT * from users order by id desc;" => {
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
                            "compound": [],
                            "limit": null,
                            "order_by": [{
                                "expr": {
                                    "name": "id",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Desc"
                            }]
                        }
                    }
                }
            ]
        }
    },
    multiple_order_0: {
        "SELECT * from users order by id, name;" => {
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
                            "compound": [],
                            "limit": null,
                            "order_by": [{
                                "expr": {
                                    "name": "id",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Asc"
                            },
                            {
                                "expr": {
                                    "name": "name",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Asc"
                            }]
                        }
                    }
                }
            ]
        }
    },
    multiple_order_1: {
        "SELECT * from users order by id asc, name;" => {
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
                            "compound": [],
                            "limit": null,
                            "order_by": [{
                                "expr": {
                                    "name": "id",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Asc"
                            },
                            {
                                "expr": {
                                    "name": "name",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Asc"
                            }]
                        }
                    }
                }
            ]
        }
    },
    multiple_order_2: {
        "SELECT * from users order by id, name desc;" => {
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
                            "compound": [],
                            "limit": null,
                            "order_by": [{
                                "expr": {
                                    "name": "id",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Asc"
                            },
                            {
                                "expr": {
                                    "name": "name",
                                    "type": "Expr::Variable"
                                },
                                "ordering": "Desc"
                            }]
                        }
                    }
                }
            ]
        }
    }
}
