use std::ops::{Deref, DerefMut};

use futures::{
    StreamExt,
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

pub fn transport<Req, Res>() -> ((Tx<Req>, Rx<Res>), (Rx<Req>, Tx<Res>)) {
    let (req_tx, req_rx) = mpsc::unbounded();
    let (res_tx, res_rx) = mpsc::unbounded();
    ((Tx(req_tx), Rx(res_rx)), (Rx(req_rx), Tx(res_tx)))
}

pub struct Tx<T>(pub UnboundedSender<T>);

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

impl<T> Deref for Tx<T> {
    type Target = UnboundedSender<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Tx<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Rx<T>(pub UnboundedReceiver<T>);

impl<T> Rx<T> {
    pub fn try_recv(&mut self) -> Option<T> {
        self.0.try_next().ok().flatten()
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.0.next().await
    }
}

impl<T> Deref for Rx<T> {
    type Target = UnboundedReceiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Rx<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
