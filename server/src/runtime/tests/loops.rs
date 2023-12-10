#[cfg(test)]
mod test {
    use crate::runtime::{tests::helpers::exec_assert, types::RV};
    use std::rc::Rc;

    #[test]
    fn test_loop_statements_0() {
        exec_assert(
            "for (var $i = 0; $i < 10; $i = $i + 1) {
            {
                {
                    if ($i == 2) continue;
                    if ($i == 8) break;
                    TestUtils.out($i);
                }
            }
        }",
            vec![
                RV::Num(0.0),
                RV::Num(1.0),
                RV::Num(3.0),
                RV::Num(4.0),
                RV::Num(5.0),
                RV::Num(6.0),
                RV::Num(7.0),
            ],
        );
    }

    #[test]
    fn test_loop_statements_1() {
        exec_assert(
            "for (var $i = 0; $i < 10000000; $i = $i+1) {
            if ($i > 17) break;
            if ($i < 15) continue;
            for (var $j = 0; $j < 10000000; $j = $j + 1) {
                TestUtils.out($i + \":\" + $j);
                if ($j > 2) break;
            }
        }",
            vec![
                RV::Str(Rc::new("15:0".to_string())),
                RV::Str(Rc::new("15:1".to_string())),
                RV::Str(Rc::new("15:2".to_string())),
                RV::Str(Rc::new("15:3".to_string())),
                RV::Str(Rc::new("16:0".to_string())),
                RV::Str(Rc::new("16:1".to_string())),
                RV::Str(Rc::new("16:2".to_string())),
                RV::Str(Rc::new("16:3".to_string())),
                RV::Str(Rc::new("17:0".to_string())),
                RV::Str(Rc::new("17:1".to_string())),
                RV::Str(Rc::new("17:2".to_string())),
                RV::Str(Rc::new("17:3".to_string())),
            ],
        );
    }

    #[test]
    fn test_loop_statements_2() {
        exec_assert(
            "var $q = 0;

        for (var $i = 0; $i < 10000000; $i = $i+1) {
            break;
            $q = $q + 1;
            TestUtils.out(\"Shouldn't be shown\");
        }
        
        {
            {
                {
                    {
                        {
                            {
                                {
                                    TestUtils.out($q);
                                }
                            }
                        }
                    }
                }
            }
        }",
            vec![RV::Num(0.0)],
        );
    }
}
