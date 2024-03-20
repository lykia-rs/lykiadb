use lykiadb_server::runtime::interpreter::test_helpers::get_runtime;
use lykiadb_server::runtime::types::RV;
use lykiadb_server::runtime::ServerSession;
use std::io::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::StreamExt as _;
use tracing::info;

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
                    let mut session = ServerSession::new(socket);
                    info!("Client {} connected", peer);
                    session.handle().await;
                    info!("Client {} disconnected", peer);
                });
            }
        }
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let (out, mut runtime) = get_runtime();

    let prog_0 = "
        function resolvedFirst() {
            var $a = \"global\";
            {
                function showA() {
                    TestUtils.out($a);
                };
            
                showA();
                var $a = \"block\";
                showA();
            }
        };
        resolvedFirst();
    ";
    runtime.interpret(prog_0).unwrap();
    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
    let prog_1 = "resolvedFirst();";
    let err_1 = runtime.interpret(prog_1).unwrap_err();

    println!("{:?}", err_1);
    out.write().unwrap().expect(vec![
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
        RV::Str(Arc::new("global".to_string())),
    ]);
    tracing_subscriber::fmt::init();
    Server::new()?.listen("0.0.0.0:19191").await?.serve().await
}
