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
            "type": "Stmt::Program",
            "body": [{
                  "expr": {
                    "type": "Expr::Select",
                    "value": {
                      "compound": [],
                      "core": {
                        "distinct": "ImplicitAll",
                        "from": {
                          "subqueries": [
                            {
                              "alias": null,
                              "name": "employees",
                              "namespace": null,
                              "type": "Collection"
                            }
                          ],
                          "type": "Group"
                        },
                        "projection": [
                          {
                            "alias": null,
                            "expr": {
                               "callee": {
                                  "name": "avg",
                                  "type": "Expr::Variable"
                              },
                              "args": [
                                {
                                  "name": "salary",
                                  "type": "Expr::Variable"
                                }
                              ],
                              "type": "Expr::Call"
                            },
                            "type": "Expr"
                          }
                        ],
                        "where": null,
                        "group_by":  [{
                            "name": "department_id",
                            "type": "Expr::Variable"
                        }],
                        "having": null
                      },
                      "limit": null,
                      "order_by": null
                    }
                  },
                  "type": "Stmt::Expression"
            }]
        }
    },
    more_complex_0: {
        "SELECT avg(salary) from employees group by department_id having avg(salary) > 1000;" => {
            "type": "Stmt::Program",
            "body": [{
                  "expr": {
                    "type": "Expr::Select",
                    "value": {
                      "compound": [],
                      "core": {
                        "distinct": "ImplicitAll",
                        "from": {
                          "subqueries": [
                            {
                              "alias": null,
                              "name": "employees",
                              "namespace": null,
                              "type": "Collection"
                            }
                          ],
                          "type": "Group"
                        },
                        "projection": [
                          {
                            "alias": null,
                            "expr": {
                               "callee": {
                                  "name": "avg",
                                  "type": "Expr::Variable"
                              },
                              "args": [
                                {
                                  "name": "salary",
                                  "type": "Expr::Variable"
                                }
                              ],
                              "type": "Expr::Call"
                            },
                            "type": "Expr"
                          }
                        ],
                        "where": null,
                        "group_by":  [{
                            "name": "department_id",
                            "type": "Expr::Variable"
                        }],
                        "having": {
                            "type": "Expr::Binary",
                            "operation": "Greater",
                            "left": {
                                "callee": {
                                    "name": "avg",
                                    "type": "Expr::Variable"
                                },
                                "args": [
                                    {
                                        "name": "salary",
                                        "type": "Expr::Variable"
                                    }
                                ],
                                "type": "Expr::Call"
                            },
                            "right": {
                                "type": "Expr::Literal",
                                "value": "Num(1000.0)",
                                "raw": "1000"
                            }
                        }
                      },
                      "limit": null,
                      "order_by": null
                    }
                  },
                  "type": "Stmt::Expression"
            }]
        }
    },
    more_complex_1: {
        "SELECT avg(salary) from employees group by department_id, job_id having avg(salary) > 1000;" => {
            "type": "Stmt::Program",
            "body": [{
                  "expr": {
                    "type": "Expr::Select",
                    "value": {
                      "compound": [],
                      "core": {
                        "distinct": "ImplicitAll",
                        "from": {
                          "subqueries": [
                            {
                              "alias": null,
                              "name": "employees",
                              "namespace": null,
                              "type": "Collection"
                            }
                          ],
                          "type": "Group"
                        },
                        "projection": [
                          {
                            "alias": null,
                            "expr": {
                               "callee": {
                                  "name": "avg",
                                  "type": "Expr::Variable"
                              },
                              "args": [
                                {
                                  "name": "salary",
                                  "type": "Expr::Variable"
                                }
                              ],
                              "type": "Expr::Call"
                            },
                            "type": "Expr"
                          }
                        ],
                        "where": null,
                        "group_by":  [
                            {
                                "name": "department_id",
                                "type": "Expr::Variable"
                            },
                            {
                                "name": "job_id",
                                "type": "Expr::Variable"
                            }
                        ],
                        "having": {
                            "type": "Expr::Binary",
                            "operation": "Greater",
                            "left": {
                                "callee": {
                                    "name": "avg",
                                    "type": "Expr::Variable"
                                },
                                "args": [
                                    {
                                        "name": "salary",
                                        "type": "Expr::Variable"
                                    }
                                ],
                                "type": "Expr::Call"
                            },
                            "right": {
                                "type": "Expr::Literal",
                                "value": "Num(1000.0)",
                                "raw": "1000"
                            }
                        }
                      },
                      "limit": null,
                      "order_by": null
                    }
                  },
                  "type": "Stmt::Expression"
            }]
        }
    }
}
