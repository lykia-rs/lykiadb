use std::sync::Arc;
use lykiadb_server::{engine::interpreter::test_helpers::{assert_err, assert_out}, value::RV};

fn expect_plan(query: &str, expected_plan: &str) {
    assert_out(query,
        vec![
            RV::Str(Arc::new(expected_plan.to_string())),
        ],
    );
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

        let flags = directives_and_input[..directives_end - 1].trim().split(",").map(|flag| {
            let kv: Vec<&str> = flag.split("=").collect();
            return (kv[0].trim(), kv[1].trim());
        }).fold(std::collections::HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k, v);
            acc
        });

        let io_parts: Vec<&str> = rest.split("---").collect();
        
        if flags.get("expect") == Some(&"error") {
            assert_err(io_parts[0].trim(),
            io_parts[1].trim()
            );
            continue;
        }
        
        expect_plan(io_parts[0].trim(), io_parts[1].trim());
    }
}
