use futures::{SinkExt, StreamExt, future};
use schemas::enumeration::EnumAdjacentlyTagged;
use schemas::service::{TestRequest, TestServer, TestService};
use schemas::structure::Structure;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone)]
struct ServiceImpl {}

impl TestService for ServiceImpl {
    async fn say_hello(&self, arg: String) -> String {
        format!("Hello, {}!", arg)
    }

    async fn complex(&self, mut input: Structure, n: i32) -> Structure {
        input.count += n;
        input
    }

    async fn ping_enum(&self, arg: EnumAdjacentlyTagged) -> EnumAdjacentlyTagged {
        arg
    }
}

#[tokio::main]
async fn main() {
    unsafe { std::env::set_var("RUST_LOG", "debug") };
    env_logger::init();

    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    let (mut client_transport, server_transport) = rawr::transport();
    let server_task = TestServer::new(server_transport, ServiceImpl {});
    tokio::spawn(server_task);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut write, mut read) = ws_stream.split();
        // TODO: You should probably multiplex here, handle multiple clients concurrently.
        let (server_tx, server_rx) = &mut client_transport;

        let handle_incoming = Box::pin(async {
            while let Some(msg) = read.next().await {
                let msg = match msg {
                    Ok(Message::Close(_))
                    | Err(Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)) => break,
                    Ok(msg) => msg,
                    Err(e) => panic!("{:?}", e),
                };
                log::debug!("Received message: {}", msg);
                let msg: rawr::Request<TestRequest> =
                    serde_json::from_str(&msg.to_string()).unwrap();
                server_tx.send(msg);
            }
        });

        let handle_outgoing = Box::pin(async {
            while let Some(res) = server_rx.recv().await {
                let res = Message::text(serde_json::to_string(&res).unwrap());
                log::debug!("Sending message: {}", res);
                write.send(res).await.unwrap();
            }
        });

        future::select(handle_incoming, handle_outgoing).await;
    }
}
