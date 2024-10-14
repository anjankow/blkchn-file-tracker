use std::{fmt::Display, time::SystemTime};

pub enum EventType {
    AttributeChanged,
    Created,
    Deleted,
    MovedFrom,
    MovedTo,
    Opened,
    Written,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttributeChanged => write!(f, "AttributeChanged"),
            Self::Created => write!(f, "Created"),
            Self::Deleted => write!(f, "Deleted"),
            Self::MovedFrom => write!(f, "MovedFrom"),
            Self::MovedTo => write!(f, "MovedTo"),
            Self::Opened => write!(f, "Opened"),
            Self::Written => write!(f, "Written"),
        }
    }
}

impl Clone for EventType {
    fn clone(&self) -> Self {
        match self {
            Self::AttributeChanged => Self::AttributeChanged,
            Self::Created => Self::Created,
            Self::Deleted => Self::Deleted,
            Self::MovedFrom => Self::MovedFrom,
            Self::MovedTo => Self::MovedTo,
            Self::Opened => Self::Opened,
            Self::Written => Self::Written,
        }
    }
}

pub struct Event {
    pub event_type: EventType,
    pub file_name: String,

    pub file_info: Option<FileInfo>,
}

impl Clone for Event {
    fn clone(&self) -> Self {
        Self {
            event_type: self.event_type.clone(),
            file_name: self.file_name.clone(),
            file_info: self.file_info.clone(),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file_name, self.event_type)
    }
}

pub struct FileInfo {
    pub size: u64,
    pub mode: u32, // libc::mode_t;

    pub access_ts: Option<SystemTime>,
    pub modify_ts: Option<SystemTime>,
    pub created_ts: Option<SystemTime>,
}

impl Clone for FileInfo {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            mode: self.mode,
            access_ts: self.access_ts,
            modify_ts: self.modify_ts,
            created_ts: self.created_ts,
        }
    }
}
