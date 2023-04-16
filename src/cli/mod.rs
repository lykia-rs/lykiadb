use std::fs::File;
use std::io::{BufReader, Read, stdin, stdout, Write};
use crate::lang::execution::runtime::{Runtime, RuntimeMode};

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
    let file = File::open(filename).unwrap();
    let mut content: String = String::new();
    BufReader::new(file).read_to_string(&mut content).expect("File couldn't be read.");
    let mut runtime = Runtime::new(RuntimeMode::File);
    runtime.interpret(&content);
}

fn run_repl() {
    println!("REPL mode");
    let mut line = String::new();
    let mut runtime = Runtime::new(RuntimeMode::Repl);
    loop {
        print!("> ");
        let _ = stdout().flush();
        stdin().read_line(&mut line).expect("Invalid input");
        if line.is_empty() || line.trim() == "exit" {
            break;
        }
        runtime.interpret(&line);
        line.clear();
    }
}
