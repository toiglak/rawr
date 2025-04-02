use rawr::{AbstractClient, AbstractServer, ClientTransport, Result, ServerTransport};
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
    /// Create a new client.
    pub fn new(
        transport: ClientTransport<TestRequest, TestResponse>,
    ) -> (Self, impl Future<Output = ()>) {
        let (inner, task) = AbstractClient::new(transport);
        (Self { inner }, task)
    }
}

impl TestClient {
    pub async fn say_hello(&self, arg: String) -> Result<String> {
        let req = TestRequest::say_hello((arg,));
        match self.inner.make_request(req).await {
            Ok(TestResponse::say_hello(ret)) => Ok(ret),
            Ok(_) => panic!("Unexpected response"),
            Err(e) => Err(e),
        }
    }

    pub async fn complex(&self, arg0: Structure, arg1: i32) -> Result<Structure> {
        let req = TestRequest::complex((arg0, arg1));
        match self.inner.make_request(req).await {
            Ok(TestResponse::complex(ret)) => Ok(ret),
            Ok(_) => panic!("Unexpected response"),
            Err(e) => Err(e),
        }
    }

    pub async fn ping_enum(&self, arg: EnumAdjacentlyTagged) -> Result<EnumAdjacentlyTagged> {
        let req = TestRequest::ping_enum((arg,));
        match self.inner.make_request(req).await {
            Ok(TestResponse::ping_enum(ret)) => Ok(ret),
            Ok(_) => panic!("Unexpected response"),
            Err(e) => Err(e),
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
                let res = service_handler.say_hello(arg0).await;
                Ok(TestResponse::say_hello(res))
            }
            TestRequest::complex((arg0, arg1)) => {
                let res = service_handler.complex(arg0, arg1).await;
                Ok(TestResponse::complex(res))
            }
            TestRequest::ping_enum((arg0,)) => {
                let res = service_handler.ping_enum(arg0).await;
                Ok(TestResponse::ping_enum(res))
            }
        };

        AbstractServer::new(server_transport, handle_request)
    }
}
