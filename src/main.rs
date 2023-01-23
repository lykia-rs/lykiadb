
fn main() {
    println!("sumer v0");
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 {
        println!("Too many args. Usage: sumer [script]");
        std::process::exit(64);
    }
    let filename_arg = args.get(1);
    match filename_arg {
        Some(filename) => run_file(filename),
        None => println!("REPL mode")
    }
}

fn run_file(filename: &String) {
    println!("filename: {filename}");
}