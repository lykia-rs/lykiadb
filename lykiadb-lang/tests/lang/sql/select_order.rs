use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    no_order: {
        "SELECT * from users;" => {
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
                      "@type": "SqlFrom::Group",
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
                      "@type": "SqlFrom::Group",
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
                  "limit": null,
                  "order_by": [
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "id"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Desc"
                      }
                    }
                  ]
                }
              }
            }
          ]
        }
    },
    multiple_order_0: {
        "SELECT * from users order by id, name;" => {
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
                      "@type": "SqlFrom::Group",
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
                  "limit": null,
                  "order_by": [
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "id"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Asc"
                      }
                    },
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "name"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Asc"
                      }
                    }
                  ]
                }
              }
            }
          ]
        }
    },
    multiple_order_1: {
        "SELECT * from users order by id asc, name;" =>         {
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
                      "@type": "SqlFrom::Group",
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
                  "limit": null,
                  "order_by": [
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "id"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Asc"
                      }
                    },
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "name"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Asc"
                      }
                    }
                  ]
                }
              }
            }
          ]
        }
    },
    multiple_order_2: {
        "SELECT * from users order by id, name desc;" => {
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
                      "@type": "SqlFrom::Group",
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
                  "limit": null,
                  "order_by": [
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "id"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Asc"
                      }
                    },
                    {
                      "@type": "SqlOrderByClause",
                      "expr": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "name"
                        }
                      },
                      "ordering": {
                        "@type": "SqlOrdering::Desc"
                      }
                    }
                  ]
                }
              }
            }
          ]
        }
    }
}
