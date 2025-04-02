//! TODO: This should probably be in /examples.

use futures::stream::{self, StreamExt};
use schemas::{
    enumeration::EnumAdjacentlyTagged,
    service::{TestClient, TestServer, TestService},
    structure::Structure,
};
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
    let (client_transport, server_transport) = rawr::transport();

    // Create server and client.
    let (client, client_task) = TestClient::new(client_transport);
    let server_task = TestServer::new(server_transport, ServiceImpl {});

    // Run tasks.
    tokio::spawn(client_task);
    tokio::spawn(server_task);

    // Make 10 concurrent requests to the server.
    let client = &client;
    let make_request = async move |i| {
        let response = client.say_hello(format!("World {}", i + 1)).await.unwrap();
        println!("{}: {}", i + 1, response);
    };
    let make_requests = stream::iter(0..10).for_each_concurrent(None, make_request);

    // For posterity: https://www.youtube.com/watch?v=ms8zKpS_dZE
    // ASSERT: Requests should be processed concurrently (take ~1s).
    if let Err(_) = time::timeout(Duration::from_secs(2), make_requests).await {
        panic!("test took more than 1 second to complete");
    }

    client.complex(Structure::default(), 42).await.unwrap();
}
