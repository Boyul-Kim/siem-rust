use std::env;
use std::time::UNIX_EPOCH;
use std::time::Duration;
use windows::Win32::System::EventLog::{
    OpenEventLogW, ReadEventLogW, CloseEventLog, EVENTLOG_SEQUENTIAL_READ, READ_EVENT_LOG_READ_FLAGS, EVENTLOGRECORD
};
use windows::Win32::System::SystemServices::EVENTLOG_FORWARDS_READ;
use windows::Win32::Foundation::{HANDLE, GetLastError, ERROR_INSUFFICIENT_BUFFER};
use windows::core::{Error as WinError, PCWSTR};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::error::Error;
use std::mem::size_of;
use chrono;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Establish a TCP connection
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    // Get the OS type
    let os_type: &str = env::consts::OS;

    // Define the logging function based on OS
    match os_type {
        "windows" => {
            log_windows_os(stream).await?;
        }
        "linux" => {
            log_linux_os(stream).await?;
        }
        "macos" => {
            log_mac_os(stream).await?;
        }
        _ => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported OS",
            )) as Box<dyn Error>);
        }
    }

    Ok(())
}

fn open_windows_event_log() -> Result<HANDLE, Box<dyn Error>> {
    let log_name: Vec<u16> = OsStr::new("System")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<u16>>();
    let log_name_ptr: PCWSTR = PCWSTR(log_name.as_ptr());
    let log_handle: HANDLE = unsafe { OpenEventLogW(None, log_name_ptr)? };
    Ok(log_handle)
}

async fn read_event_log(log_handle: HANDLE, mut stream: TcpStream) -> Result<u32, Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![0u8; 4096];
    let mut bytes_read: u32 = 0u32;
    let mut min_number_of_bytes_needed: u32 = 0u32;

    unsafe {
        loop {
            let read_flags_value: u32 = EVENTLOG_SEQUENTIAL_READ.0 | EVENTLOG_FORWARDS_READ;
            let read_flags: READ_EVENT_LOG_READ_FLAGS = READ_EVENT_LOG_READ_FLAGS(read_flags_value);
            let success: Result<(), WinError> = ReadEventLogW(
                log_handle,
                read_flags,
                0,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
                &mut min_number_of_bytes_needed,
            );

            if success.is_err() {
                let error_code: windows::Win32::Foundation::WIN32_ERROR = GetLastError();
                if error_code == ERROR_INSUFFICIENT_BUFFER {
                    buffer.resize(min_number_of_bytes_needed as usize, 0);
                } else {
                    // Handle other errors
                    return Err(Box::new(WinError::from_win32()) as Box<dyn Error>);
                }
            }

            let mut offset: usize = 0;
            while offset < bytes_read as usize {
                let record_ptr: *const EVENTLOGRECORD = buffer.as_ptr().add(offset) as *const EVENTLOGRECORD;
                let record: &EVENTLOGRECORD = &*record_ptr;

                let event_id: u32 = record.EventID & 0xFFFF_FFFF;
                let record_number: u32 = record.RecordNumber;
                let time_generated: u32 = record.TimeGenerated;

                let datetime: std::time::SystemTime = UNIX_EPOCH + Duration::from_secs(time_generated as u64);
                let datetime: chrono::DateTime<chrono::Local> = datetime.into();
                let time_generated_str: String = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                let source_name_ptr: *const u16 = (record_ptr as *const u8).add(size_of::<EVENTLOGRECORD>()) as *const u16;
                let source_name: String = {
                    let mut len: usize = 0;
                    while *source_name_ptr.add(len) != 0 {
                        len += 1;
                    }
                    let slice: &[u16] = std::slice::from_raw_parts(source_name_ptr, len);
                    String::from_utf16_lossy(slice)
                };

                let computer_name_ptr: *const u16 = source_name_ptr.add(source_name.len() + 1);
                let computer_name: String = {
                    let mut len: usize = 0;
                    while *computer_name_ptr.add(len) != 0 {
                        len += 1;
                    }
                    let slice: &[u16] = std::slice::from_raw_parts(computer_name_ptr, len);
                    String::from_utf16_lossy(slice)
                };

                println!("Event ID: {}", event_id);
                println!("Record Number: {}", record_number);
                println!("Time Generated: {}", time_generated_str);
                println!("Source Name: {}", source_name);
                println!("Computer Name: {}", computer_name);
                println!(" ");
                println!("sending...");
                let data = "finished\n";
                let mut writer = BufWriter::new(&mut stream);

                writer.write_all(data.as_bytes()).await?;
                writer.flush().await?;
                // stream.write_all(b"finished").await?;
                offset += record.Length as usize;
            }
        }
    };

    Ok(bytes_read)
}

async fn log_windows_os(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let log_handle: HANDLE = open_windows_event_log()?;
    let bytes_read: u32 = read_event_log(log_handle, stream).await?;

    println!("Read {} bytes from the event log", bytes_read);

    unsafe {
        let _ = CloseEventLog(log_handle);
    };

    Ok(())
}

async fn log_linux_os(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("RUNNING LINUX");

    Ok(())
}

async fn log_mac_os(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("RUNNING MAC");

    Ok(())
}
