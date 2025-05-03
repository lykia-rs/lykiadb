use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
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
                              "kind": "IdentifierKind::Symbol",
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
    multi_0: {
        "SELECT * from users, admins;" => {
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
                              "kind": "IdentifierKind::Symbol",
                              "name": "users"
                            },
                            "namespace": null
                          },
                          {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "admins"
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
    multi_1: {
        "SELECT * from users u, admins as a;" => {
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
                            "alias": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "u"
                            },
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "users"
                            },
                            "namespace": null
                          },
                          {
                            "@type": "SqlCollectionIdentifier",
                            "alias": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "a"
                            },
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "admins"
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
    multi_2: {
        "SELECT * from (SELECT * from users) u;" => {
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
                            "@type": "SqlFrom::Select",
                            "alias": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "u"
                            },
                            "subquery": {
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
                                          "kind": "IdentifierKind::Symbol",
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
    expr_source: {
      "SELECT * from $users as u;" => {
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
                          "@type": "SqlExpressionSource",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Symbol",
                            "name": "u"
                          },
                          "expr":{
                              "@type": "Expr::Variable",
                              "name": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Variable",
                                  "name": "$users"
                              }
                          }
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
  expr_source_complex: {
    "SELECT * from items i, ['user1', 'user2'] as u;" => {
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
                        "alias":  {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Symbol",
                          "name": "i"
                        },
                        "name": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Symbol",
                          "name": "items"
                        },
                        "namespace": null
                      },
                      {
                        "@type": "SqlExpressionSource",
                        "alias": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Symbol",
                          "name": "u"
                        },
                        "expr": {
                          "@type": "Expr::Literal",
                          "raw": "",
                          "value": {
                            "Array": [
                              {
                                "@type": "Expr::Literal",
                                "raw": "user1",
                                "value": {
                                  "Str": "user1"
                                }
                              },
                              {
                                "@type": "Expr::Literal",
                                "raw": "user2",
                                "value": {
                                  "Str": "user2"
                                }
                              }
                            ]
                          }
                        }
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
  }

}
