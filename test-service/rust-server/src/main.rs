use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream).await.expect("Failed to accept");
        let (mut write, mut read) = ws_stream.split();

        while let Some(Ok(msg)) = read.next().await {
            if msg.is_close() {
                break;
            }

            let msg = msg.to_text().unwrap();
            write.send(Message::text(msg)).await.unwrap();
        }
    }
}
