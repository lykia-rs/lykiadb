use std::{
    fs::File,
    io::{BufReader, Read, stdout},
};

use clap::Parser;
use lykiadb_common::comm::{
    Message, Request, Response,
    client::{ClientSession, Protocol, get_session},
};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the script to be executed
    filename: Option<String>,
}

struct Shell;

impl Shell {
    async fn run_repl(&mut self, session: &mut impl ClientSession) {
        println!("REPL mode");

        let mut rl = DefaultEditor::new().unwrap();
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    if line.trim() == ".exit" {
                        break;
                    }

                    let response = session
                        .send_receive(Message::Request(Request::Run(line.to_string())))
                        .await
                        .unwrap();
                    self.handle_response("prompt", &line, response);

                    rl.add_history_entry(line.as_str()).unwrap();
                }
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => break,
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
    }

    async fn run_file(&mut self, session: &mut impl ClientSession, filename: &str) {
        let file = File::open(filename).expect("File couldn't be opened.");

        let mut content: String = String::new();

        BufReader::new(file)
            .read_to_string(&mut content)
            .expect("File couldn't be read.");

        let msg = Message::Request(Request::Run(content.to_string()));

        let response = session.send_receive(msg).await.unwrap();
        self.handle_response(filename, &content, response);
    }

    fn handle_response(&mut self, filename: &str, content: &str, response: Message) {
        match response {
            Message::Response(Response::Value(value)) |
            Message::Response(Response::Program(value)) => {
                match value {
                    serde_json::Value::String(str) => {
                        println!("{}", str)
                    },
                    _ => {
                        println!("{}", serde_json::to_string_pretty(&value).unwrap())
                    }
                }
            }
            Message::Response(Response::Error(err)) => {
                err.report(filename, content, &mut stdout());
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
        Some(filename) => shell.run_file(&mut session, &filename).await,
        None => shell.run_repl(&mut session).await,
    };
}
