#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
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
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "department_id"
                        }
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
                              "@type": "Expr::Variable",
                              "name": {
                                "@type": "Identifier",
                                "dollar": false,
                                "name": "salary"
                              }
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "department_id"
                        }
                      }
                    ],
                    "having": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Call",
                        "args": [
                          {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "salary"
                            }
                          }
                        ],
                        "callee": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
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
                              "@type": "Expr::Variable",
                              "name": {
                                "@type": "Identifier",
                                "dollar": false,
                                "name": "salary"
                              }
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
                      "@type": "SqlCollectionSubquery::Group",
                      "values": [
                        {
                          "@type": "SqlCollectionIdentifier",
                          "alias": null,
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
                            "name": "employees"
                          },
                          "namespace": null
                        }
                      ]
                    },
                    "group_by": [
                      {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "department_id"
                        }
                      },
                      {
                        "@type": "Expr::Variable",
                        "name": {
                          "@type": "Identifier",
                          "dollar": false,
                          "name": "job_id"
                        }
                      }
                    ],
                    "having": {
                      "@type": "Expr::Binary",
                      "left": {
                        "@type": "Expr::Call",
                        "args": [
                          {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "salary"
                            }
                          }
                        ],
                        "callee": {
                          "@type": "Expr::Variable",
                          "name": {
                            "@type": "Identifier",
                            "dollar": false,
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
                              "@type": "Expr::Variable",
                              "name": {
                                "@type": "Identifier",
                                "dollar": false,
                                "name": "salary"
                              }
                            }
                          ],
                          "callee": {
                            "@type": "Expr::Variable",
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
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
