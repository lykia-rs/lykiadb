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
                              "dollar": false,
                              "name": "users"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
                                "dollar": false,
                                "name": "orders"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "dollar": false,
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
                                "dollar": false,
                                "name": "carts"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "dollar": false,
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
                                  "dollar": false,
                                  "name": "users"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                  "dollar": false,
                                  "name": "orders"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                "dollar": false,
                                "name": "users"
                              },
                              "namespace": null
                            },
                            "right": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "dollar": false,
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
                              "dollar": false,
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
                                  "dollar": false,
                                  "name": "b"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                  "dollar": false,
                                  "name": "a"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                  "dollar": false,
                                  "name": "b"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                              "dollar": false,
                              "name": "a"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
                              "dollar": false,
                              "name": "a"
                            },
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "demo"
                            },
                            "namespace": null
                          },
                          "right": {
                            "@type": "SqlCollectionIdentifier",
                            "alias": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "b"
                            },
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
                          "dollar": false,
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
                                    "dollar": false,
                                    "name": "c"
                                  },
                                  "tail": [{
                                    "@type": "Identifier",
                                    "dollar": false,
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
                                    "dollar": false,
                                    "name": "d"
                                  },
                                  "tail": [{
                                    "@type": "Identifier",
                                    "dollar": false,
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
                                  "dollar": false,
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
                                                  "dollar": false,
                                                  "name": "a"
                                                },
                                                "tail": [{
                                                  "@type": "Identifier",
                                                  "dollar": false,
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
                                                  "dollar": false,
                                                  "name": "b"
                                                },
                                                "tail": [{
                                                  "@type": "Identifier",
                                                  "dollar": false,
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
                                                "dollar": false,
                                                "name": "a"
                                              },
                                              "name": {
                                                "@type": "Identifier",
                                                "dollar": false,
                                                "name": "demo"
                                              },
                                              "namespace": null
                                            },
                                            "right": {
                                              "@type": "SqlCollectionIdentifier",
                                              "alias": {
                                                "@type": "Identifier",
                                                "dollar": false,
                                                "name": "b"
                                              },
                                              "name": {
                                                "@type": "Identifier",
                                                "dollar": false,
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
                                  "dollar": false,
                                  "name": "d"
                                },
                                "name": {
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                "dollar": false,
                                "name": "baskets"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "dollar": false,
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
                                "dollar": false,
                                "name": "orders"
                              },
                              "tail": [{
                                "@type": "Identifier",
                                "dollar": false,
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
                                  "dollar": false,
                                  "name": "users"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                  "dollar": false,
                                  "name": "orders"
                                },
                                "tail": [{
                                  "@type": "Identifier",
                                  "dollar": false,
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
                                "dollar": false,
                                "name": "users"
                              },
                              "namespace": null
                            },
                            "right": {
                              "@type": "SqlCollectionIdentifier",
                              "alias": null,
                              "name": {
                                "@type": "Identifier",
                                "dollar": false,
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
                              "dollar": false,
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
