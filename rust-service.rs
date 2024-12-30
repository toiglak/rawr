#[allow(async_fn_in_trait)]
pub trait Service: 'static + Send + Sync {
    async fn call(&self, arg: ::std::string::String) -> String;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum ServiceRequest {
    call((String,)),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "payload")]
pub enum ServiceResponse { 
    call(String),
}

pub struct ServiceClient {
  req_tx: ::rawr::ChannelTx<::rawr::Request<ServiceRequest>>,
  res_rx: ::rawr::ChannelRx<::rawr::Response<ServiceResponse>>,
}

impl ServiceClient {
    pub async fn new() -> (
        ::rawr::ChannelRx<::rawr::Request<ServiceRequest>>,
        ::rawr::ChannelTx<::rawr::Response<ServiceResponse>>,
        Self,
    ) {
        let (req_tx, req_rx) = ::rawr::channel::<::rawr::Request<ServiceRequest>>();
        let (res_tx, res_rx) = ::rawr::channel::<::rawr::Response<ServiceResponse>>();
        (req_rx, res_tx, Self {
            req_tx,
            res_rx,
        })
    }

    pub async fn call(&self, arg: String) -> String {
        let req = ::rawr::Request {
            id: 0,
            data: ServiceRequest::call((arg,)),
        };
        self.req_tx.send(req).await.unwrap();
        let resp = self.res_rx.next().await.unwrap();
        match resp {
            ServiceResponse::call(data) => data,
        }
    }
}

pub struct ServiceServer<S: Service> {
    handler: ::std::sync::Arc<S>,
    req_rx: ::rawr::ChannelRx<::rawr::Request<ServiceRequest>>,
    res_tx: ::rawr::ChannelTx<::rawr::Response<ServiceResponse>>,
}

impl<S: Service> ServiceServer<S> {
    pub fn new(
        handler: S,
    ) -> (
        ::rawr::ChannelTx<::rawr::Request<ServiceRequest>>,
        ::rawr::ChannelRx<::rawr::Response<ServiceResponse>>,
        Self,
    ) {
        let (req_tx, req_rx) = ::rawr::channel::<::rawr::Request<ServiceRequest>>();
        let (res_tx, res_rx) = ::rawr::channel::<::rawr::Response<ServiceResponse>>();
        (req_tx, res_rx, Self {
            handler: ::std::sync::Arc::new(handler),
            req_rx,
            res_tx,
        })
    }

    pub async fn run(mut self) {
        while let Some(request) = self.req_rx.next().await {
            let handler = self.handler.clone();
            let mut res_tx = self.res_tx.clone();

            match request.data {
                ServiceRequest::call(args) => {
                    let ret = handler.call(args.0).await;
                    let resp = ::rawr::Response {
                        id: request.id,
                        data: ServiceResponse::call(ret),
                    };
                    res_tx.send(resp).await.unwrap();
                }
            };
        }
    }
}