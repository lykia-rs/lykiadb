#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    limit_5: {
        "SELECT * from users limit 5;" => ""
    },
    limit_5_offset_10: {
        "SELECT * from users limit 5 offset 10;" =>         {
          "type": "Stmt::Program",
          "body": [
            {
              "type": "Stmt::Expression",
              "expr": {
                "type": "Expr::Select",
                "query": {
                  "type": "SqlSelect",
                  "compound": [],
                  "core": {
                    "distinct": {
                      "type": "SqlDistinct::ImplicitAll"
                    },
                    "from": {
                      "type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "lexeme": "users",
                            "literal": {
                              "Str": "users"
                            },
                            "tok_type": {
                              "Identifier": {
                                "dollar": false
                              }
                            }
                          },
                          "namespace": null,
                        }
                      ],
                    },
                    "group_by": null,
                    "having": null,
                    "projection": [
                      {
                        "collection": null,
                        "type": "SqlProjection::All"
                      }
                    ],
                    "type": "SqlSelectCore",
                    "where": null
                  },
                  "limit": {
                    "count": {
                      "raw": "5",
                      "type": "Expr::Literal",
                      "value": {
                        "Num": 5.0
                      }
                    },
                    "offset": {
                      "raw": "10",
                      "type": "Expr::Literal",
                      "value": {
                        "Num": 10.0
                      }
                    },
                    "type": "SqlLimitClause"
                  },
                  "order_by": null,
                },
              },
            }
          ]
        }
    },
    limit_10_offset_5: {
        "SELECT * from users limit 5, 10;" => ""
    }
}
