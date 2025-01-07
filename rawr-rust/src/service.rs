use std::{
    future::Future,
    sync::atomic::{AtomicU32, Ordering},
    sync::Arc,
};

use futures::channel::oneshot;
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

pub struct Request<P> {
    /// Unique identifier used to send the response back to the correct caller
    /// when multiple calls to the same method were made.
    pub id: u32,
    /// Request payload. It consists of the arguments of the rpc call.
    pub data: P,
}

pub struct Response<P> {
    /// Unique identifier used to send the response back to the correct caller
    /// when multiple calls to the same method were made.
    pub id: u32,
    /// Request payload. It consists of the return value of the rpc call.
    pub data: P,
}

pub struct AbstractClient<Req, Res> {
    counter: Arc<AtomicU32>,
    req_tx: Tx<Request<Req>>,
    res_map: Arc<DashMap<u32, oneshot::Sender<Response<Res>>>>,
}

impl<Req, Res> AbstractClient<Req, Res> {
    pub fn new(transport: ClientTransport<Req, Res>) -> (Self, impl Future<Output = ()>) {
        let (req_tx, res_rx) = transport;

        let res_map = Arc::new(DashMap::new());

        let client = Self {
            counter: Arc::new(AtomicU32::new(0)),
            req_tx,
            res_map: res_map.clone(),
        };

        (client, handle_responses(res_rx, res_map))
    }

    pub async fn make_request(&self, data: Req) -> Res {
        //// Make a request.
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.res_map.insert(id, tx);
        self.req_tx.send(Request { id, data });

        //// Wait for the response.
        let resp = rx.await.expect("Failed to receive response");
        resp.data
    }
}

impl<Req, Res> Clone for AbstractClient<Req, Res> {
    fn clone(&self) -> Self {
        Self {
            counter: self.counter.clone(),
            req_tx: self.req_tx.clone(),
            res_map: self.res_map.clone(),
        }
    }
}

async fn handle_responses<ResData>(
    mut res_rx: Rx<Response<ResData>>,
    req_map: Arc<DashMap<u32, oneshot::Sender<Response<ResData>>>>,
) {
    while let Some(resp) = res_rx.recv().await {
        if let Some((_, sender)) = req_map.remove(&resp.id) {
            sender.send(resp).ok();
        } else {
            // log::trace!("Received response with unknown id: {}", resp.id);
        }
    }
}

// /// Used to send request to a service to handle it.
// ///
// /// `ClientTx` is used to send the response back to the appropriate client.
// pub type MultiplexTx = mpsc::UnboundedSender<(ClientTx, Request)>;
// /// Used to send service response back to the original client.
// pub type ClientTx = mpsc::UnboundedSender<Response>;

// /// Maps requests coming from multiple clients into a single stream of requests for the
// /// service handler.
// ///
// /// This mapping is necessary because request ID is local to each client and without it
// /// id-s coming from different clients would collide.
// pub fn multiplex_requests(
//     mut service_tx: mpsc::UnboundedSender<Request>,
//     mut service_rx: mpsc::UnboundedReceiver<Response>,
// ) -> MultiplexTx {
//     type ClientReqId = u64;
//     type ServerReqId = u64;

//     let (request_tx, mut request_rx) = mpsc::unbounded::<(ClientTx, Request)>();

//     let response_map: Arc<Mutex<HashMap<ServerReqId, (ClientReqId, ClientTx)>>>;
//     response_map = Arc::new(Mutex::new(HashMap::new()));

//     let map = response_map.clone();
//     let client_to_service = async move {
//         let mut server_id = 0;
//         while let Some((response_tx, mut request)) = request_rx.next().await {
//             server_id += 1;
//             map.lock()
//                 .await
//                 .insert(server_id, (request.id, response_tx));
//             request.id = server_id;
//             service_tx.send(request).await.ok();
//         }
//     };

//     let map = response_map.clone();
//     let service_to_client = async move {
//         while let Some(mut response) = service_rx.next().await {
//             if let Some((req_id, mut sender)) = map.lock().await.remove(&response.id) {
//                 response.id = req_id;
//                 sender.send(response).await.ok();
//             }
//         }
//     };

//     tokio::spawn(client_to_service);
//     tokio::spawn(service_to_client);

//     request_tx
// }
