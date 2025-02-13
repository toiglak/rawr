use futures::{future, stream, SinkExt, StreamExt};
use schemas::{
    enumeration::EnumAdjacentlyTagged,
    module::ImportedStruct,
    service::{TestClient, TestResponse, TestService},
    structure::Structure,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{error::ProtocolError, protocol::Message, Error},
};

#[tokio::main]
async fn main() {
    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");

    let (client_transport, server_transport) = rawr::transport();

    // Create server and client.
    let (client, client_task) = TestClient::new(client_transport);

    // Run client task.
    tokio::spawn(client_task);

    // Handle communication with the server.
    tokio::spawn(async move {
        let ws = connect_async(&format!("ws://{}", addr)).await.unwrap().0;

        let (mut socket_tx, mut socket_rx) = ws.split();
        let (mut server_rx, server_tx) = server_transport;

        let handle_incoming = async {
            while let Some(msg) = socket_rx.next().await {
                let msg = match msg {
                    Ok(Message::Close(_))
                    | Err(Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)) => break,
                    Ok(msg) => msg,
                    Err(e) => panic!("{:?}", e),
                };
                let msg: rawr::Response<TestResponse> =
                    serde_json::from_str(&msg.to_string()).unwrap();
                server_tx.send(msg);
            }
        };

        let handle_outgoing = async {
            while let Some(req) = server_rx.recv().await {
                let req = Message::text(serde_json::to_string(&req).unwrap());
                socket_tx.send(req).await.unwrap();
            }
        };

        future::join(handle_incoming, handle_outgoing).await;
    });

    // Make 10 concurrent requests to the server.
    let client = &client;
    let make_request = |i| async move {
        let response = client.say_hello(format!("World {}", i + 1)).await;
        assert_eq!(response, format!("Hello, World {}!", i + 1));
    };
    stream::iter(0..10)
        .for_each_concurrent(None, make_request)
        .await;

    // Test complex method.
    let res = client.complex(Structure::default(), 42).await;
    assert_eq!(res.count, 42);

    //// Test sending enum back and forth.

    let en = EnumAdjacentlyTagged::VariantA;
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantB();
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantC(42);
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantD(());
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantE(ImportedStruct {
        value: "string".to_string(),
    });
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantF((
        42,
        ImportedStruct {
            value: "string".to_string(),
        },
    ));
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantG(
        42,
        ImportedStruct {
            value: "string".to_string(),
        },
    );
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantH {};
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);

    let en = EnumAdjacentlyTagged::VariantI {
        a: 42,
        b: ImportedStruct {
            value: "string".to_string(),
        },
    };
    let res = client.ping_enum(en.clone()).await;
    assert_eq!(res, en);
}
