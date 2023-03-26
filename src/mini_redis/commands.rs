use std::time::Duration;

use crate::mini_redis::request::{parse_request, Command, Request};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

use super::KeyValueStore;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn handle_connection(mut socket: TcpStream, store: KeyValueStore) {
    let timeout_result = tokio::time::timeout(CONNECTION_TIMEOUT, async {
        handle_request(&mut socket, &store).await
    })
    .await;

    match timeout_result {
        Ok(()) => {}
        Err(err) => eprintln!("Error handling connection: {}", err),
    }

    let _ = &socket.shutdown().await;
}

async fn handle_request(socket: &mut TcpStream, store: &KeyValueStore) {
    let mut buffer = String::new();
    let mut buf = BufStream::new(socket);
    buf.read_line(&mut buffer).await.unwrap();

    match parse_request(&buffer) {
        Ok(request) => {
            println!("Parsed request: {:?}", request);

            match request.command {
                Command::Get => get_command_from_request(store, &mut buf, request).await,
                Command::Set => set_command_from_request(store, &mut buf, request).await,
                Command::Delete => delete_command_from_request(store, &mut buf, request).await,
                Command::Exists => exists_command_from_request(store, &mut buf, request).await,
                Command::Keys => keys_command_from_request(store, &mut buf, request).await,
            }
        }
        Err(e) => {
            eprintln!("Error parsing request: {}", e);
            let response = format!("Error parsing request: {}", e);
            buf.write_all(response.as_bytes()).await.unwrap();
            buf.flush().await.unwrap();
        }
    }
}

async fn get_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: Request,
) {
    let response = {
        let store_lock = store.lock().await;
        if let Some(key) = &request.key {
            store_lock.get(key).cloned()
        } else {
            None
        }
    };

    let response = match response {
        Some(value) => format!("Value: {}", value),
        None => "Key not found".to_string(),
    };
    buf.write_all(response.as_bytes()).await.unwrap();
    buf.flush().await.unwrap();
}

async fn set_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: Request,
) {
    if let Some(value) = request.value {
        let mut store_lock = store.lock().await;
        if let Some(key) = request.key {
            store_lock.insert(key, value);
        }
        let response = "OK";
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    } else {
        let response = "Error: Missing value\n";
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    }
}

async fn delete_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: Request,
) {
    if let Some(key) = request.key {
        let mut store_lock = store.lock().await;
        let removed = store_lock.remove(&key).is_some();

        let response = if removed { "1\n" } else { "0\n" };
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    } else {
        let response = "Error: Missing key\n";
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    }
}

async fn exists_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: Request,
) {
    if let Some(key) = request.key {
        let store_lock = store.lock().await;
        let exists = store_lock.contains_key(&key);

        let response = if exists { "1\n" } else { "0\n" };
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    } else {
        let response = "Error: Missing key\n";
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    }
}

async fn keys_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: Request,
) {
    if let Some(pattern) = request.pattern {
        let store_lock = store.lock().await;
        let pattern_regex = regex::Regex::new(&pattern).unwrap();
        let keys: Vec<String> = store_lock
            .keys()
            .filter(|key| pattern_regex.is_match(key))
            .cloned()
            .collect();

        let response = format!("Keys: {:?}\n", keys);
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    } else {
        let response = "Error: Missing pattern\n";
        buf.write_all(response.as_bytes()).await.unwrap();
        buf.flush().await.unwrap();
    }
}
