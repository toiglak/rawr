use futures::{future, SinkExt, StreamExt};
use schemas::service::{TestRequest, TestServer, TestService};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Clone)]
struct ServiceImpl {}

impl TestService for ServiceImpl {
    async fn say_hello(&self, arg: String) -> String {
        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        format!("Hello, {}!", arg)
    }
}

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    let (mut client_transport, server_transport) = rawr::transport();
    let server_task = TestServer::new(server_transport, ServiceImpl {});
    tokio::spawn(server_task);

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut write, mut read) = ws_stream.split();
        let (req_tx, res_rx) = &mut client_transport; // todo: you should probably multiplex here

        let handle_incoming = async {
            while let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                let msg: rawr::Request<TestRequest> =
                    serde_json::from_str(&msg.to_string()).unwrap();
                req_tx.send(msg);
            }
        };

        let handle_outgoing = async {
            while let Some(res) = res_rx.recv().await {
                let res = Message::text(serde_json::to_string(&res).unwrap());
                write.send(res).await.unwrap();
            }
        };

        future::join(handle_incoming, handle_outgoing).await;
    }
}
