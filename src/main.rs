use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;
use serde_json::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufStream, BufWriter},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

type KeyValueStore = Arc<Mutex<HashMap<String, String>>>;

#[derive(Debug, Deserialize, PartialEq)]
enum Command {
    #[serde(alias = "GET")]
    Get,
    #[serde(alias = "SET")]
    Set,
    #[serde(alias = "DEL")]
    Delete,
    #[serde(alias = "EXISTS")]
    Exists,
    #[serde(alias = "KEYS")]
    Keys,
}

#[derive(Debug, Deserialize)]
struct Request {
    command: Command,
    key: Option<String>,
    value: Option<String>,
    pattern: Option<String>,
}

fn parse_request(request_data: &str) -> Result<Request, Error> {
    serde_json::from_str(request_data)
}

async fn handle_connection(mut socket: TcpStream, store: KeyValueStore) {
    let mut buffer = String::new();
    let mut buf = BufStream::new(&mut socket);

    buf.read_line(&mut buffer).await.unwrap();

    println!("Request: {}", buffer);

    match parse_request(&buffer) {
        Ok(request) => {
            println!("Parsed request: {:?}", request);

            match request.command {
                Command::Get => {
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
                Command::Set => {
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
                Command::Delete => {
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

                Command::Exists => {
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

                Command::Keys => {
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
                _ => {
                    let response = "Unknown command";
                    buf.write_all(response.as_bytes()).await.unwrap();
                    buf.flush().await.unwrap();
                }
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

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let store = Arc::new(Mutex::new(HashMap::new()));

    println!("Listening on: {}", listener.local_addr().unwrap());
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let store = Arc::clone(&store);
        tokio::spawn(async move {
            handle_connection(socket, store).await;
        });
    }
}
