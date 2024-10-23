use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut line = String::new();

            while let Ok(bytes_read) = reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break; // Connection closed
                }
                println!("Received log: {}", line.trim());
                // Perform parsing and indexing here
                line.clear();
            }
        });
    }
}