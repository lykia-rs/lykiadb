#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::runtime::{tests::exec_assert, types::RV};

    #[test]
    fn test_higher_order_0() {
        exec_assert(
            "fun f($x, $q) {
            $x($q);
        };
        
        fun g($q) {
            print($q);
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
            "fun makeCounter() {
            var $i = 0;
            fun count() {
                $i = $i + 1;
                print($i);
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
          fun showA() {
            print($a);
          };
        
          showA();
          var $a = \"block\";
          showA();
        }",
            vec![
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("global".to_string())),
            ],
        );
    }

    #[test]
    fn test_resolving_read_1() {
        exec_assert(
            "var $a = \"global\";
        {
            fun showA() {
                print($a);
            };

            showA();
            var $a = \"block\";
            showA();
            fun showB() {
                print($a);
            };
            showB();
        }",
            vec![
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("block".to_string())),
            ],
        );
    }

    #[test]
    fn test_resolving_read_2() {
        exec_assert(
            "{
            var $a = \"global\";
            {
              fun showA() {
                print($a);
              };
          
              showA();
              var $a = \"block\";
              showA();
            }
          }",
            vec![
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("global".to_string())),
            ],
        );
    }

    #[test]
    fn test_resolving_write_0() {
        exec_assert(
            "var $a = \"global\";
        {
          fun showA() {
            print($a);
          };
        
          var $a = \"block\";
          
          fun showB() {
            print($a);
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
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("block".to_string())),
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("test".to_string())),
            ],
        );
    }

    #[test]
    fn test_anonymous_fn_0() {
        exec_assert(
            "var $pr = fun a() {
                    print(\"hello\");
                };

                $pr();
                a();
          ",
            vec![
                RV::Str(Rc::new("hello".to_string())),
                RV::Str(Rc::new("hello".to_string()))
            ],
        );
    }

    #[test]
    fn test_anonymous_fn_1() {
        exec_assert(
            "(fun a() {
                    print(\"hello\");
                  })();

                  a();
          ",
          vec![
            RV::Str(Rc::new("hello".to_string())),
            RV::Str(Rc::new("hello".to_string()))
        ],
        );
    }

    #[test]
    fn test_anonymous_fn_2() {
        exec_assert(
            "var $pr = fun() {
                    print(\"hello\");
                };

                $pr();
          ",
            vec![RV::Str(Rc::new("hello".to_string()))],
        );
    }

    #[test]
    fn test_anonymous_fn_3() {
        exec_assert(
            "(fun() {
                    print(\"hello\");
                  })();
          ",
            vec![RV::Str(Rc::new("hello".to_string()))],
        );
    }

    #[test]
    fn test_resolving_write_1() {
        exec_assert(
            "var $a = \"global\";
        {
          var $showA = fun() {
            print($a);
          };
        
          var $a = \"block\";
          
          var $showB = fun() {
            print($a);
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
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("block".to_string())),
                RV::Str(Rc::new("global".to_string())),
                RV::Str(Rc::new("test".to_string())),
            ],
        );
    }
}
