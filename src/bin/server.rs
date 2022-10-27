use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type ShardedData = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

fn with_shards(capacity: usize) -> ShardedData {
    let mut data = Vec::with_capacity(capacity);
    for _ in 0..capacity {
        data.push(Mutex::new(HashMap::new()));
    }
    Arc::new(data)
}

#[tokio::main]
async fn main() {
    // Bind listener to mini-redis
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let data = with_shards(10);

    loop {
        // IP and port of connection are ignored
        let (socket, _) = listener.accept().await.unwrap();

        // Clone handle
        let data = data.clone();

        // Spawn task for each socket and move socket into task
        tokio::spawn(async move {
            process(socket, data).await;
        });
    }
}

async fn process(socket: TcpStream, data: ShardedData) {
    // Use frames instead of byte streams
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        // Respond with default response
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(cmd) => {
                let mut shard = data[hash(cmd.key()) % data.len()].lock().unwrap();
                shard.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let shard = data[hash(cmd.key()) % data.len()].lock().unwrap();
                if let Some(value) = shard.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            command => panic!("Unimplemented command: {:?}", command),
        };

        // Write response to client
        connection.write_frame(&response).await.unwrap();
    }
}

fn hash<T: Hash + ?Sized>(t: &T) -> usize {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish() as usize
}
