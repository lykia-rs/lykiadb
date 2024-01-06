#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    limit_5: {
        "SELECT * from users limit 5;" => {
          "Program": {
            "body": [
              {
                "Expression": {
                  "expr": {
                    "Select": {
                      "query": {
                        "compound": [],
                        "core": {
                          "distinct": "ImplicitAll",
                          "from": {
                            "Group": [
                              {
                                "Collection": {
                                  "alias": null,
                                  "name": {
                                    "lexeme": "users",
                                    "literal": {
                                      "Str": "users"
                                    },
                                    "tok_type": {
                                      "Identifier": {
                                        "dollar": false
                                      }
                                    }
                                  },
                                  "namespace": null
                                }
                              }
                            ]
                          },
                          "group_by": null,
                          "having": null,
                          "projection": [
                            {
                              "All": {
                                "collection": null
                              }
                            }
                          ],
                          "where": null
                        },
                        "limit": {
                          "count": {
                            "Default": {
                              "Literal": {
                                "raw": "5",
                                "value": {
                                  "Num": 5.0
                                }
                              }
                            }
                          },
                          "offset": null
                        },
                        "order_by": null
                      }
                    }
                  }
                }
              }
            ]
          }
        }
    },
    limit_5_offset_10: {
        "SELECT * from users limit 5 offset 10;" => {
          "Program": {
            "body": [
              {
                "Expression": {
                  "expr": {
                    "Select": {
                      "query": {
                        "compound": [],
                        "core": {
                          "distinct": "ImplicitAll",
                          "from": {
                            "Group": [
                              {
                                "Collection": {
                                  "alias": null,
                                  "name": {
                                    "lexeme": "users",
                                    "literal": {
                                      "Str": "users"
                                    },
                                    "tok_type": {
                                      "Identifier": {
                                        "dollar": false
                                      }
                                    }
                                  },
                                  "namespace": null
                                }
                              }
                            ]
                          },
                          "group_by": null,
                          "having": null,
                          "projection": [
                            {
                              "All": {
                                "collection": null
                              }
                            }
                          ],
                          "where": null
                        },
                        "limit": {
                          "count": {
                            "Default": {
                              "Literal": {
                                "raw": "5",
                                "value": {
                                  "Num": 5.0
                                }
                              }
                            }
                          },
                          "offset": {
                            "Default": {
                              "Literal": {
                                "raw": "10",
                                "value": {
                                  "Num": 10.0
                                }
                              }
                            }
                          }
                        },
                        "order_by": null
                      }
                    }
                  }
                }
              }
            ]
          }
        }
    },
    limit_10_offset_5: {
        "SELECT * from users limit 5, 10;" => {
          "Program": {
            "body": [
              {
                "Expression": {
                  "expr": {
                    "Select": {
                      "query": {
                        "compound": [],
                        "core": {
                          "distinct": "ImplicitAll",
                          "from": {
                            "Group": [
                              {
                                "Collection": {
                                  "alias": null,
                                  "name": {
                                    "lexeme": "users",
                                    "literal": {
                                      "Str": "users"
                                    },
                                    "tok_type": {
                                      "Identifier": {
                                        "dollar": false
                                      }
                                    }
                                  },
                                  "namespace": null
                                }
                              }
                            ]
                          },
                          "group_by": null,
                          "having": null,
                          "projection": [
                            {
                              "All": {
                                "collection": null
                              }
                            }
                          ],
                          "where": null
                        },
                        "limit": {
                          "count": {
                            "Default": {
                              "Literal": {
                                "raw": "10",
                                "value": {
                                  "Num": 10.0
                                }
                              }
                            }
                          },
                          "offset": {
                            "Default": {
                              "Literal": {
                                "raw": "5",
                                "value": {
                                  "Num": 5.0
                                }
                              }
                            }
                          }
                        },
                        "order_by": null
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
