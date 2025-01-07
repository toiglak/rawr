use futures::stream::{self, StreamExt};
use schemas::service::{TestClient, TestServer, TestService};
use tokio::time::{self, Duration};

#[derive(Clone)]
struct ServiceImpl {}

impl TestService for ServiceImpl {
    async fn say_hello(&self, arg: String) -> String {
        // To see the test fail, uncomment this line.
        // std::thread::sleep(std::time::Duration::from_secs(1));
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
    let make_requests = stream::iter(0..10).for_each_concurrent(None, make_request);

    // For posterity: https://www.youtube.com/watch?v=ms8zKpS_dZE
    // ASSERT: Requests should be processed concurrently (take ~1s).
    if let Err(_) = time::timeout(Duration::from_secs(2), make_requests).await {
        panic!("for_each_concurrent took more than 2 seconds");
    }
}
