use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, BufReader};
use std::error::Error;
use bincode;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct WrapperEventlogRecord {
    length: u32,
    reserved: u32,
    record_number: u32,
    time_generated: u32,
    time_written: u32,
    event_id: u32,
    num_strings: u16,
    event_category: u16,
    reserved_flags: u16,
    closing_record_number: u32,
    string_offset: u32,
    user_sid_length: u32,
    user_sid_offset: u32,
    data_length: u32,
    data_offset: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            match handle_client(&mut socket).await {
                Ok(_) => println!("Client handled successfully"),
                Err(e) => eprintln!("Error handling client: {:?}", e),
            }
        });
    }
}

async fn handle_client(socket: &mut tokio::net::TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(socket);

    loop {
        // Read the length of the serialized data (4 bytes)
        let mut length_buf = [0u8; 4];
        if reader.read_exact(&mut length_buf).await.is_err() {
            break; // End of stream or read error
        }

        // Convert length from bytes to usize
        let data_len = u32::from_be_bytes(length_buf) as usize;
        let mut data_buf = vec![0u8; data_len];
        
        // Read the actual serialized data
        if reader.read_exact(&mut data_buf).await.is_err() {
            break; // End of stream or read error
        }

        // Deserialize the data into WrapperEventlogRecord
        match bincode::deserialize::<WrapperEventlogRecord>(&data_buf) {
            Ok(event_record) => {
                println!("Received event log record: {:?}", event_record);
            },
            Err(e) => eprintln!("Failed to deserialize event record: {:?}", e),
        }
    }

    Ok(())
}
