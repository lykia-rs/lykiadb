use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    single_union: {
        "SELECT * FROM users UNION SELECT * FROM users;" => {
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
                      "compound": {
                        "@type": "SqlSelectCompound",
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
                        "operator": {
                          "@type": "SqlCompoundOperator::Union"
                        }
                      },
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
    single_intersect: {
        "SELECT * FROM users INTERSECT SELECT * FROM users;" => {
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
                      "compound": {
                        "@type": "SqlSelectCompound",
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
                        "operator": {
                          "@type": "SqlCompoundOperator::Intersect"
                        }
                      },
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
    single_except: {
        "SELECT * FROM users EXCEPT SELECT * FROM users;" => {
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
                      "compound": {
                        "@type": "SqlSelectCompound",
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
                        "operator": {
                          "@type": "SqlCompoundOperator::Except"
                        }
                      },
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
    union_and_except: {
        "SELECT * FROM users UNION SELECT * FROM users EXCEPT SELECT * FROM users;" => {
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
                      "compound": {
                        "@type": "SqlSelectCompound",
                        "core": {
                          "@type": "SqlSelectCore",
                          "compound": {
                            "@type": "SqlSelectCompound",
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
                            "operator": {
                              "@type": "SqlCompoundOperator::Except"
                            }
                          },
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
                        "operator": {
                          "@type": "SqlCompoundOperator::Union"
                        }
                      },
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
    }
}
