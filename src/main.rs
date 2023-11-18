extern crate core;

mod cli;
pub mod lang;
pub mod runtime;
pub mod util;

fn main() {
    cli::init();
}
