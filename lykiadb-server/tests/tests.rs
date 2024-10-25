#![recursion_limit = "192"]

mod runtime;
mod util;
mod planner {
    use test_each_file::test_each_file;
    use crate::util::run_test;

    test_each_file! { in "lykiadb-server/tests/planner" => run_test }
}
