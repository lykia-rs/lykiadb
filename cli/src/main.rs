use std::{
    fs::File,
    io::{BufReader, Read},
};

use clap::Parser;
use lykiadb_server::net::{Message, Request};
use lykiadb_shell::ClientSession;
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the script to be executed
    filename: Option<String>,

    #[clap(short, long, default_value = "false")]
    print_ast: bool,
}

/*pub fn init() {
    let args = Args::parse();
    match args.filename {
        Some(filename) => run_file(&filename, args.print_ast),
        None => run_repl(),
    }
}*/

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

async fn run_file(filename: &str, print_ast: bool) {
    let file = File::open(filename).expect("File couldn't be opened.");

    let mut content: String = String::new();

    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");

    let socket = TcpStream::connect("localhost:19191").await.unwrap();

    let mut session = ClientSession::new(socket);
    loop {
        session
            .send(Message::Request(Request::Run(content.to_string())))
            .await
            .unwrap();

        let response = session.handle().await;
        println!("{:?}", response);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.filename {
        Some(filename) => run_file(&filename, args.print_ast).await,
        None => (),
    };
}
