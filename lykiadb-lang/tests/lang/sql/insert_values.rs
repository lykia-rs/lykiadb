use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    plain: {
        "var $result = INSERT INTO db.users values ({
            name: 'John',
            surname: 'Doe',
            age: 42,
        });" =>         {
          "@type": "Stmt::Program",
          "body": [
            {
              "@type": "Stmt::Declaration",
              "dst": {
                "@type": "Identifier",
                "kind": "IdentifierKind::Variable",
                "name": "$result"
              },
              "expr": {
                "@type": "Expr::Insert",
                "command": {
                  "@type": "SqlInsert",
                  "collection": {
                    "@type": "SqlCollectionIdentifier",
                    "alias": null,
                    "name": {
                      "@type": "Identifier",
                      "kind": "IdentifierKind::Symbol",
                      "name": "users"
                    },
                    "namespace": {
                      "@type": "Identifier",
                      "kind": "IdentifierKind::Symbol",
                      "name": "db"
                    }
                  },
                  "values": {
                    "@type": "SqlValues::Values",
                    "values": [
                      {
                        "@type": "Expr::Literal",
                        "raw": "",
                        "value": {
                          "Object": {
                            "age": {
                              "@type": "Expr::Literal",
                              "raw": "42",
                              "value": {
                                "Num": 42.0
                              }
                            },
                            "name": {
                              "@type": "Expr::Literal",
                              "raw": "John",
                              "value": {
                                "Str": "John"
                              }
                            },
                            "surname": {
                              "@type": "Expr::Literal",
                              "raw": "Doe",
                              "value": {
                                "Str": "Doe"
                              }
                            }
                          }
                        }
                      }
                    ]
                  }
                }
              }
            }
          ]
        }
    }
}
