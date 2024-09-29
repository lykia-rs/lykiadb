use lykiadb_server::{engine::interpreter::test_helpers::exec_assert, value::RV};
use std::sync::Arc;

#[test]
fn test_if() {
    exec_assert(
        "var $a = 30;

    if ($a > 50) {
        TestUtils.out(\"> 50\");
    }
    else if ($a > 20) {
        TestUtils.out(\"50 > $a > 20\");
    }
    else {
        TestUtils.out(\"< 20\");
    }",
        vec![RV::Str(Arc::new("50 > $a > 20".to_string()))],
    );
}
