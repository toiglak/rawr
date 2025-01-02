use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");

    let (mut ws, _) = connect_async(&format!("ws://{}", addr)).await.unwrap();
    ws.send(Message::text("rust_client")).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap();
    assert_eq!(msg.to_text().unwrap(), "rust_client");
}
