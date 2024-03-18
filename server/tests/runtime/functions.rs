use lykiadb_server::runtime::{
    interpreter::test_helpers::{exec_assert, get_runtime},
    types::RV,
};
use std::sync::Arc;

#[test]
fn test_higher_order_0() {
    exec_assert(
        "function f($x, $q) {
        $x($q);
    };
    
    function g($q) {
        TestUtils.out($q);
    };
    
    for (var $i=0; $i<10; $i = $i + 1) {
        f(g, $i);
    }",
        vec![
            RV::Num(0.0),
            RV::Num(1.0),
            RV::Num(2.0),
            RV::Num(3.0),
            RV::Num(4.0),
            RV::Num(5.0),
            RV::Num(6.0),
            RV::Num(7.0),
            RV::Num(8.0),
            RV::Num(9.0),
        ],
    );
}

#[test]
fn test_high_order_1() {
    exec_assert(
        "function makeCounter() {
        var $i = 0;
        function count() {
            $i = $i + 1;
            TestUtils.out($i);
        };
    
        return count;
    };
    var $count = makeCounter();
    $count();
    $count();",
        vec![RV::Num(1.0), RV::Num(2.0)],
    );
}

#[test]
fn test_resolving_read_0() {
    exec_assert(
        "var $a = \"global\";
    {
        function showA() {
        TestUtils.out($a);
        };
    
        showA();
        var $a = \"block\";
        showA();
    }",
        vec![
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("global".to_string())),
        ],
    );
}

#[test]
fn test_resolving_read_1() {
    exec_assert(
        "var $a = \"global\";
    {
        function showA() {
            TestUtils.out($a);
        };

        showA();
        var $a = \"block\";
        showA();
        function showB() {
            TestUtils.out($a);
        };
        showB();
    }",
        vec![
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("block".to_string())),
        ],
    );
}

#[test]
fn test_resolving_read_2() {
    exec_assert(
        "{
        var $a = \"global\";
        {
            function showA() {
            TestUtils.out($a);
            };
        
            showA();
            var $a = \"block\";
            showA();
        }
        }",
        vec![
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("global".to_string())),
        ],
    );
}

#[test]
fn test_resolving_write_0() {
    exec_assert(
        "var $a = \"global\";
    {
        function showA() {
        TestUtils.out($a);
        };
    
        var $a = \"block\";
        
        function showB() {
        TestUtils.out($a);
        };
    
        //
        showA();
        showB();
        //
        $a = \"test\";
        //
        showA();
        showB();
    }",
        vec![
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("block".to_string())),
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("test".to_string())),
        ],
    );
}

#[test]
fn test_anonymous_fn_0() {
    exec_assert(
        "var $pr = function a() {
                TestUtils.out(\"hello\");
            };

            $pr();
            a();
        ",
        vec![
            RV::Str(Arc::new("hello".to_string())),
            RV::Str(Arc::new("hello".to_string())),
        ],
    );
}

#[test]
fn test_anonymous_fn_1() {
    exec_assert(
        "(function a() {
                TestUtils.out(\"hello\");
                })();

                a();
        ",
        vec![
            RV::Str(Arc::new("hello".to_string())),
            RV::Str(Arc::new("hello".to_string())),
        ],
    );
}

#[test]
fn test_anonymous_fn_2() {
    exec_assert(
        "var $pr = function() {
                TestUtils.out(\"hello\");
            };

            $pr();
        ",
        vec![RV::Str(Arc::new("hello".to_string()))],
    );
}

#[test]
fn test_anonymous_fn_3() {
    exec_assert(
        "(function() {
                TestUtils.out(\"hello\");
                })();
        ",
        vec![RV::Str(Arc::new("hello".to_string()))],
    );
}

#[test]
fn test_resolving_write_1() {
    exec_assert(
        "var $a = \"global\";
    {
        var $showA = function() {
        TestUtils.out($a);
        };
    
        var $a = \"block\";
        
        var $showB = function() {
        TestUtils.out($a);
        };
    
        //
        $showA();
        $showB();
        //
        $a = \"test\";
        //
        $showA();
        $showB();
    }",
        vec![
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("block".to_string())),
            RV::Str(Arc::new("global".to_string())),
            RV::Str(Arc::new("test".to_string())),
        ],
    );
}

#[test]
fn test_resolve_object() {
    exec_assert(
        "
        
        var $text = 'outer $text';
        
        var $a = {
            myFun: function() {
                function pre_define() {
                    TestUtils.out($text);
                };
                pre_define();
                //
                var $text = 'inner $text';
                //
                function post_define() {
                    TestUtils.out($text);
                };
                post_define();
            }
        };
        $a.myFun();
        ",
        vec![
            RV::Str(Arc::new("outer $text".to_string())),
            RV::Str(Arc::new("inner $text".to_string())),
        ],
    );
}

#[test]
fn test_resolve_deeper_object() {
    exec_assert(
        "
        var $text = 'outer $text';
        
        var $a = {
            b: {
                c0: {
                    myFun: function() {
                        function pre_define() {
                            TestUtils.out($text);
                        };
                        pre_define();
                        //
                        var $text = 'c0 inner $text';
                        //
                        function post_define() {
                            TestUtils.out($text);
                        };
                        post_define();
                    }
                },
                c1: {
                    myFun: function() {
                        function pre_define() {
                            TestUtils.out($text);
                        };
                        pre_define();
                        //
                        var $text = 'c1 inner $text';
                        //
                        function post_define() {
                            TestUtils.out($text);
                        };
                        post_define();
                    }
                }
            }
        };
        $a.b.c0.myFun();
        $a.b.c1.myFun();
        ",
        vec![
            RV::Str(Arc::new("outer $text".to_string())),
            RV::Str(Arc::new("c0 inner $text".to_string())),
            RV::Str(Arc::new("outer $text".to_string())),
            RV::Str(Arc::new("c1 inner $text".to_string())),
        ],
    );
}

#[test]
fn test_resolve_multiple_programs() {
    let (out, mut runtime) = get_runtime();

    let prog_0 = "
        function resolvedFirst() {
            var $a = \"global\";
            {
                function showA() {
                    TestUtils.out($a);
                };
            
                showA();
                var $a = \"block\";
                showA();
            }
        };
        resolvedFirst();
    ";
    runtime.interpret(prog_0).unwrap();
    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
    let prog_1 = "resolvedFirst();";
    let err_1 = runtime.interpret(prog_1).unwrap_err();

    println!("{:?}", err_1);
    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
}
