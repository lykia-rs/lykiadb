use ::std::time::Instant;
use lykiadb_common::comm::tcp::TcpConnection;
use lykiadb_common::comm::{CommunicationError, Message, Request, Response};
use lykiadb_server::interpreter::Interpreter;
use lykiadb_server::session::{Session, SessionMode};
use tokio::net::TcpStream;
use tracing::{error, info};
use std::io::Error;
use tokio::net::TcpListener;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::TcpListenerStream;

const ASCII_ART: &str = r"
$$\                 $$\       $$\           $$$$$$$\  $$$$$$$\
$$ |                $$ |      \__|          $$  __$$\ $$  __$$\
$$ |      $$\   $$\ $$ |  $$\ $$\  $$$$$$\  $$ |  $$ |$$ |  $$ |
$$ |      $$ |  $$ |$$ | $$  |$$ | \____$$\ $$ |  $$ |$$$$$$$\ |
$$ |      $$ |  $$ |$$$$$$  / $$ | $$$$$$$ |$$ |  $$ |$$  __$$\
$$ |      $$ |  $$ |$$  _$$<  $$ |$$  __$$ |$$ |  $$ |$$ |  $$ |
$$$$$$$$\ \$$$$$$$ |$$ | \$$\ $$ |\$$$$$$$ |$$$$$$$  |$$$$$$$  |
\________| \____$$ |\__|  \__|\__| \_______|\_______/ \_______/
          $$\   $$ |
          \$$$$$$  |
           \______/
";

struct Server {
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new() -> Result<Self, Error> {
        Ok(Server { listener: None })
    }

    pub async fn listen(mut self, addr: &str) -> Result<Self, Error> {
        let listener = TcpListener::bind(addr).await?;
        println!("{ASCII_ART}");
        info!("Listening on {}", listener.local_addr()?);
        self.listener = Some(listener);
        Ok(self)
    }

    pub async fn serve(self) -> Result<(), Error> {
        if let Some(listener) = self.listener {
            let mut stream = TcpListenerStream::new(listener);
            while let Some(socket) = stream.try_next().await? {
                let peer = socket.peer_addr()?;
                tokio::spawn(async move {
                    let mut session = Connection::new(socket);
                    info!("Client {} connected", peer);
                    session.handle().await;
                    info!("Client {} disconnected", peer);
                });
            }
        }
        Ok(())
    }
}

pub struct Connection<'v> {
    conn: TcpConnection,
    session: Session<'v>,
}

impl<'v> Connection<'v> {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            conn: TcpConnection::new(stream),
            session: Session::new(SessionMode::File, Interpreter::new(None, true)),
        }
    }

    pub async fn handle(&mut self) {
        while let Some(message) = self.conn.read().await.unwrap() {
            // Here we measure the time it takes to process a message

            match &message {
                Message::Request(req) => match req {
                    Request::Run(command) => {
                        let start = Instant::now();
                        let execution = self.session.interpret(command);
                        let elapsed = start.elapsed();
                        info!("{:?} (took {:?})", message, elapsed);
                        let response = if execution.is_ok() {
                            Response::Value(
                                execution.unwrap().to_string().into(),
                                elapsed.as_millis() as u64,
                            )
                        } else {
                            Response::Error(
                                execution.err().unwrap().generalize(),
                                elapsed.as_millis() as u64,
                            )
                        };

                        self.conn.write(Message::Response(response)).await.unwrap();
                    }
                },
                _ => error!("Unsupported message type"),
            }
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), CommunicationError> {
        self.conn.write(msg).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    Server::new()?.listen("0.0.0.0:19191").await?.serve().await
}
