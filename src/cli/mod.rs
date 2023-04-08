use std::io::{stdin, stdout, Write};
use crate::lang::parsing::expr::{Printer, Visitor};
use crate::lang::parsing::parser::Parser;
use crate::lang::parsing::scanner::Scanner;

pub fn init() {
    println!("sumer v0");
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 {
        println!("Too many args. Usage: sumer [script]");
        std::process::exit(64);
    }
    let filename_arg = args.get(1);
    match filename_arg {
        Some(filename) => run_file(filename),
        None => run_repl()
    }
}

fn run_file(filename: &str) {
    println!("filename: {filename}");
}

fn run_repl() {
    println!("REPL mode");
    let mut line = String::new();
    loop {
        print!("> ");
        let _ = stdout().flush();
        stdin().read_line(&mut line).expect("Invalid input");
        if line.is_empty() || line.trim() == "exit" {
            break;
        }
        run(&line);
        line.clear();
    }
}

fn run(source: &str) {
    let tokens = Scanner::scan(&source);
    for t in &tokens {
        println!("{:?}", t);
    }
    let ast = Parser::parse(&tokens);
    println!("{}", Printer::new().visit_expr(&ast));
}