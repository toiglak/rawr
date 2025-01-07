#![allow(non_camel_case_types)]

use ::std::string::String;
use std::{
    future::Future,
    sync::atomic::{AtomicU32, Ordering},
    sync::Arc,
};

use ::rawr::{Request, Response, Rx, Tx};
use rawr::dashmap::DashMap;
use rawr::futures::channel::oneshot;
use serde::{Deserialize, Serialize};

#[allow(async_fn_in_trait)]
pub trait TestService: 'static + Send + Sync {
    async fn say_hello(&self, arg: String) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum TestRequest {
    say_hello((String,)),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum TestResponse {
    say_hello(String),
}

// CRAZY IDEA: Now that we have `TestClient` not take any "special" functions, we
// can directly implement the `TestService` trait for `TestClient`! It may not seem
// like a big thing but it is! It means that "calls" on the client point directly
// to the trait in the IDE!

pub struct TestClient {
    counter: AtomicU32,
    req_tx: Tx<Request<TestRequest>>,
    pending: Arc<DashMap<u32, oneshot::Sender<Response<TestResponse>>>>,
}

impl TestClient {
    /// Create a new client. Returns a future that must be spawned on a runtime for
    /// the client to start processing requests and responses.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let (client_transport, server_transport) = rawr::transport();
    ///
    /// let server_task = TestServer::new(server_transport, ServiceImpl);
    /// let (mut client, client_task) = TestClient::new(client_transport);
    ///
    /// // Run tasks.
    /// tokio::spawn(client_task);
    /// tokio::spawn(server_task);
    ///
    /// let response = client.say_hello("World".to_string()).await;
    /// println!("{}", response);
    /// ```
    pub fn new(
        transport: (Tx<Request<TestRequest>>, Rx<Response<TestResponse>>),
    ) -> (Self, impl Future<Output = ()>) {
        let (req_tx, mut res_tx) = transport;
        let pending = Arc::new(DashMap::new());
        let client = Self {
            counter: AtomicU32::new(0),
            req_tx,
            pending: pending.clone(),
        };

        let run_client = {
            async move {
                while let Some(resp) = res_tx.recv().await {
                    if let Some((_, sender)) = pending.remove(&resp.id) {
                        sender.send(resp).ok();
                    } else {
                        // log::trace!("Received response with unknown id: {}", resp.id);
                    }
                }
            }
        };

        (client, run_client)
    }

    pub async fn say_hello(&self, arg: String) -> String {
        let (tx, rx) = oneshot::channel();
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        self.pending.insert(id, tx);
        let req = Request {
            id,
            data: TestRequest::say_hello((arg,)),
        };
        self.req_tx.send(req);
        let resp = rx.await.expect("Failed to receive response");
        match resp.data {
            TestResponse::say_hello(data) => data,
            #[allow(unreachable_patterns)]
            _ => panic!("Unexpected response"),
        }
    }
}

pub struct TestServer;

#[expect(unused)]
impl TestServer {
    /// Create a new server. Returns a future that must be spawned on a runtime for
    /// the server to start processing requests.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let (client_transport, server_transport) = rawr::transport();
    ///
    /// let server_task = TestServer::new(server_transport, ServiceImpl);
    /// let (mut client, client_task) = TestClient::new(client_transport);
    ///
    /// // Run tasks.
    /// tokio::spawn(client_task);
    /// tokio::spawn(server_task);
    ///
    /// let response = client.say_hello("World".to_string()).await;
    /// println!("{}", response);
    /// ```
    pub async fn new(
        server_transport: (Rx<Request<TestRequest>>, Tx<Response<TestResponse>>),
        handler: impl TestService,
    ) {
        let (mut req_rx, mut res_tx) = server_transport;
        while let Some(req) = req_rx.recv().await {
            let resp = match req.data {
                TestRequest::say_hello((arg,)) => {
                    let data = handler.say_hello(arg).await;
                    Response {
                        id: req.id,
                        data: TestResponse::say_hello(data),
                    }
                }
            };
            res_tx.send(resp);
        }
    }
}
