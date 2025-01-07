use futures::stream::{self, StreamExt};
use schemas::service::{TestClient, TestServer, TestService};

#[derive(Clone)]
struct ServiceImpl {}

impl TestService for ServiceImpl {
    async fn say_hello(&self, arg: String) -> String {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        format!("Hello, {}!", arg)
    }
}

#[tokio::main]
async fn main() {
    let (client_transport, server_transport) = rawr::transport();

    // Create server and client.
    let server_task = TestServer::new(server_transport, ServiceImpl {});
    let (client, client_task) = TestClient::new(client_transport);

    // Run tasks.
    tokio::spawn(client_task);
    tokio::spawn(server_task);

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
