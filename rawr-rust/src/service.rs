use futures::{
    channel::{mpsc, oneshot},
    lock::Mutex,
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};
use thiserror::Error;

pub mod channel;

pub use channel::*;

pub type Result<T> = std::result::Result<T, TransportError>;

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
    pub id: u64,
    /// Request payload. It consists of the arguments of the rpc call.
    pub data: P,
}

pub struct Response<P> {
    /// Unique identifier used to send the response back to the correct caller
    /// when multiple calls to the same method were made.
    pub id: u64,
    /// Request payload. It consists of the return value of the rpc call.
    pub data: P,
}

#[expect(unused)]
pub struct Client<ReqData, ResData> {
    counter: AtomicU64,
    request_tx: Mutex<mpsc::UnboundedSender<Request<ReqData>>>,
    response_map: Arc<Mutex<HashMap<u64, oneshot::Sender<Response<ResData>>>>>,
}

impl<ReqData, ResData> Client<ReqData, ResData> {
    #[expect(unused)]
    pub async fn new() -> (
        mpsc::UnboundedReceiver<Request<ReqData>>,
        mpsc::UnboundedSender<Response<ResData>>,
        Self,
    ) {
        let (request_tx, request_rx) = mpsc::unbounded::<Request<ReqData>>();
        let (response_tx, mut response_rx) = mpsc::unbounded::<Response<ResData>>();
        let response_map: Arc<Mutex<HashMap<u64, oneshot::Sender<Response<ResData>>>>>;
        response_map = Arc::new(Mutex::new(HashMap::new()));

        let response_map_ = response_map.clone();

        todo!();
        // tokio::spawn(async move {
        //     while let Some(response) = response_rx.next().await {
        //         if let Some(sender) = response_map_.lock().await.remove(&response.id) {
        //             sender.send(response).ok();
        //         }
        //     }
        // });

        (
            request_rx,
            response_tx,
            Self {
                counter: AtomicU64::new(0),
                request_tx: Mutex::new(request_tx),
                response_map,
            },
        )
    }

    #[expect(unused)]
    pub async fn make_request(&self, req: ReqData) -> ResData {
        let id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Create a oneshot channel to receive the response
        let (sender, receiver) = oneshot::channel();
        self.response_map.lock().await.insert(id, sender);

        // Send the request

        todo!();
        // let data = serde_json::to_string(&req).unwrap();
        // let request = Request { id, payload: data };
        // self.request_tx.lock().await.send(request).await.ok();

        todo!();
        // Receive the response
        // let response = receiver.await.unwrap();
        // serde_json::from_str(&response.data).unwrap()
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
