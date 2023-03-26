use std::{collections::HashMap, sync::Arc};

use tokio::{net::TcpListener, sync::Mutex};

use crate::mini_redis::{handle_connection, KeyValueStore};

mod mini_redis;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let store: KeyValueStore = Arc::new(Mutex::new(HashMap::new()));

    println!("Listening on: {}", listener.local_addr().unwrap());
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let store = Arc::clone(&store);
        tokio::spawn(async move {
            handle_connection(socket, store).await;
        });
    }
}
