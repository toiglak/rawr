use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    StreamExt,
};

// TODO: Consider accepting any channel implementation implementing a trait.

pub fn transport<Req, Res>() -> ((Tx<Req>, Rx<Res>), (Rx<Req>, Tx<Res>)) {
    let (req_tx, req_rx) = mpsc::unbounded();
    let (res_tx, res_rx) = mpsc::unbounded();
    ((Tx(req_tx), Rx(res_rx)), (Rx(req_rx), Tx(res_tx)))
}

pub struct Tx<T>(UnboundedSender<T>);

impl<T> Tx<T> {
    pub fn send(&self, message: T) {
        self.0.unbounded_send(message).unwrap();
    }
}

impl<T> Clone for Tx<T> {
    fn clone(&self) -> Self {
        Tx(self.0.clone())
    }
}

pub struct Rx<T>(UnboundedReceiver<T>);

impl<T> Rx<T> {
    pub fn try_recv(&mut self) -> Option<T> {
        self.0.try_next().ok().flatten()
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.0.next().await
    }
}
