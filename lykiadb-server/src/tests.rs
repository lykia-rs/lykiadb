mod planner {
    use crate::session::SessionTester;
    use test_each_file::test_each_path;
    test_each_path! {
        in "lykiadb-server/src/query/plan/tests" => {
            |path: &std::path::Path| {
                let input = std::fs::read_to_string(path).expect("Failed to read test file");
                lykiadb_common::testing::TestRunner::new(Box::new(|| Box::new(SessionTester::new())))
                    .test_file_named(path.to_str().unwrap_or(""), &input)
            }
        }
    }
}

mod interpreter {
    use crate::session::SessionTester;
    use test_each_file::test_each_path;

    test_each_path! {
        in "lykiadb-server/src/interpreter/tests" => {
            |path: &std::path::Path| {
                let input = std::fs::read_to_string(path).expect("Failed to read test file");
                lykiadb_common::testing::TestRunner::new(Box::new(|| Box::new(SessionTester::new())))
                    .test_file_named(path.to_str().unwrap_or(""), &input)
            }
        }
    }
}
