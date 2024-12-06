mod util;
mod planner {
    use test_each_file::test_each_file;
    use crate::util::run_test_file;

    test_each_file! { in "lykiadb-server/tests/planner" => run_test_file }
}

mod interpreter {
    use test_each_file::test_each_file;
    use crate::util::run_test_file;

    test_each_file! { in "lykiadb-server/tests/interpreter" => run_test_file }
}