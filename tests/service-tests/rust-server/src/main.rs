use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

type TestStruct = schemas::structure::Structure;

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut write, mut read) = ws_stream.split();

        while let Some(Ok(message)) = read.next().await {
            if message.is_close() {
                break;
            }

            // Deserialize Structure from the client.
            let message = message.to_text().unwrap();
            let message: TestStruct = serde_json::from_str(message).unwrap();
            assert_eq!(message, TestStruct::default());

            // Serialize Structure and send it back to the client.
            let message = serde_json::to_string(&message).unwrap();
            write.send(Message::text(message)).await.unwrap();
        }
    }
}
