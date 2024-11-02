// src/types.rs

#[cfg(target_os = "windows")]
use windows::Win32::System::EventLog::EVENTLOGRECORD;
#[cfg(target_os = "windows")]
use serde::{Serialize, Deserialize};

#[cfg(target_os = "windows")]
#[derive(Serialize, Deserialize, Debug)]
pub struct WrapperEventlogRecord {
    pub length: u32,
    pub reserved: u32,
    pub record_number: u32,
    pub time_generated: u32,
    pub time_written: u32,
    pub event_id: u32,
    pub num_strings: u16,
    pub event_category: u16,
    pub reserved_flags: u16,
    pub closing_record_number: u32,
    pub string_offset: u32,
    pub user_sid_length: u32,
    pub user_sid_offset: u32,
    pub data_length: u32,
    pub data_offset: u32,
}

#[cfg(target_os = "windows")]
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
