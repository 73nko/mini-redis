use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: mini_redis_client <request>");
        return Ok(());
    }

    let request = &args[1];

    let mut stream = TcpStream::connect("127.0.0.1:6379").await?;
    let (reader, writer) = stream.split();

    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    // Send the request
    writer
        .write_all(format!("{}\n", request).as_bytes())
        .await?;
    writer.flush().await?;

    // Read the response
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    println!("Response: {}", response);

    Ok(())
}
