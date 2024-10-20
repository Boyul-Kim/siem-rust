use std::env;
use std::time::UNIX_EPOCH;
use std::time::Duration;
use windows::Win32::System::EventLog::{
    OpenEventLogW, ReadEventLogW, CloseEventLog, EVENTLOG_SEQUENTIAL_READ, READ_EVENT_LOG_READ_FLAGS, EVENTLOGRECORD
};
use windows::Win32::System::SystemServices::EVENTLOG_FORWARDS_READ;
use windows::Win32::Foundation::{HANDLE, GetLastError, ERROR_INSUFFICIENT_BUFFER};
use windows::core::{Error, PCWSTR};
use std::os::windows::ffi::OsStrExt;
use std::ffi::OsStr;

fn main() {

    let os_type: &str = env::consts::OS;

    let run_logging: fn() -> Result<(), Error> = match os_type {
        "windows" =>  log_windows_os,
        "linux" => log_linux_os,
        "mac" => log_mac_os,
        _ => {
            println!("Unsupported OS");
            return;
        }
    };

    let _ = run_logging();
}

fn open_windows_event_log() -> Result<HANDLE, Error> {
    let log_name: Vec<u16> = OsStr::new("System")
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<u16>>();
    let log_name_ptr: PCWSTR = PCWSTR(log_name.as_ptr());
    let log_handle: HANDLE = unsafe { OpenEventLogW(None, log_name_ptr)? };
    Ok(log_handle)
}

fn read_event_log(log_handle: HANDLE) -> Result<u32, Error> {
    let mut buffer: Vec<u8> = vec![0u8; 4096];
    let mut bytes_read: u32 = 0u32;
    let mut min_number_of_bytes_needed: u32 = 0u32;

    unsafe {
        loop{
            let read_flags_value: u32 = EVENTLOG_SEQUENTIAL_READ.0 | EVENTLOG_FORWARDS_READ;
            let read_flags: READ_EVENT_LOG_READ_FLAGS = READ_EVENT_LOG_READ_FLAGS(read_flags_value);
            let success: Result<(), Error> = ReadEventLogW(
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
                    return Err(Error::from_win32());
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

                offset += record.Length as usize;
            }
        }
    };

    Ok(bytes_read)
}

fn log_windows_os() -> Result<(), Error> {
    let log_handle: HANDLE = open_windows_event_log()?;
    let bytes_read: u32 = read_event_log(log_handle)?;

    println!("Read {} bytes from the event log", bytes_read);

    unsafe { 
        let _ = CloseEventLog(log_handle); 
    };

    Ok(())
}

fn log_linux_os() -> Result<(), Error> {
    println!("RUNNING LINUX");

    Ok(())
}

fn log_mac_os() -> Result<(), Error> {
    println!("RUNNING MAC");

    Ok(())
}