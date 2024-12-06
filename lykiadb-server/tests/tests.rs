#![recursion_limit = "192"]

mod util;
mod planner {
    use crate::util::run_test;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/planner" => run_test }
}

mod interpreter {
    use crate::util::run_test;
    use test_each_file::test_each_file;

    test_each_file! { in "lykiadb-server/tests/interpreter" => run_test }
}