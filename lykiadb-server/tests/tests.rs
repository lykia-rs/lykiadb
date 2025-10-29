mod planner {
    use lykiadb_server::engine::RuntimeTester;
    use test_each_file::test_each_file;
    test_each_file! {
        in "lykiadb-server/tests/planner" => {
            |input| {
                lykiadb_common::TestRunner::new(Box::new(|| Box::new(RuntimeTester::new()))).test_file(input)
            }
        }
    }
}

mod interpreter {
    use lykiadb_server::engine::RuntimeTester;
    use test_each_file::test_each_file;

    test_each_file! {
        in "lykiadb-server/tests/interpreter" => {
            |input| {
                lykiadb_common::TestRunner::new(Box::new(|| Box::new(RuntimeTester::new()))).test_file(input)
            }
        }
    }
}
