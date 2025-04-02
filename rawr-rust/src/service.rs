use std::{
    sync::Arc,
    sync::atomic::{AtomicU32, Ordering},
};

use futures::{StreamExt, channel::oneshot};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dashmap::DashMap;

pub mod channel;

pub use channel::*;

pub type Result<T> = std::result::Result<T, RequestError>;
pub type ClientTransport<Req, Res> = (Tx<Packet<Req>>, Rx<Packet<Result<Res>>>);
pub type ServerTransport<Req, Res> = (Rx<Packet<Req>>, Tx<Packet<Result<Res>>>);

#[derive(Debug, Clone, Error)]
pub enum TransportError {
    #[error("Failed to send data")]
    SendError,
    #[error("Failed to receive data")]
    ReceiveError,
    #[error("Connection closed")]
    Closed,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum RequestError {
    #[error("Transport closed")]
    TransportClosed,
    #[error("Request was cancelled")]
    Cancelled,
}

#[derive(Serialize, Deserialize)]
pub struct Packet<P> {
    /// Unique identifier used to send the response back to the correct caller,
    /// when multiple calls to the same method were made.
    pub id: u32,
    /// Request/response payload. It consists of the arguments of the rpc call when
    /// packet is a request and the return value of the rpc call when packet is a
    /// response.
    pub data: P,
}

pub struct AbstractClient<Req, Res> {
    counter: Arc<AtomicU32>,
    requests: Arc<DashMap<u32, oneshot::Sender<Packet<Result<Res>>>>>,
    server_tx: Tx<Packet<Req>>,
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

        (
            client,
            dispatch_server_responses::<Res, Res>(server_rx, requests),
        )
    }

    pub async fn make_request(&self, data: Req) -> Result<Res> {
        //// Make a request.
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.requests.insert(id, tx);
        self.server_tx.send(Packet { id, data });

        //// Wait for the response.
        match rx.await {
            Ok(packet) => packet.data,
            Err(_) => Err(RequestError::TransportClosed),
        }
    }

    pub fn cancel_all(&self) {
        // NOTE: We can't do into_iter() because DashMap is behind Arc.

        // Collect all requests keys
        let keys: Vec<u32> = self.requests.iter().map(|entry| *entry.key()).collect();

        // Cancel all requests
        for key in keys {
            if let Some((_, sender)) = self.requests.remove(&key) {
                let packet = Packet {
                    id: key,
                    data: Err(RequestError::Cancelled),
                };
                sender.send(packet).ok();
            }
        }
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

async fn dispatch_server_responses<Req, Res>(
    mut server_rx: Rx<Packet<Result<Res>>>,
    requests: Arc<DashMap<u32, oneshot::Sender<Packet<Result<Res>>>>>,
) {
    while let Some(res) = server_rx.recv().await {
        if let Some((_, sender)) = requests.remove(&res.id) {
            let packet = Packet {
                id: res.id,
                data: res.data,
            };
            sender.send(packet).ok();
        }
    }
}

pub struct AbstractServer;

impl AbstractServer {
    pub async fn new<Req, Res>(
        server_transport: ServerTransport<Req, Res>,
        handle_request: impl AsyncFn(Req) -> Result<Res>,
    ) {
        let (client_rx, client_tx) = server_transport;
        let handle_request = async |req: Packet<Req>| {
            let data = handle_request(req.data).await;
            let res = Packet { id: req.id, data };
            client_tx.send(res);
        };
        // TODO: Consider returning a stream, so that user can handle requests in
        // parallel if they want to.
        client_rx.0.for_each_concurrent(None, handle_request).await;
    }
}
