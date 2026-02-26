mod planner {
    use lykiadb_server::session::SessionTester;
    use test_each_file::test_each_file;
    test_each_file! {
        in "lykiadb-server/tests/planner" => {
            |input| {
                lykiadb_common::testing::TestRunner::new(Box::new(|| Box::new(SessionTester::new()))).test_file(input)
            }
        }
    }
}

mod interpreter {
    use lykiadb_server::session::SessionTester;
    use test_each_file::test_each_file;

    test_each_file! {
        in "lykiadb-server/tests/interpreter" => {
            |input| {
                lykiadb_common::testing::TestRunner::new(Box::new(|| Box::new(SessionTester::new()))).test_file(input)
            }
        }
    }
}
