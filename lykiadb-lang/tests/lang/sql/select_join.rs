use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
        "SELECT * from users join orders;" => {
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
                          "@type": "SqlFrom::Join",
                          "constraint": null,
                          "join_type": {
                            "@type": "SqlJoinType::Inner"
                          },
                          "left": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "users"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "orders"
                            },
                            "namespace": null
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
    constraint_0: {
        "SELECT * from users inner join orders on users.id = orders.user_id inner join order_items on orders.id = carts.order_id;" => {
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
                          "@type": "SqlFrom::Join",
                          "constraint": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "orders"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "id"
                              }]
                            },
                            "operation": {
                              "@type": "IsEqual"
                            },
                            "right": {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "carts"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "order_id"
                              }]
                            }
                          },
                          "join_type": {
                            "@type": "SqlJoinType::Inner"
                          },
                          "left": {
                            "@type": "SqlFrom::Join",
                            "constraint": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "users"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "id"
                                }]
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "orders"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "user_id"
                                }]
                              }
                            },
                            "join_type": {
                              "@type": "SqlJoinType::Inner"
                            },
                            "left": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "users"
                              },
                              "namespace": null
                            },
                            "right": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "orders"
                              },
                              "namespace": null
                            }
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "order_items"
                            },
                            "namespace": null
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
    constraint_1: {
        "SELECT * FROM a LEFT JOIN b ON b.num = a.num and b.value = \"abc\";" => {
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
                          "@type": "SqlFrom::Join",
                          "constraint": {
                            "@type": "Expr::Logical",
                            "left": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "b"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "num"
                                }]
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "a"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "num"
                                }]
                              }
                            },
                            "operation": {
                              "@type": "And"
                            },
                            "right": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "b"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "value"
                                }]
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::Literal",
                                "raw": "abc",
                                "value": {
                                  "Str": "abc"
                                }
                              }
                            }
                          },
                          "join_type": {
                            "@type": "SqlJoinType::Left"
                          },
                          "left": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "a"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "b"
                            },
                            "namespace": null
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
    complex_join_0: {
        "select a.* from demo a join demo b on true and true;" => {
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
                          "@type": "SqlFrom::Join",
                          "constraint": {
                            "@type": "Expr::Logical",
                            "left": {
                              "@type": "Expr::Literal",
                              "raw": "true",
                              "value": {
                                "Bool": true
                              }
                            },
                            "operation": {
                              "@type": "And"
                            },
                            "right": {
                              "@type": "Expr::Literal",
                              "raw": "true",
                              "value": {
                                "Bool": true
                              }
                            }
                          },
                          "join_type": {
                            "@type": "SqlJoinType::Inner"
                          },
                          "left": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "a"
                            },
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "demo"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "b"
                            },
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "demo"
                            },
                            "namespace": null
                          }
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
                          "name": "a"
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
    complex_join_1: {
        "select * from ((select * from demo a inner join demo b on a.id = b.id) c inner join demo d on c.id = d.id);" =>         {
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
                          "@type": "SqlFrom::Group",
                          "values": [
                            {
                              "@type": "SqlFrom::Join",
                              "constraint": {
                                "@type": "Expr::Binary",
                                "left": {
                                  "@type": "Expr::FieldPath",
                                  "head": {
                                    "@type": "Identifier",
                                    "kind": "IdentifierKind::Plain",
                                    "name": "c"
                                  },
                                  "tail": [{
                                    "@type": "Identifier",
                                    "kind": "IdentifierKind::Plain",
                                    "name": "id"
                                  }]
                                },
                                "operation": {
                                  "@type": "IsEqual"
                                },
                                "right": {
                                  "@type": "Expr::FieldPath",
                                  "head": {
                                    "@type": "Identifier",
                                    "kind": "IdentifierKind::Plain",
                                    "name": "d"
                                  },
                                  "tail": [{
                                    "@type": "Identifier",
                                    "kind": "IdentifierKind::Plain",
                                    "name": "id"
                                  }]
                                }
                              },
                              "join_type": {
                                "@type": "SqlJoinType::Inner"
                              },
                              "left": {
                                "@type": "SqlFrom::Select",
                                "alias": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "c"
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
                                            "@type": "SqlFrom::Join",
                                            "constraint": {
                                              "@type": "Expr::Binary",
                                              "left": {
                                                "@type": "Expr::FieldPath",
                                                "head": {
                                                  "@type": "Identifier",
                                                  "kind": "IdentifierKind::Plain",
                                                  "name": "a"
                                                },
                                                "tail": [{
                                                  "@type": "Identifier",
                                                  "kind": "IdentifierKind::Plain",
                                                  "name": "id"
                                                }]
                                              },
                                              "operation": {
                                                "@type": "IsEqual"
                                              },
                                              "right": {
                                                "@type": "Expr::FieldPath",
                                                "head": {
                                                  "@type": "Identifier",
                                                  "kind": "IdentifierKind::Plain",
                                                  "name": "b"
                                                },
                                                "tail": [{
                                                  "@type": "Identifier",
                                                  "kind": "IdentifierKind::Plain",
                                                  "name": "id"
                                                }]
                                              }
                                            },
                                            "join_type": {
                                              "@type": "SqlJoinType::Inner"
                                            },
                                            "left": {
                                              "@type": "SqlCollectionIdentifier",
                                              "alias": {
                                                "@type": "Identifier",
                                                "kind": "IdentifierKind::Plain",
                                                "name": "a"
                                              },
                                              "name": {
                                                "@type": "Identifier",
                                                "kind": "IdentifierKind::Plain",
                                                "name": "demo"
                                              },
                                              "namespace": null
                                            },
                                            "right": {
                                              "@type": "SqlCollectionIdentifier",
                                              "alias": {
                                                "@type": "Identifier",
                                                "kind": "IdentifierKind::Plain",
                                                "name": "b"
                                              },
                                              "name": {
                                                "@type": "Identifier",
                                                "kind": "IdentifierKind::Plain",
                                                "name": "demo"
                                              },
                                              "namespace": null
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
                              },
                              "right": {
                                "@type": "SqlCollectionIdentifier",
                                "alias": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "d"
                                },
                                "name": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "demo"
                                },
                                "namespace": null
                              }
                            }
                          ]
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
    complex_join_2: {
        "select * from users inner join orders on users.id = orders.user_id inner join baskets on baskets.order_id = orders.id;" => {
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
                          "@type": "SqlFrom::Join",
                          "constraint": {
                            "@type": "Expr::Binary",
                            "left": {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "baskets"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "order_id"
                              }]
                            },
                            "operation": {
                              "@type": "IsEqual"
                            },
                            "right":{
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "orders"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "id"
                              }]
                            }
                          },
                          "join_type": {
                            "@type": "SqlJoinType::Inner"
                          },
                          "left": {
                            "@type": "SqlFrom::Join",
                            "constraint": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "users"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "id"
                                }]
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "orders"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Plain",
                                  "name": "user_id"
                                }]
                              }
                            },
                            "join_type": {
                              "@type": "SqlJoinType::Inner"
                            },
                            "left": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "users"
                              },
                              "namespace": null
                            },
                            "right": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "orders"
                              },
                              "namespace": null
                            }
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "baskets"
                            },
                            "namespace": null
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
