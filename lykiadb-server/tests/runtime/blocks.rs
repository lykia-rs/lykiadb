use lykiadb_server::{engine::{
    error::ExecutionError,
    interpreter::{
        test_helpers::{exec_assert, get_runtime},
        InterpretError,
    },
}, value::types::RV};
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

#[test]
fn test_blocks_1() {
    let (out, mut runtime) = get_runtime();

    let prog_0 = "
        function fnBlock() {
            var $a = \"global\";
            {
                var $a = \"block\";
                TestUtils.out($a);
            }
            TestUtils.out($a);
        };
        fnBlock();
        TestUtils.out($a);
    ";

    let prog_1 = "TestUtils.out($a);";
    let expected_err_message = "Variable '$a' was not found";
    //
    let err_0 = runtime.interpret(prog_0).unwrap_err();

    if let ExecutionError::Interpret(InterpretError::Other { message }) = err_0 {
        assert_eq!(message, expected_err_message);
    }

    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("block".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
    //
    let err_1 = runtime.interpret(prog_1).unwrap_err();

    if let ExecutionError::Interpret(InterpretError::Other { message }) = err_1 {
        assert_eq!(message, expected_err_message);
    }

    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("block".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
}
