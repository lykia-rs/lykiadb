mod planner {
    use lykiadb_server::engine::interpreter::test_helpers::InterpreterTester;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/planner" => InterpreterTester::test_file }
}

mod interpreter {
    use lykiadb_server::engine::interpreter::test_helpers::InterpreterTester;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/interpreter" => InterpreterTester::test_file }
}