#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "var $result = INSERT INTO db.users values ({
            name: 'John',
            surname: 'Doe',
            age: 42,
        });" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Insert",
                        "value": {
                        }
                    }
                }
            ]
        }
    }
}
