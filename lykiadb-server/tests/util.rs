use lykiadb_server::{
    engine::interpreter::test_helpers::{assert_err, assert_out, assert_out_str},
    value::RV,
};
use std::{collections::HashMap, sync::Arc};

fn run_plan(input: &str, output: &str, flags: HashMap<&str, &str>) {
    if flags.get("expect") == Some(&"error") {
        assert_err(input, output);
    } else {
        assert_out(input, vec![RV::Str(Arc::new(output.to_string()))]);
    }
}

fn run_interpreter(input: &str, output: &str, flags: HashMap<&str, &str>) {
    if flags.get("expect") == Some(&"error") {
        assert_err(input, output);
    } else {
        assert_out_str(input, output.to_string().split("\n").map(|x| x.to_string()).collect());
    }
}

pub fn run_test(input: &str) {
    let parts: Vec<&str> = input.split("#[").collect();

    for part in parts[1..].iter() {
        let directives_and_input = part.trim();

        let directives_end = directives_and_input
            .find('>')
            .unwrap_or(directives_and_input.len());

        let rest = directives_and_input[directives_end + 1..]
            .trim()
            .to_string();

        let flags = directives_and_input[..directives_end - 1]
            .trim()
            .split(',')
            .map(|flag| {
                let kv: Vec<&str> = flag.split('=').collect();
                return (kv[0].trim(), kv[1].trim());
            })
            .fold(std::collections::HashMap::new(), |mut acc, (k, v)| {
                acc.insert(k, v);
                acc
            });

        let io_parts: Vec<&str> = rest.split("---").collect();

        match flags.get("run") {
            Some(&"plan") => {
                run_plan(io_parts[0].trim(), io_parts[1].trim(), flags.clone());
            }
            Some(&"interpreter") => {
                run_interpreter(io_parts[0].trim(), io_parts[1].trim(), flags.clone());
            }
            _ => panic!("Unknown directive"),
        }
    }
}
