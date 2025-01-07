use schemas::service::{TestClient, TestServer, TestService};

struct ServiceImpl;

impl TestService for ServiceImpl {
    async fn say_hello(&self, arg: String) -> String {
        format!("Hello, {}!", arg)
    }
}

#[tokio::main]
async fn main() {
    let (client_transport, server_transport) = rawr::transport();

    let (mut client, client_task) = TestClient::new(client_transport);
    let server_task = TestServer::new(server_transport, ServiceImpl);

    // Run handlers.
    tokio::spawn(client_task);
    tokio::spawn(server_task);

    let response = client.say_hello("World".to_string()).await;
    println!("{}", response);
}
