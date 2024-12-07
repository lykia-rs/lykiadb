use std::{
    fs::File,
    io::{stdin, stdout, BufReader, Read, Write},
};

use clap::Parser;
use lykiadb_connect::session::ClientSession;
use lykiadb_connect::{get_session, report_error, Message, Protocol, Request, Response};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the script to be executed
    filename: Option<String>,

    #[clap(short, long, default_value = "false")]
    print_ast: bool,
}

struct Shell;

impl Shell {
    async fn run_repl(&mut self, session: &mut impl ClientSession) {
        println!("REPL mode");
        let mut line = String::new();

        loop {
            print!("lykia > ");
            let _ = stdout().flush();
            stdin().read_line(&mut line).expect("Invalid input");
            if line.is_empty() || line.trim() == ".exit" {
                break;
            }

            let response = session
                .send_receive(Message::Request(Request::Run(line.to_string())))
                .await
                .unwrap();
            self.handle_response("prompt", &line, response);
            line.clear();
        }
    }

    async fn run_file(
        &mut self,
        session: &mut impl ClientSession,
        filename: &str,
        print_ast: bool,
    ) {
        let file = File::open(filename).expect("File couldn't be opened.");

        let mut content: String = String::new();

        BufReader::new(file)
            .read_to_string(&mut content)
            .expect("File couldn't be read.");

        let msg = if print_ast {
            Message::Request(Request::Ast(content.to_string()))
        } else {
            Message::Request(Request::Run(content.to_string()))
        };

        let response = session.send_receive(msg).await.unwrap();
        self.handle_response(filename, &content, response);
    }

    fn handle_response(&mut self, filename: &str, content: &str, response: Message) {
        match response {
            Message::Response(Response::Value(result)) => println!("{:?}", result),
            Message::Response(Response::Program(value)) => {
                println!("{}", serde_json::to_string_pretty(&value).unwrap())
            }
            Message::Response(Response::Error(err)) => {
                report_error(filename, content, err.clone(), &mut stdout())
            }
            _ => panic!(""),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut session = get_session("localhost:19191", Protocol::Tcp).await;
    let mut shell = Shell;
    match args.filename {
        Some(filename) => {
            shell
                .run_file(&mut session, &filename, args.print_ast)
                .await
        }
        None => shell.run_repl(&mut session).await,
    };
}
