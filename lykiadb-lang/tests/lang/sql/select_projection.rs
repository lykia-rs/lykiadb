use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
        "SELECT * from users;" =>         {
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
                              "kind": "IdentifierKind::Plain",
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

    collection: {
        "SELECT users.* from users;" =>         {
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
                              "kind": "IdentifierKind::Plain",
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
                          "collection": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "users"
                          }
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
    mixed_0: {
        "SELECT id, users.name as username from users;" => {
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
                              "kind": "IdentifierKind::Plain",
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
                          "@type": "SqlProjection::Expr",
                          "alias": null,
                          "expr": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "id"
                            },
                            "tail": []
                          }
                        },
                        {
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "username"
                          },
                          "expr": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "users"
                            },
                            "tail": [{
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "name"
                            }]
                          }
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
    mixed_1: {
        "SELECT 5 as five, \"text\" as some_text from users;" =>         {
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
                              "kind": "IdentifierKind::Plain",
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
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "five"
                          },
                          "expr": {
                            "@type": "Expr::Literal",
                            "raw": "5",
                            "value": {
                              "Num": 5.0
                            }
                          }
                        },
                        {
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "some_text"
                          },
                          "expr": {
                            "@type": "Expr::Literal",
                            "raw": "text",
                            "value": {
                              "Str": "text"
                            }
                          }
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
    mixed_2: {
        "SELECT 5 + 27 as addition, 4 / 2 as division from users;" =>         {
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
                              "kind": "IdentifierKind::Plain",
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
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "addition"
                          },
                          "expr": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::Literal",
                              "raw": "5",
                              "value": {
                                "Num": 5.0
                              }
                            },
                            "operation": {
                              "@type": "Add"
                            },
                            "right": {
                              "@type": "Expr::Literal",
                              "raw": "27",
                              "value": {
                                "Num": 27.0
                              }
                            }
                          }
                        },
                        {
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "division"
                          },
                          "expr": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::Literal",
                              "raw": "4",
                              "value": {
                                "Num": 4.0
                              }
                            },
                            "operation": {
                              "@type": "Divide"
                            },
                            "right": {
                              "@type": "Expr::Literal",
                              "raw": "2",
                              "value": {
                                "Num": 2.0
                              }
                            }
                          }
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
    mixed_no_from: {
        "SELECT 5 + 27 as addition, 4 / 2 as division;" =>         {
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
                      "from": null,
                      "group_by": null,
                      "having": null,
                      "projection": [
                        {
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "addition"
                          },
                          "expr": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::Literal",
                              "raw": "5",
                              "value": {
                                "Num": 5.0
                              }
                            },
                            "operation": {
                              "@type": "Add"
                            },
                            "right": {
                              "@type": "Expr::Literal",
                              "raw": "27",
                              "value": {
                                "Num": 27.0
                              }
                            }
                          }
                        },
                        {
                          "@type": "SqlProjection::Expr",
                          "alias": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "division"
                          },
                          "expr": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::Literal",
                              "raw": "4",
                              "value": {
                                "Num": 4.0
                              }
                            },
                            "operation": {
                              "@type": "Divide"
                            },
                            "right": {
                              "@type": "Expr::Literal",
                              "raw": "2",
                              "value": {
                                "Num": 2.0
                              }
                            }
                          }
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
