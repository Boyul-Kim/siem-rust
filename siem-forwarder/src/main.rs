// src/main.rs

use tokio::net::TcpStream;
use std::error::Error;

mod platform;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;

    // Windows
    #[cfg(target_os = "windows")]
    {
        platform::windows::log_windows_os(stream).await?;
    }

    // Linux
    #[cfg(target_os = "linux")]
    {
        platform::linux::log_linux_os(stream).await?;
    }

    // macOS
    #[cfg(target_os = "macos")]
    {
        platform::macos::log_macos_os(stream).await?;
    }

    // Unsupported OS
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Unsupported OS",
        )));
    }

    Ok(())
}
