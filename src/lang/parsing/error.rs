use std::process::exit;

pub fn parse_err(msg: &str, line: u32) {
    println!("{} at line {}", msg, line + 1);
    exit(1);
}

