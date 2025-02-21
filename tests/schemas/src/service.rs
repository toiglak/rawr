use rawr::{AbstractClient, AbstractServer, ClientTransport, ServerTransport};
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::{enumeration::EnumAdjacentlyTagged, structure::Structure};

#[allow(async_fn_in_trait)]
pub trait TestService: Clone + 'static + Send + Sync {
    async fn say_hello(&self, arg: String) -> String;
    /// Service should increment `count` by `n`.
    async fn complex(&self, input: Structure, n: i32) -> Structure;
    async fn ping_enum(&self, arg: EnumAdjacentlyTagged) -> EnumAdjacentlyTagged;
}

///////////// GENERATED CODE /////////////

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum TestRequest {
    complex((Structure, i32)),
    say_hello((String,)),
    ping_enum((EnumAdjacentlyTagged,)),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum TestResponse {
    complex(Structure),
    say_hello(String),
    ping_enum(EnumAdjacentlyTagged),
}

#[derive(Clone)]
pub struct TestClient {
    inner: AbstractClient<TestRequest, TestResponse>,
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
        transport: ClientTransport<TestRequest, TestResponse>,
    ) -> (Self, impl Future<Output = ()>) {
        let (inner, client_task) = AbstractClient::new(transport);
        (Self { inner }, client_task)
    }
}

impl TestService for TestClient {
    async fn say_hello(&self, arg: String) -> String {
        let req = TestRequest::say_hello((arg,));
        let res = self.inner.make_request(req).await;

        #[allow(irrefutable_let_patterns)]
        if let TestResponse::say_hello(ret) = res {
            ret
        } else {
            // Perhaps this should return an error instead of panicking?
            panic!("Unexpected response")
        }
    }

    async fn complex(&self, arg0: Structure, arg1: i32) -> Structure {
        let req = TestRequest::complex((arg0, arg1));
        let res = self.inner.make_request(req).await;

        #[allow(irrefutable_let_patterns)]
        if let TestResponse::complex(ret) = res {
            ret
        } else {
            // Perhaps this should return an error instead of panicking?
            panic!("Unexpected response")
        }
    }

    async fn ping_enum(&self, arg: EnumAdjacentlyTagged) -> EnumAdjacentlyTagged {
        let req = TestRequest::ping_enum((arg,));
        let res = self.inner.make_request(req).await;

        #[allow(irrefutable_let_patterns)]
        if let TestResponse::ping_enum(ret) = res {
            ret
        } else {
            // Perhaps this should return an error instead of panicking?
            panic!("Unexpected response")
        }
    }
}

pub struct TestServer;

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
    pub fn new(
        server_transport: ServerTransport<TestRequest, TestResponse>,
        service_handler: impl TestService,
    ) -> impl Future<Output = ()> {
        let handle_request = async move |req: TestRequest| match req {
            TestRequest::say_hello((arg0,)) => {
                let data = service_handler.say_hello(arg0).await;
                TestResponse::say_hello(data)
            }
            TestRequest::complex((arg0, arg1)) => {
                let data = service_handler.complex(arg0, arg1).await;
                TestResponse::complex(data)
            }
            TestRequest::ping_enum((arg0,)) => {
                let data = service_handler.ping_enum(arg0).await;
                TestResponse::ping_enum(data)
            }
        };

        AbstractServer::new(server_transport, handle_request)
    }
}
