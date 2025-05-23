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
                    "@type": "SqlFrom::Group",
                    "values": [
                      {
                        "@type": "SqlCollectionIdentifier",
                        "namespace": null,
                        "name": {
                          "@type": "Identifier",
                          "name": "users",
                          "kind": "IdentifierKind::Symbol"
                        },
                        "alias": null
                      }
                    ]
                  },
                  "where": {
                    "@type": "Expr::Binary",
                    "operation": {
                      "@type": "In"
                    },
                    "left": {
                      "@type": "Expr::FieldPath",
                      "head": {
                        "@type": "Identifier",
                        "name": "id",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "tail": []
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
                                  "@type": "Expr::FieldPath",
                                  "head": {
                                    "@type": "Identifier",
                                    "name": "id",
                                    "kind": "IdentifierKind::Symbol"
                                  },
                                  "tail": []
                                },
                                "alias": null
                              }
                            ],
                            "from": {
                              "@type": "SqlFrom::Group",
                              "values": [
                                {
                                  "@type": "SqlCollectionIdentifier",
                                  "namespace": null,
                                  "name": {
                                    "@type": "Identifier",
                                    "name": "users",
                                    "kind": "IdentifierKind::Symbol"
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "NotIn"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "id",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
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
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "name": "id",
                                  "kind": "IdentifierKind::Symbol"
                                },
                                "tail": []
                              },
                              "alias": null
                            }
                          ],
                          "from": {
                            "@type": "SqlFrom::Group",
                            "values": [
                              {
                                "@type": "SqlCollectionIdentifier",
                                "namespace": null,
                                "name": {
                                  "@type": "Identifier",
                                  "name": "users",
                                  "kind": "IdentifierKind::Symbol"
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
  id_between: {
    "SELECT * FROM users WHERE id between 100 and 500;" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Between",
                  "kind": {
                    "@type": "Between"
                  },
                  "subject": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "id",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "lower": {
                    "@type": "Expr::Literal",
                    "raw": "100",
                    "value": {
                      "Num": 100.0
                    }
                  },
                  "upper": {
                    "@type": "Expr::Literal",
                    "raw": "500",
                    "value": {
                      "Num": 500.0
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
  ,
  id_not_between: {
    "SELECT * FROM users WHERE id not between 100 and 500;" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Between",
                  "kind": {
                    "@type": "NotBetween"
                  },
                  "subject": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "id",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "lower": {
                    "@type": "Expr::Literal",
                    "raw": "100",
                    "value": {
                      "Num": 100.0
                    }
                  },
                  "upper": {
                    "@type": "Expr::Literal",
                    "raw": "500",
                    "value": {
                      "Num": 500.0
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
  name_like: {
    "SELECT * FROM users WHERE name like '%John%';" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "Like"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "name",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "value": {
                      "Str": "%John%"
                    },
                    "raw": "%John%"
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
  name_not_like: {
    "SELECT * FROM users WHERE name not like '%John%';" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "NotLike"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "name",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "value": {
                      "Str": "%John%"
                    },
                    "raw": "%John%"
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
  level_is_100: {
    "SELECT * FROM users WHERE level is 100;" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "Is"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "level",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "value": {
                      "Num": 100.0
                    },
                    "raw": "100"
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
  level_is_not_100: {
    "SELECT * FROM users WHERE level is not 100;" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "IsNot"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "level",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "value": {
                      "Num": 100.0
                    },
                    "raw": "100"
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
  aggregate_call: {
    "SELECT avg(id) FROM users;" => {
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
                    "@type": "SqlProjection::Expr",
                    "expr": {
                      "@type": "Expr::Call",
                      "callee": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Symbol",
                          "name": "avg"
                        }
                      },
                      "args": [
                        {
                          "@type": "Expr::FieldPath",
                          "head": {
                            "@type": "Identifier",
                            "name": "id",
                            "kind": "IdentifierKind::Symbol"
                          },
                          "tail": []
                        }
                      ],
                    },
                    "alias": null
                  }
                ],
                "from": {
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
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
      ]
    }
  },
  function_call: {
    "SELECT * FROM users WHERE id = $hello(50);" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "IsEqual"
                  },
                  "left": {
                    "@type": "Expr::FieldPath",
                    "head": {
                      "@type": "Identifier",
                      "name": "id",
                      "kind": "IdentifierKind::Symbol"
                    },
                    "tail": []
                  },
                  "right": {
                    "@type": "Expr::Call",
                    "callee": {
                      "@type": "Expr::Variable",
                      "name": {
                        "@type": "Identifier",
                        "kind": "IdentifierKind::Variable",
                        "name": "$hello"
                      }
                    },
                    "args": [
                      {
                        "@type": "Expr::Literal",
                        "raw": "50",
                        "value": {
                          "Num": 50.0
                        }
                      }
                    ]
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
  namespace_function_call: {
    "SELECT * FROM users WHERE math::ceil(score) > 100;" => {
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
                  "@type": "SqlFrom::Group",
                  "values": [
                    {
                      "@type": "SqlCollectionIdentifier",
                      "namespace": null,
                      "name": {
                        "@type": "Identifier",
                        "name": "users",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "alias": null
                    }
                  ]
                },
                "where": {
                  "@type": "Expr::Binary",
                  "operation": {
                    "@type": "Greater"
                  },
                  "left": {
                    "@type": "Expr::Call",
                    "args":[{
                      "@type": "Expr::FieldPath",
                      "head": {
                        "@type": "Identifier",
                        "name": "score",
                        "kind": "IdentifierKind::Symbol"
                      },
                      "tail": []
                    }],
                    "callee": {
                      "@type": "Expr::Get",
                      "object": {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Symbol",
                          "name": "math"
                        }
                      },
                      "name": {
                        "@type": "Identifier",
                        "kind": "IdentifierKind::Symbol",
                        "name": "ceil"
                      }
                    },
                  },
                  "right": {
                    "@type": "Expr::Literal",
                    "raw": "100",
                    "value": {
                      "Num": 100.0
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
