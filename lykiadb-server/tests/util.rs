use lykiadb_lang::{ast::stmt::Stmt, parser::program::Program};
use lykiadb_server::{engine::interpreter::ExecutionContext, plan::planner::Planner};
use pretty_assertions::assert_eq;

fn expect_plan(query: &str, expected_plan: &str) {
    let ctx = ExecutionContext::new(None);
    let mut planner = Planner::new(ctx);
    let program = query.parse::<Program>().unwrap();
    match *program.get_root() {
        Stmt::Program { body, .. } if matches!(body.get(0), Some(Stmt::Expression { .. })) => {
            if let Some(Stmt::Expression { expr, .. }) = body.get(0) {
                let generated_plan = planner.build(&expr).unwrap();
                assert_eq!(expected_plan, generated_plan.to_string().trim());
            }
        }
        _ => panic!("Expected expression statement."),
    }
}

pub fn run_test(input: &str) {
    let parts: Vec<&str> = input.split("#[").collect();

    for part in parts[1..].iter() {
        let directives_and_input = part.trim();
        let directives_end = directives_and_input.find('>').unwrap_or(directives_and_input.len());
        let rest = directives_and_input[directives_end+1..].trim().to_string();
        let io_parts: Vec<&str> = rest.split("---").collect();
        expect_plan(&io_parts[0].trim(),&io_parts[1].trim());
    }
}