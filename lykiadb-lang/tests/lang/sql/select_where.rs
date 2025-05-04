use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
        "SELECT * from users where id = 1;" => {
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
                      "where": {
                        "@type": "Expr::Binary",
                        "left": {
                          "@type": "Expr::FieldPath",
                          "head": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Symbol",
                            "name": "id"
                          },
                          "tail": []
                        },
                        "operation": {
                          "@type": "IsEqual"
                        },
                        "right": {
                          "@type": "Expr::Literal",
                          "raw": "1",
                          "value": {
                            "Num": 1.0
                          }
                        }
                      }
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
        "SELECT * from users where id > 100 and name = 'John';" => {
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
                      "where": {
                        "@type": "Expr::Logical",
                        "left": {
                          "@type": "Expr::Binary",
                          "left": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Symbol",
                              "name": "id"
                            },
                            "tail": []
                          },
                          "operation": {
                            "@type": "Greater"
                          },
                          "right": {
                            "@type": "Expr::Literal",
                            "raw": "100",
                            "value": {
                              "Num": 100.0
                            }
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
                              "kind": "IdentifierKind::Symbol",
                              "name": "name"
                            },
                            "tail": []
                          },
                          "operation": {
                            "@type": "IsEqual"
                          },
                          "right": {
                            "@type": "Expr::Literal",
                            "raw": "John",
                            "value": {
                              "Str": "John"
                            }
                          }
                        }
                      }
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
        "SELECT * from users where (id > 100 and name = 'John') or (id < 10 and name = 'Jane');" => {
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
                      "where": {
                        "@type": "Expr::Logical",
                        "left": {
                          "@type": "Expr::Grouping",
                          "expr": {
                            "@type": "Expr::Logical",
                            "left": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Symbol",
                                  "name": "id"
                                },
                                "tail": []
                              },
                              "operation": {
                                "@type": "Greater"
                              },
                              "right": {
                                "@type": "Expr::Literal",
                                "raw": "100",
                                "value": {
                                  "Num": 100.0
                                }
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
                                  "kind": "IdentifierKind::Symbol",
                                  "name": "name"
                                },
                                "tail": []
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::Literal",
                                "raw": "John",
                                "value": {
                                  "Str": "John"
                                }
                              }
                            }
                          }
                        },
                        "operation": {
                          "@type": "Or"
                        },
                        "right": {
                          "@type": "Expr::Grouping",
                          "expr": {
                            "@type": "Expr::Logical",
                            "left": {
                              "@type": "Expr::Binary",
                              "left": {
                                "@type": "Expr::FieldPath",
                                "head": {
                                  "@type": "Identifier",
                                  "kind": "IdentifierKind::Symbol",
                                  "name": "id"
                                },
                                "tail": []
                              },
                              "operation": {
                                "@type": "Less"
                              },
                              "right": {
                                "@type": "Expr::Literal",
                                "raw": "10",
                                "value": {
                                  "Num": 10.0
                                }
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
                                  "kind": "IdentifierKind::Symbol",
                                  "name": "name"
                                },
                                "tail": []
                              },
                              "operation": {
                                "@type": "IsEqual"
                              },
                              "right": {
                                "@type": "Expr::Literal",
                                "raw": "Jane",
                                "value": {
                                  "Str": "Jane"
                                }
                              }
                            }
                          }
                        }
                      }
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
