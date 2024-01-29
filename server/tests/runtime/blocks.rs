use lykiadb_server::runtime::{interpreter::test_helpers::exec_assert, types::RV};
use std::sync::Arc;

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
            RV::Str(Arc::new("inner a".to_string())),
            RV::Str(Arc::new("outer b".to_string())),
            RV::Str(Arc::new("global c".to_string())),
            RV::Str(Arc::new("outer a".to_string())),
            RV::Str(Arc::new("outer b".to_string())),
            RV::Str(Arc::new("global c".to_string())),
            RV::Str(Arc::new("global a".to_string())),
            RV::Str(Arc::new("global b".to_string())),
            RV::Str(Arc::new("global c".to_string())),
        ],
    );
}
