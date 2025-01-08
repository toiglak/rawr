use futures::{future, stream, SinkExt, StreamExt};
use schemas::service::{TestClient, TestResponse, TestService};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");

    let (client_transport, server_transport) = rawr::transport();

    // Create server and client.
    let (client, client_task) = TestClient::new(client_transport);

    // Run client task.
    tokio::spawn(client_task);

    // Handle communication with the server.
    tokio::spawn(async move {
        let ws = connect_async(&format!("ws://{}", addr)).await.unwrap().0;

        let (mut out, mut inc) = ws.split();
        let (mut req_rx, res_tx) = server_transport;

        let handle_incoming = async {
            while let Some(msg) = inc.next().await {
                let msg = msg.unwrap();
                let msg: rawr::Response<TestResponse> =
                    serde_json::from_str(&msg.to_string()).unwrap();
                res_tx.send(msg);
            }
        };

        let handle_outgoing = async {
            while let Some(req) = req_rx.recv().await {
                let req = Message::text(serde_json::to_string(&req).unwrap());
                out.send(req).await.unwrap();
            }
        };

        future::join(handle_incoming, handle_outgoing).await;
    });

    // Make 10 concurrent requests to the server.
    let client = &client;
    let make_request = |i| async move {
        let response = client.say_hello(format!("World {}", i + 1)).await;
        println!("{}: {}", i + 1, response);
    };
    stream::iter(0..10)
        .for_each_concurrent(None, make_request)
        .await;
}
