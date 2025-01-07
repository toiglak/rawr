#![allow(non_camel_case_types)]

use ::std::string::String;
use std::future::Future;

use ::rawr::{Request, Response, Rx, Tx};
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
    req_tx: Tx<Request<TestRequest>>,
    res_rx: Rx<Response<TestResponse>>,
}

impl TestClient {
    pub fn new(
        transport: (Tx<Request<TestRequest>>, Rx<Response<TestResponse>>),
    ) -> (Self, impl Future<Output = ()>) {
        let (req_rx, res_tx) = transport;

        let run_client = async move {
            // while let Some(req) = req_rx.recv().await {
            //     match req.data {
            //         TestRequest::say_hello(args) => {
            //             let ret = self.hello(args.0).await;
            //             let resp = Response {
            //                 id: req.id,
            //                 data: TestResponse::say_hello(ret),
            //             };
            //             res_tx.send(resp);
            //         }
            //     };
            // }
        };

        (
            Self {
                req_tx: req_rx,
                res_rx: res_tx,
            },
            run_client,
        )
    }

    pub async fn say_hello(&mut self, arg: String) -> String {
        let req = Request {
            id: 0,
            data: TestRequest::say_hello((arg,)),
        };
        self.req_tx.send(req);
        let resp = self.res_rx.recv().await.unwrap();
        let resp = resp.data;
        match resp {
            TestResponse::say_hello(data) => data,
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
    /// // Run handlers.
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

        // let (req_rx, res_tx) = server_transport;

        // while let Some(request) = req_rx.recv().await {
        //     let handler = handler.clone();
        //     let mut res_tx = res_tx.clone();

        //     match request.data {
        //         TestRequest::say_hello(args) => {
        //             let ret = handler.hello(args.0).await;
        //             let resp = Response {
        //                 id: request.id,
        //                 data: TestResponse::say_hello(ret),
        //             };
        //             res_tx.send(resp);
        //         }
        //     };
        // }
    }
}
