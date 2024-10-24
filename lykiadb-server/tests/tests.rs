#![recursion_limit = "192"]
use util::run_test;
use test_each_file::test_each_file;

mod runtime;
mod util;

test_each_file! { in "lykiadb-server/tests/planner" => run_test }
