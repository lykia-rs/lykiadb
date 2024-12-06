use lykiadb_server::{
    engine::interpreter::test_helpers::{assert_err, assert_out, get_runtime},
    value::RV,
};
use std::{collections::HashMap, sync::Arc};

fn run_plan(io_parts: Vec<String>, flags: HashMap<&str, &str>) {
    for chunk in io_parts.chunks(2) {
        if flags.get("expect") == Some(&"error") {
            assert_err(&chunk[0], &chunk[1]);
        } else {
            assert_out(&chunk[0], vec![RV::Str(Arc::new(chunk[1].to_string()))]);
        }
    }
}

fn run_interpreter(io_parts: Vec<String>, flags: HashMap<&str, &str>) {
    let (out, mut runtime) = get_runtime();

    for chunk in io_parts.chunks(2) {
        if flags.get("expect") == Some(&"error") {
            assert_err(&chunk[0], &chunk[1]);
        } else {
            runtime.interpret(&chunk[0]).unwrap();
            out.write().unwrap().expect_str(chunk[1].to_string().split("\n").map(|x| x.to_string()).collect());
        }
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

        let io_parts = rest.split("---").map(|x| x.trim().to_string()).collect();

        match flags.get("run") {
            Some(&"plan") => {
                run_plan(io_parts, flags.clone());
            }
            Some(&"interpreter") => {
                run_interpreter(io_parts, flags.clone());
            }
            _ => panic!("Unknown directive"),
        }
    }
}
