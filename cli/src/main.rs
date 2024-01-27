use std::{fs::File, io::{BufReader, Read}};

use clap::Parser;
use liblykia::Request;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the script to be executed
    filename: Option<String>,

    #[clap(short, long, default_value = "false")]
    print_ast: bool,
}

pub fn init() {
    let args = Args::parse();
    match args.filename {
        Some(filename) => run_file(&filename, args.print_ast),
        None => run_repl(),
    }
}

fn run_file(filename: &str, print_ast: bool) {
    let file = File::open(filename).expect("File couldn't be opened.");

    let mut content: String = String::new();

    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");

    // send file to server
    println!("{:?}", Request::Execute(content));
}

fn run_repl() {
    /*println!("REPL mode");
    let mut line = String::new();
    let mut runtime = Runtime::new(RuntimeMode::Repl);
    loop {
        print!("lykia > ");
        let _ = stdout().flush();
        stdin().read_line(&mut line).expect("Invalid input");
        if line.is_empty() || line.trim() == ".exit" {
            break;
        }
        // send line to server
        line.clear();
    }*/
}

fn main() {
    init();
}
