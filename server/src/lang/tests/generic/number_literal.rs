#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    number_1: {
        "1;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Num": 1.0
                          },
                        "raw": "1"
                    }
                }
            ]
        }
    },
    number_floating: {
        "4.0;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Num": 4.0
                          },
                        "raw": "4.0"
                    }
                }
            ]
        }
    },
    number_e: {
        "1e2;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Num": 100.0
                        },
                        "raw": "1e2"
                    }
                }
            ]
        }
    },
    number_e_floating_0: {
        "1.7976931348623157E+308;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value":  {
                            "Num": 1.7976931348623157e308
                          },
                        "raw": "1.7976931348623157E+308"
                    }
                }
            ]
        }
    },
    number_e_floating_1: {
        "1.7976931348623157E308;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Num": 1.7976931348623157e308
                          },
                        "raw": "1.7976931348623157E308"
                    }
                }
            ]
        }
    },
    number_e_floating_2: {
        "1.7976931348623155E-308;" => {
            "@type": "Stmt::Program",
            "body": [
                {
                    "@type": "Stmt::Expression",
                    "expr": {
                        "@type": "Expr::Literal",
                        "value": {
                            "Num": 1.7976931348623155e-308
                          },
                        "raw": "1.7976931348623155E-308"
                    }
                }
            ]
        }
    }
}
