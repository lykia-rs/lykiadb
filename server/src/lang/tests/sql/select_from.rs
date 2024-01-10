#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users;" => {
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
                          "@type": "SqlProjection::All",
                          "collection": null
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
    multi_0: {
        "SELECT * from users, admins;" => {
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
                              "name": "users"
                            },
                            "namespace": null
                          },
                          {
                            "@type": "SqlCollectionIdentifier",
                            "alias": null,
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "admins"
                            },
                            "namespace": null
                          }
                        ]
                      },
                      "group_by": null,
                      "having": null,
                      "projection": [
                        {
                          "@type": "SqlProjection::All",
                          "collection": null
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
    multi_1: {
        "SELECT * from users u, admins as a;" => {
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
                            "alias": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "u"
                            },
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "users"
                            },
                            "namespace": null
                          },
                          {
                            "@type": "SqlCollectionIdentifier",
                            "alias": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "a"
                            },
                            "name": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "admins"
                            },
                            "namespace": null
                          }
                        ]
                      },
                      "group_by": null,
                      "having": null,
                      "projection": [
                        {
                          "@type": "SqlProjection::All",
                          "collection": null
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
    multi_2: {
        "SELECT * from (SELECT * from users) u;" => {
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
                            "@type": "SqlCollectionSubquery::Select",
                            "alias": {
                              "@type": "Identifier",
                              "dollar": false,
                              "name": "u"
                            },
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
                                      "@type": "SqlProjection::All",
                                      "collection": null
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
                      },
                      "group_by": null,
                      "having": null,
                      "projection": [
                        {
                          "@type": "SqlProjection::All",
                          "collection": null
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
