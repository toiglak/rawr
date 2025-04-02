use std::{
    future::Future,
    sync::Arc,
    sync::atomic::{AtomicU32, Ordering},
};

use futures::{StreamExt, channel::oneshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dashmap::DashMap;

pub mod channel;

pub use channel::*;

pub type Result<T> = std::result::Result<T, TransportError>;
pub type ClientTransport<Req, Res> = (Tx<Request<Req>>, Rx<Response<Res>>);
pub type ServerTransport<Req, Res> = (Rx<Request<Req>>, Tx<Response<Res>>);

#[derive(Debug, Clone, Error)]
pub enum TransportError {
    #[error("Failed to send data")]
    SendError,
    #[error("Failed to receive data")]
    ReceiveError,
    #[error("Connection closed")]
    Closed,
}

#[derive(Serialize, Deserialize)]
pub struct Request<P> {
    /// Unique identifier used to send the response back to the correct caller
    /// when multiple calls to the same method were made.
    pub id: u32,
    /// Request payload. It consists of the arguments of the rpc call.
    pub data: P,
}

#[derive(Serialize, Deserialize)]
pub struct Response<P> {
    /// Unique identifier used to send the response back to the correct caller
    /// when multiple calls to the same method were made.
    pub id: u32,
    /// Request payload. It consists of the return value of the rpc call.
    pub data: P,
}

pub struct AbstractClient<Req, Res> {
    counter: Arc<AtomicU32>,
    requests: Arc<DashMap<u32, oneshot::Sender<Response<Res>>>>,
    server_tx: Tx<Request<Req>>,
}

impl<Req, Res> AbstractClient<Req, Res> {
    pub fn new(transport: ClientTransport<Req, Res>) -> (Self, impl Future<Output = ()>) {
        let (server_tx, server_rx) = transport;

        let requests = Arc::new(DashMap::new());

        let client = Self {
            counter: Arc::new(AtomicU32::new(0)),
            requests: requests.clone(),
            server_tx,
        };

        (client, dispatch_server_responses(server_rx, requests))
    }

    pub async fn make_request(&self, data: Req) -> Res {
        //// Make a request.
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.requests.insert(id, tx);
        self.server_tx.send(Request { id, data });

        //// Wait for the response.
        let res = rx.await.expect("Failed to receive response");
        res.data
    }
}

impl<Req, Res> Clone for AbstractClient<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter.clone(),
            server_tx: self.server_tx.clone(),
            requests: self.requests.clone(),
        }
    }
}

async fn dispatch_server_responses<ResData>(
    mut server_rx: Rx<Response<ResData>>,
    requests: Arc<DashMap<u32, oneshot::Sender<Response<ResData>>>>,
) {
    while let Some(res) = server_rx.recv().await {
        if let Some((_, sender)) = requests.remove(&res.id) {
            sender.send(res).ok();
        } else {
            // log::trace!("Received response with unknown id: {}", res.id);
        }
    }
}

pub struct AbstractServer;

impl AbstractServer {
    pub async fn new<Req, Res>(
        server_transport: ServerTransport<Req, Res>,
        handle_request: impl AsyncFn(Req) -> Res,
    ) {
        let (client_rx, client_tx) = server_transport;
        let handle_request = async |req: Request<Req>| {
            let data = handle_request(req.data).await;
            let res = Response { id: req.id, data };
            client_tx.send(res);
        };
        // TODO: Consider returning a stream, so that user can handle requests in
        // parallel if they want to.
        client_rx.0.for_each_concurrent(None, handle_request).await;
    }
}
