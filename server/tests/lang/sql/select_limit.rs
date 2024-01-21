use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    limit_5: {
        "SELECT * from users limit 5;" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Select",
                "query": {
                  "@type": "SqlSelect",
                  "core": {
                    "@type": "SqlSelectCore",
                    "compound": null,
                    "distinct": {
                      "@type": "SqlDistinct::ImplicitAll"
                    },
                    "from": {
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "users"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": null,
                    "having": null,
                    "projection": [
                      {
                        "@type": "SqlProjection::All",
                        "collection": null
                      }
                    ],
                    "where": null
                  },
                  "limit": {
                    "@type": "SqlLimitClause",
                    "count": {
                      "@type": "Expr::Literal",
                      "raw": "5",
                      "value": {
                        "Num": 5.0
                      }
                    },
                    "offset": null
                  },
                  "order_by": null
                }
              }
            }
          ]
        }
    },
    limit_5_offset_10: {
        "SELECT * from users limit 5 offset 10;" =>         {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Select",
                "query": {
                  "@type": "SqlSelect",
                  "core": {
                    "@type": "SqlSelectCore",
                    "compound": null,
                    "distinct": {
                      "@type": "SqlDistinct::ImplicitAll"
                    },
                    "from": {
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "users"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": null,
                    "having": null,
                    "projection": [
                      {
                        "@type": "SqlProjection::All",
                        "collection": null
                      }
                    ],
                    "where": null
                  },
                  "limit": {
                    "@type": "SqlLimitClause",
                    "count": {
                      "@type": "Expr::Literal",
                      "raw": "5",
                      "value": {
                        "Num": 5.0
                      }
                    },
                    "offset": {
                      "@type": "Expr::Literal",
                      "raw": "10",
                      "value": {
                        "Num": 10.0
                      }
                    }
                  },
                  "order_by": null
                }
              }
            }
          ]
        }
    },
    limit_10_offset_5: {
        "SELECT * from users limit 5, 10;" => {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Expression",
              "expr": {
                "@type": "Expr::Select",
                "query": {
                  "@type": "SqlSelect",
                  "core": {
                    "@type": "SqlSelectCore",
                    "compound": null,
                    "distinct": {
                      "@type": "SqlDistinct::ImplicitAll"
                    },
                    "from": {
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "users"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": null,
                    "having": null,
                    "projection": [
                      {
                        "@type": "SqlProjection::All",
                        "collection": null
                      }
                    ],
                    "where": null
                  },
                  "limit": {
                    "@type": "SqlLimitClause",
                    "count": {
                      "@type": "Expr::Literal",
                      "raw": "10",
                      "value": {
                        "Num": 10.0
                      }
                    },
                    "offset": {
                      "@type": "Expr::Literal",
                      "raw": "5",
                      "value": {
                        "Num": 5.0
                      }
                    }
                  },
                  "order_by": null
                }
              }
            }
          ]
        }
    }
}
