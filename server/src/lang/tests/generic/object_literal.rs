#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain_0: {
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
