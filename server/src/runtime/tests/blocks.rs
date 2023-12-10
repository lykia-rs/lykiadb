#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::runtime::{tests::helpers::exec_assert, types::RV};

    #[test]
    fn test_blocks_0() {
        exec_assert(
            "var $a = \"global a\";
        var $b = \"global b\";
        var $c = \"global c\";
        {
           var $a = \"outer a\";
           var $b = \"outer b\";
           {
              var $a = \"inner a\";
              TestUtils.out($a);
              TestUtils.out($b);
              TestUtils.out($c);
           }
           TestUtils.out($a);
           TestUtils.out($b);
           TestUtils.out($c);
        }
        TestUtils.out($a);
        TestUtils.out($b);
        TestUtils.out($c);",
            vec![
                RV::Str(Rc::new("inner a".to_string())),
                RV::Str(Rc::new("outer b".to_string())),
                RV::Str(Rc::new("global c".to_string())),
                RV::Str(Rc::new("outer a".to_string())),
                RV::Str(Rc::new("outer b".to_string())),
                RV::Str(Rc::new("global c".to_string())),
                RV::Str(Rc::new("global a".to_string())),
                RV::Str(Rc::new("global b".to_string())),
                RV::Str(Rc::new("global c".to_string())),
            ],
        );
    }
}
