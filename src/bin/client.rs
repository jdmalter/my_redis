use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        responder: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        responder: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx1, mut rx) = mpsc::channel(32);
    let tx2 = tx1.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(command) = rx.recv().await {
            match command {
                Command::Get { key, responder } => {
                    let response = client.get(&key).await;
                    let _ = responder.send(response);
                }
                Command::Set {
                    key,
                    value,
                    responder,
                } => {
                    let response = client.set(&key, value).await;
                    let _ = responder.send(response);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command = Command::Get {
            key: "hello".to_string(),
            responder: resp_tx,
        };
        tx1.send(command).await.unwrap();
        let response = resp_rx.await;
        let _ = dbg!(response);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            responder: resp_tx,
        };
        tx2.send(command).await.unwrap();
        let response = resp_rx.await;
        let _ = dbg!(response);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
