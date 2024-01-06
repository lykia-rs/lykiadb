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
            "Program": {
              "body": [
                {
                  "Declaration": {
                    "dst": {
                      "lexeme": "$result",
                      "literal": {
                        "Str": "$result"
                      },
                      "tok_type": {
                        "Identifier": {
                          "dollar": true
                        }
                      }
                    },
                    "expr": {
                      "Insert": {
                        "command": {
                          "collection": {
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
                            "namespace": {
                              "lexeme": "db",
                              "literal": {
                                "Str": "db"
                              },
                              "tok_type": {
                                "Identifier": {
                                  "dollar": false
                                }
                              }
                            }
                          },
                          "values": {
                            "Values": [
                              {
                                "Default": {
                                  "Literal": {
                                    "raw": "",
                                    "value": {
                                      "Object": {
                                        "age": {
                                          "Literal": {
                                            "raw": "42",
                                            "value": {
                                              "Num": 42.0
                                            }
                                          }
                                        },
                                        "name": {
                                          "Literal": {
                                            "raw": "John",
                                            "value": {
                                              "Str": "John"
                                            }
                                          }
                                        },
                                        "surname": {
                                          "Literal": {
                                            "raw": "Doe",
                                            "value": {
                                              "Str": "Doe"
                                            }
                                          }
                                        }
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
                  }
                }
              ]
            }
          }
    }
}
