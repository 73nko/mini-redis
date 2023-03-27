use std::time::{Duration, Instant};

use crate::mini_redis::request::{parse_request, Command, Request};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpStream;

use super::request::CommandRequest;
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

    while let Ok(request) = parse_request(&buffer) {
        for command_request in request.commands {
            println!("Parsed request: {:?}", command_request);

            match command_request.command {
                Command::Get => get_command_from_request(store, &mut buf, command_request).await,
                Command::Set => set_command_from_request(store, &mut buf, command_request).await,
                Command::Keys => keys_command_from_request(store, &mut buf, command_request).await,
                Command::Delete => del_command_from_request(store, &mut buf, command_request).await,
                Command::Exists => {
                    exists_command_from_request(store, &mut buf, command_request).await
                }
            }
        }
    }
}

async fn get_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: CommandRequest,
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
        Some((value, expiration)) => {
            if let Some(expiration) = expiration {
                // If the key has expired, delete it and return an error
                if expiration <= Instant::now() {
                    del_command_from_request(store, buf, request).await;
                    "Key not found".to_string()
                } else {
                    format!("Value: {}", value)
                }
            } else {
                format!("Value: {}", value)
            }
        }
        None => "Key not found".to_string(),
    };
    buf.write_all(response.as_bytes()).await.unwrap();
    buf.flush().await.unwrap();
}

async fn set_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: CommandRequest,
) {
    if let Some(value) = request.value {
        // If the key has an expiration, set it to the current time + the TTL
        let expiration = request
            .expiration
            .map(|ttl| Instant::now() + Duration::from_secs(ttl));

        let mut store_lock = store.lock().await;
        if let Some(key) = request.key {
            store_lock.insert(key, (value, expiration));
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

async fn del_command_from_request(
    store: &KeyValueStore,
    buf: &mut BufStream<&mut TcpStream>,
    request: CommandRequest,
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
    request: CommandRequest,
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
    request: CommandRequest,
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
