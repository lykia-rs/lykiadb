use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
        "SELECT avg(salary) from employees group by department_id;" => {
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
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::FieldPath",
                        "head": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Plain",
                          "name": "department_id"
                        },
                        "tail": []
                      }
                    ],
                    "having": null,
                    "projection": [
                      {
                        "@type": "SqlProjection::Expr",
                        "alias": null,
                        "expr": {
                          "@type": "Expr::Call",
                          "args": [
                            {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "salary"
                              },
                              "tail": []
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "avg"
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
    more_complex_0: {
        "SELECT avg(salary) from employees group by department_id having avg(salary) > 1000;" => {
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
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::FieldPath",
                        "head": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Plain",
                          "name": "department_id"
                        },
                        "tail": []
                      }
                    ],
                    "having": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Call",
                        "args": [
                          {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "salary"
                            },
                            "tail": []
                          }
                        ],
                        "callee": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "avg"
                          }
                        }
                      },
                      "operation": {
                        "@type": "Greater"
                      },
                      "right": {
                        "@type": "Expr::Literal",
                        "raw": "1000",
                        "value": {
                          "Num": 1000.0
                        }
                      }
                    },
                    "projection": [
                      {
                        "@type": "SqlProjection::Expr",
                        "alias": null,
                        "expr": {
                          "@type": "Expr::Call",
                          "args": [
                            {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "salary"
                              },
                              "tail": []
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "avg"
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
    more_complex_1: {
        "SELECT avg(salary) from employees group by department_id, job_id having avg(salary) > 1000;" => {
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
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::FieldPath",
                        "head": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Plain",
                          "name": "department_id"
                        },
                        "tail": []
                      },
                      {
                        "@type": "Expr::FieldPath",
                        "head": {
                          "@type": "Identifier",
                          "kind": "IdentifierKind::Plain",
                          "name": "job_id"
                        },
                        "tail": []
                      }
                    ],
                    "having": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Call",
                        "args": [
                          {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "salary"
                            },
                            "tail": []
                          }
                        ],
                        "callee": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "kind": "IdentifierKind::Plain",
                            "name": "avg"
                          }
                        }
                      },
                      "operation": {
                        "@type": "Greater"
                      },
                      "right": {
                        "@type": "Expr::Literal",
                        "raw": "1000",
                        "value": {
                          "Num": 1000.0
                        }
                      }
                    },
                    "projection": [
                      {
                        "@type": "SqlProjection::Expr",
                        "alias": null,
                        "expr": {
                          "@type": "Expr::Call",
                          "args": [
                            {
                              "@type": "Expr::FieldPath",
                              "head": {
                                "@type": "Identifier",
                                "kind": "IdentifierKind::Plain",
                                "name": "salary"
                              },
                              "tail": []
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "avg"
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
