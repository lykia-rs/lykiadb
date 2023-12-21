#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    multiple_order: {
        "SELECT * from users order by id, name desc;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
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
