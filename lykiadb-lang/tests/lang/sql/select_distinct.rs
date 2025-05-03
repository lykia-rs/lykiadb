use crate::assert_parsing;
use crate::lang::compare_parsed_to_expected;
use serde_json::json;

assert_parsing! {
    implicit_all: {
        "SELECT surname from users;" => {
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
                          "@type": "SqlProjection::Expr",
                          "alias": null,
                          "expr": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "surname"
                            },
                            "tail": []
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
    explicit_all: {
        "SELECT all surname from users;" => {
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
                        "@type": "SqlDistinct::All"
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
                          "@type": "SqlProjection::Expr",
                          "alias": null,
                          "expr": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "surname"
                            },
                            "tail": []
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
    distinct: {
        "SELECT distinct surname from users;" => {
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
                        "@type": "SqlDistinct::Distinct"
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
                          "@type": "SqlProjection::Expr",
                          "alias": null,
                          "expr": {
                            "@type": "Expr::FieldPath",
                            "head": {
                              "@type": "Identifier",
                              "kind": "IdentifierKind::Plain",
                              "name": "surname"
                            },
                            "tail": []
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
