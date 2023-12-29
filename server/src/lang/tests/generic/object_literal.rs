#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    empty_expr: {
        "{};" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Object",
                            "value": []
                        }
                    }
                }
            ]
        }
    },
    empty_declare: {
        "var $obj = {};" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Declaration",
                    "variable": "$obj",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Object",
                            "value": []
                        }
                    }
                }
            ]
        }
    },
    plain_expr: {
        "{ a : 1, b : `q` };" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Literal",
                        "raw": "",
                        "value": {
                            "type": "Object",
                            "value": [
                                {
                                  "key": "b",
                                  "value": {
                                    "raw": "q",
                                    "type": "Expr::Literal",
                                    "value": "Str(\"q\")"
                                  }
                                },
                                {
                                  "key": "a",
                                  "value": {
                                    "raw": "1",
                                    "type": "Expr::Literal",
                                    "value": "Num(1.0)"
                                  }
                                }
                              ]
                        }
                    }
                }
            ]
        }
    },
    plain_declare: {
      "var $obj = { a : 1, b : `q` };" => {
          "type": "Stmt::Program",
          "body": [
              {
                  "type": "Stmt::Declaration",
                  "variable": "$obj",
                  "expr": {
                      "type": "Expr::Literal",
                      "raw": "",
                      "value": {
                          "type": "Object",
                          "value": [
                              {
                                "key": "b",
                                "value": {
                                  "raw": "q",
                                  "type": "Expr::Literal",
                                  "value": "Str(\"q\")"
                                }
                              },
                              {
                                "key": "a",
                                "value": {
                                  "raw": "1",
                                  "type": "Expr::Literal",
                                  "value": "Num(1.0)"
                                }
                              }
                            ]
                      }
                  }
              }
          ]
      }
  }
}
