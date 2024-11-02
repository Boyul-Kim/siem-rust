use std::env;
use windows::Win32::System::EventLog::{
    OpenEventLogW, ReadEventLogW, CloseEventLog, EVENTLOG_SEQUENTIAL_READ, READ_EVENT_LOG_READ_FLAGS, EVENTLOGRECORD, GetOldestEventLogRecord, GetNumberOfEventLogRecords, EVENTLOG_SEEK_READ
};
use windows::Win32::System::SystemServices::EVENTLOG_FORWARDS_READ;
use windows::Win32::Foundation::{HANDLE, GetLastError, ERROR_INSUFFICIENT_BUFFER};
use windows::core::{Error as WinError, PCWSTR};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufWriter};
use std::error::Error;
use bincode;
use serde::{Serialize, Deserialize};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let os_type: &str = env::consts::OS;

    match os_type {
        "windows" => log_windows_os(stream).await?,
        "linux" => log_linux_os(stream).await?,
        "macos" => log_mac_os(stream).await?,
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
        .collect();
    let log_name_ptr: PCWSTR = PCWSTR(log_name.as_ptr());
    let log_handle: HANDLE = unsafe { OpenEventLogW(None, log_name_ptr)? };
    if log_handle.is_invalid() {
        return Err(Box::new(WinError::from_win32()));
    }
    Ok(log_handle)
}

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

impl From<EVENTLOGRECORD> for WrapperEventlogRecord {
    fn from(orig: EVENTLOGRECORD) -> Self {
        WrapperEventlogRecord {
            length: orig.Length,
            reserved: orig.Reserved,
            record_number: orig.RecordNumber,
            time_generated: orig.TimeGenerated,
            time_written: orig.TimeWritten,
            event_id: orig.EventID,
            num_strings: orig.NumStrings,
            event_category: orig.EventCategory,
            reserved_flags: orig.ReservedFlags,
            closing_record_number: orig.ClosingRecordNumber,
            string_offset: orig.StringOffset,
            user_sid_length: orig.UserSidLength,
            user_sid_offset: orig.UserSidOffset,
            data_length: orig.DataLength,
            data_offset: orig.DataOffset,
        }
    }
}

async fn read_event_log(log_handle: HANDLE, mut stream: TcpStream) -> Result<u32, Box<dyn Error>> {
    let mut buffer: Vec<u8> = vec![0u8; 4096];
    let mut bytes_read: u32 = 0;
    let mut min_number_of_bytes_needed: u32 = 0;

    let mut oldest_record: u32 = 0;
    let mut total_records: u32 = 0;

    unsafe {
        let _ = GetOldestEventLogRecord(log_handle, &mut oldest_record);
        let _ = GetNumberOfEventLogRecords(log_handle, &mut total_records);
    }

    // Calculate the latest record number
    let mut last_record_read = oldest_record + total_records - 1;

    unsafe {
        loop {
            let read_flags = READ_EVENT_LOG_READ_FLAGS(EVENTLOG_SEQUENTIAL_READ.0 | EVENTLOG_FORWARDS_READ);
            let success: Result<(), WinError> = ReadEventLogW(
                log_handle,
                read_flags,
                last_record_read + 1, // Start from the beginning for testing
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
                &mut min_number_of_bytes_needed,
            );

            if success.is_err() {
                let error_code = GetLastError();
                if error_code == ERROR_INSUFFICIENT_BUFFER {
                    buffer.resize(min_number_of_bytes_needed as usize, 0);
                    continue;
                } else if error_code.to_hresult().to_string() == "0x80070026" {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                } else {
                    return Err(Box::new(WinError::from_win32()));
                }
            }

            let mut offset = 0;
            while offset < bytes_read as usize {
                let record_ptr: *const EVENTLOGRECORD = buffer.as_ptr().add(offset) as *const EVENTLOGRECORD;
                let record: &EVENTLOGRECORD = &*record_ptr;

                let wrapper_event_log_record: WrapperEventlogRecord = record.clone().into();
                let serialized_data = bincode::serialize(&wrapper_event_log_record)?;
                let data_len = (serialized_data.len() as u32).to_be_bytes();
                let mut writer = BufWriter::new(&mut stream);
                writer.write_all(&data_len).await?;
                writer.write_all(&serialized_data).await?;
                writer.flush().await?;
                offset += record.Length as usize;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
    Ok(bytes_read)
}

async fn log_windows_os(stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let log_handle: HANDLE = open_windows_event_log()?;
    let bytes_read: u32 = read_event_log(log_handle, stream).await?;
    println!("Read {} bytes from the event log", bytes_read);
    unsafe {
        CloseEventLog(log_handle);
    }
    Ok(())
}

async fn log_linux_os(_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("RUNNING LINUX");
    Ok(())
}

async fn log_mac_os(_stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("RUNNING MAC");
    Ok(())
}
