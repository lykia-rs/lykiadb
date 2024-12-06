use lykiadb_server::{
    engine::{error::ExecutionError, interpreter::test_helpers::{assert_err, assert_out, get_runtime}},
    value::RV,
};
use std::{collections::HashMap, sync::Arc};
use pretty_assertions::assert_eq;

fn run_plan(case_parts: Vec<String>, flags: HashMap<&str, &str>) {
    for chunk in case_parts.chunks(2) {
        if flags.get("expect") == Some(&"error") {
            assert_err(&chunk[0], &chunk[1]);
        } else {
            assert_out(&chunk[0], vec![RV::Str(Arc::new(chunk[1].to_string()))]);
        }
    }
}

fn run_case(case_parts: Vec<String>, _: HashMap<&str, &str>) {
    let (out, mut runtime) = get_runtime();

    assert!(case_parts.len() > 1, "Expected at least one input/output pair");

    let mut errors: Vec<ExecutionError> = vec![];

    let result = runtime.interpret(&case_parts[0]);

    if let Err(err) = result {
        errors.push(err);
    }

    for part in &case_parts[1..] {
        if part.starts_with("err") {
            assert_eq!(errors.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n"), part[3..].trim());
        }

        else if part.starts_with(">") {
            let result = runtime.interpret(part[1..].trim());

            if let Err(err) = result {
                errors.push(err);
            }
        }
        else {
            out.write().unwrap().expect_str(part.to_string().split("\n").map(|x| x.to_string()).collect());
        }        
    }
}

pub fn run_test_file(input: &str) {
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

        let case_parts = rest.split("---").map(|x| x.trim().to_string()).collect();

        match flags.get("run") {
            Some(&"plan") => {
                run_plan(case_parts, flags.clone());
            }
            Some(&"interpreter") => {
                run_case(case_parts, flags.clone());
            }
            _ => panic!("Unknown directive"),
        }
    }
}
