#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users join orders;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "users",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "orders",
                                            "namespace": null,
                                        },
                                        "constraint": null
                                    }]
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [],
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
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "users",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "orders",
                                            "namespace": null,
                                        },
                                        "constraint":         {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "id",
                                              "object": {
                                                "name": "users",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "user_id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    },{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "order_items",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "order_id",
                                              "object": {
                                                "name": "carts",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    }]
                                },
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [],
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
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "a",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Left",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "b",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Logical",
                                            "operation": "And",
                                            "left":  {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                  "name": "num",
                                                  "object": {
                                                    "name": "b",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                                "right": {
                                                  "name": "num",
                                                  "object": {
                                                    "name": "a",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                            },
                                            "right": {
                                                "type": "Expr::Binary",
                                                "operation": "IsEqual",
                                                "left": {
                                                  "name": "value",
                                                  "object": {
                                                    "name": "b",
                                                    "type": "Expr::Variable"
                                                  },
                                                  "type": "Expr::Get"
                                                },
                                                "right": {
                                                    "type": "Expr::Literal",
                                                    "value": "Str(\"abc\")",
                                                    "raw": "abc",
                                                },
                                            },
                                          }
                                    }]
                                },
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [],
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
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": "a"
                                }],
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": "a",
                                            "name": "demo",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": "b",
                                            "name": "demo",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Logical",
                                            "operation": "And",
                                            "left": {
                                                "type": "Expr::Literal",
                                                "value": "Bool(true)",
                                                "raw": "true",
                                            },
                                            "right": {
                                                "type": "Expr::Literal",
                                                "value": "Bool(true)",
                                                "raw": "true",
                                            },
                                        }
                                    }]
                                },
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    complex_join_1: {
        "select * from ((select * from demo a inner join demo b on a.id = b.id) c inner join demo d on c.id = d.id);" => {
            "type": "Stmt::Program",
            "body": [
              {
                "expr": {
                  "type": "Expr::Select",
                  "value": {
                    "compound": [],
                    "core": {
                      "distinct": "ImplicitAll",
                      "from": {
                        "type": "Group",
                        "subqueries": [{
                            "joins": [
                              {
                                "constraint": {
                                  "left": {
                                    "name": "id",
                                    "object": {
                                      "name": "c",
                                      "type": "Expr::Variable"
                                    },
                                    "type": "Expr::Get"
                                  },
                                  "operation": "IsEqual",
                                  "right": {
                                    "name": "id",
                                    "object": {
                                      "name": "d",
                                      "type": "Expr::Variable"
                                    },
                                    "type": "Expr::Get"
                                  },
                                  "type": "Expr::Binary"
                                },
                                "subquery": {
                                  "alias": "d",
                                  "name": "demo",
                                  "namespace": null,
                                  "type": "Collection"
                                },
                                "type": "Inner"
                              }
                            ],
                            "subquery":         {
                                "subqueries": [
                                  {
                                    "alias": "c",
                                    "expr": {
                                      "type": "Expr::Select",
                                      "value": {
                                        "compound": [],
                                        "core": {
                                          "distinct": "ImplicitAll",
                                          "from": {
                                            "joins": [
                                              {
                                                "constraint": {
                                                  "left": {
                                                    "name": "id",
                                                    "object": {
                                                      "name": "a",
                                                      "type": "Expr::Variable"
                                                    },
                                                    "type": "Expr::Get"
                                                  },
                                                  "operation": "IsEqual",
                                                  "right": {
                                                    "name": "id",
                                                    "object": {
                                                      "name": "b",
                                                      "type": "Expr::Variable"
                                                    },
                                                    "type": "Expr::Get"
                                                  },
                                                  "type": "Expr::Binary"
                                                },
                                                "subquery": {
                                                  "alias": "b",
                                                  "name": "demo",
                                                  "namespace": null,
                                                  "type": "Collection"
                                                },
                                                "type": "Inner"
                                              }
                                            ],
                                            "subquery": {
                                              "subqueries": [
                                                {
                                                  "alias": "a",
                                                  "name": "demo",
                                                  "namespace": null,
                                                  "type": "Collection"
                                                }
                                              ],
                                              "type": "Group"
                                            },
                                            "type": "Join"
                                          },
                                          "group_by": null,
                                          "having": null,
                                          "projection": [
                                            {
                                              "collection": null,
                                              "type": "All"
                                            }
                                          ],
                                          "where": null
                                        },
                                        "limit": null,
                                        "order_by": null
                                      }
                                    },
                                    "type": "Select"
                                  }
                                ],
                                "type": "Group"
                              },
                            "type": "Join"
                          }
                        ]
                      },
                      "group_by": null,
                      "having": null,
                      "projection": [
                        {
                          "collection": null,
                          "type": "All"
                        }
                      ],
                      "where": null
                    },
                    "limit": null,
                    "order_by": null
                  }
                },
                "type": "Stmt::Expression"
              }
            ]
        }
    },
    complex_join_2: {
        "select * from users inner join orders on users.id = orders.user_id inner join baskets on baskets.order_id = orders.id;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "distinct": "ImplicitAll",
                                "projection": [{
                                    "type": "All",
                                    "collection": null
                                }],
                                "from": {
                                    "type": "Join",
                                    "subquery": {
                                        "type": "Group",
                                        "subqueries": [{
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "users",
                                            "namespace": null,
                                        }]
                                    },
                                    "joins": [{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "orders",
                                            "namespace": null,
                                        },
                                        "constraint":         {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "id",
                                              "object": {
                                                "name": "users",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "user_id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    },{
                                        "type": "Inner",
                                        "subquery": {
                                            "type": "Collection",
                                            "alias": null,
                                            "name": "baskets",
                                            "namespace": null,
                                        },
                                        "constraint": {
                                            "type": "Expr::Binary",
                                            "operation": "IsEqual",
                                            "left": {
                                              "name": "order_id",
                                              "object": {
                                                "name": "baskets",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                            "right": {
                                              "name": "id",
                                              "object": {
                                                "name": "orders",
                                                "type": "Expr::Variable"
                                              },
                                              "type": "Expr::Get"
                                            },
                                          }
                                    }]
                                },
                                "where": null,
                                "group_by": null,
                                "having": null
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}
