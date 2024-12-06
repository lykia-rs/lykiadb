mod planner {
    use lykiadb_server::engine::test_helpers::RuntimeTester;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/planner" => RuntimeTester::test_file }
}

mod interpreter {
    use lykiadb_server::engine::test_helpers::RuntimeTester;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/interpreter" => RuntimeTester::test_file }
}