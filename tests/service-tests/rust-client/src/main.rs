use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

type TestStruct = schemas::structure::Structure;

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let (mut ws, _) = connect_async(&format!("ws://{}", addr)).await.unwrap();

    // Serialize Structure and send it to the server.
    let message = TestStruct::default();
    let message = serde_json::to_string(&message).unwrap();
    ws.send(Message::text(message)).await.unwrap();

    // Receive Structure from the server and deserialize it.
    let message = ws.next().await.unwrap().unwrap();
    let message = message.to_text().unwrap();
    let message = serde_json::from_str::<TestStruct>(message).unwrap();
    assert_eq!(message, TestStruct::default());
}
