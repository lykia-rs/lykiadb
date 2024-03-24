use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    id_in: {
        "SELECT * FROM users WHERE id IN (SELECT id FROM users);" => {
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
                    "distinct": {
                      "@type": "SqlDistinct::ImplicitAll"
                    },
                    "projection": [
                      {
                        "@type": "SqlProjection::All",
                        "collection": null
                      }
                    ],
                    "from": {
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "namespace": null,
                          "name": {
                            "@type": "Identifier",
                            "name": "users",
                            "dollar": false
                          },
                          "alias": null
                        }
                      ]
                    },
                    "where": {
                      "@type": "SqlExpr::In",
                      "left": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "name": "id",
                          "dollar": false
                        }
                      },
                      "right": {
                        "@type": "Expr::Grouping",
                        "expr": {
                          "@type": "Expr::Select",
                          "query": {
                            "@type": "SqlSelect",
                            "core": {
                              "@type": "SqlSelectCore",
                              "distinct": {
                                "@type": "SqlDistinct::ImplicitAll"
                              },
                              "projection": [
                                {
                                  "@type": "SqlProjection::Expr",
                                  "expr": {
                                    "@type": "Expr::Variable",
                                    "name": {
                                      "@type": "Identifier",
                                      "name": "id",
                                      "dollar": false
                                    }
                                  },
                                  "alias": null
                                }
                              ],
                              "from": {
                                "@type": "SqlCollectionSubquery::Group",
                                "values": [
                                  {
                                    "@type": "SqlCollectionIdentifier",
                                    "namespace": null,
                                    "name": {
                                      "@type": "Identifier",
                                      "name": "users",
                                      "dollar": false
                                    },
                                    "alias": null
                                  }
                                ]
                              },
                              "where": null,
                              "group_by": null,
                              "having": null,
                              "compound": null
                            },
                            "order_by": null,
                            "limit": null
                          }
                        }
                      }
                    },
                    "group_by": null,
                    "having": null,
                    "compound": null
                  },
                  "order_by": null,
                  "limit": null
                }
              }
            }
          ]
        }
    },
    id_not_in: {
      "SELECT * FROM users WHERE id NOT IN (SELECT id FROM users);" => {
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
                  "distinct": {
                    "@type": "SqlDistinct::ImplicitAll"
                  },
                  "projection": [
                    {
                      "@type": "SqlProjection::All",
                      "collection": null
                    }
                  ],
                  "from": {
                    "@type": "SqlCollectionSubquery::Group",
                    "values": [
                      {
                        "@type": "SqlCollectionIdentifier",
                        "namespace": null,
                        "name": {
                          "@type": "Identifier",
                          "name": "users",
                          "dollar": false
                        },
                        "alias": null
                      }
                    ]
                  },
                  "where": {
                    "@type": "SqlExpr::NotIn",
                    "left": {
                      "@type": "Expr::Variable",
                      "name": {
                        "@type": "Identifier",
                        "name": "id",
                        "dollar": false
                      }
                    },
                    "right": {
                      "@type": "Expr::Grouping",
                      "expr": {
                        "@type": "Expr::Select",
                        "query": {
                          "@type": "SqlSelect",
                          "core": {
                            "@type": "SqlSelectCore",
                            "distinct": {
                              "@type": "SqlDistinct::ImplicitAll"
                            },
                            "projection": [
                              {
                                "@type": "SqlProjection::Expr",
                                "expr": {
                                  "@type": "Expr::Variable",
                                  "name": {
                                    "@type": "Identifier",
                                    "name": "id",
                                    "dollar": false
                                  }
                                },
                                "alias": null
                              }
                            ],
                            "from": {
                              "@type": "SqlCollectionSubquery::Group",
                              "values": [
                                {
                                  "@type": "SqlCollectionIdentifier",
                                  "namespace": null,
                                  "name": {
                                    "@type": "Identifier",
                                    "name": "users",
                                    "dollar": false
                                  },
                                  "alias": null
                                }
                              ]
                            },
                            "where": null,
                            "group_by": null,
                            "having": null,
                            "compound": null
                          },
                          "order_by": null,
                          "limit": null
                        }
                      }
                    }
                  },
                  "group_by": null,
                  "having": null,
                  "compound": null
                },
                "order_by": null,
                "limit": null
              }
            }
          }
        ]
      }
  }
}
