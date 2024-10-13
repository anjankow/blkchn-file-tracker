use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;
use std::{collections::HashMap, fmt::Display, io, sync::mpsc};

#[derive(Debug)]
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

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Error {
        Error { message }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl EventType {
    fn from_mask(mask: inotify::EventMask) -> Option<Vec<EventType>> {
        let mut ret = Vec::with_capacity(1);
        if mask.contains(inotify::EventMask::ATTRIB) {
            ret.push(EventType::AttributeChanged);
        }
        if mask.contains(inotify::EventMask::CREATE) {
            ret.push(EventType::Created);
        }
        if mask.contains(inotify::EventMask::DELETE) {
            ret.push(EventType::Deleted);
        }
        if mask.contains(inotify::EventMask::MOVED_FROM) {
            ret.push(EventType::MovedFrom);
        }
        if mask.contains(inotify::EventMask::MOVED_TO) {
            ret.push(EventType::MovedTo);
        }
        if mask.contains(inotify::EventMask::OPEN) {
            ret.push(EventType::Opened);
        }
        if mask.contains(inotify::EventMask::CLOSE_WRITE) {
            ret.push(EventType::Written);
        }

        if ret.len() > 0 {
            return Some(ret);
        }
        None
    }
}

pub struct DirWatcher {
    inotify: inotify::Inotify,
}

impl DirWatcher {
    pub fn new(directory: &str) -> Result<DirWatcher, Error> {
        let inotify = inotify::Inotify::init()?;

        let event_mask = inotify::WatchMask::ATTRIB
            | inotify::WatchMask::CLOSE_WRITE
            | inotify::WatchMask::CREATE
            | inotify::WatchMask::DELETE
            | inotify::WatchMask::MOVE
            | inotify::WatchMask::OPEN;

        inotify
            .watches()
            .add(directory, event_mask)?;

        Ok(DirWatcher { inotify })
    }

    pub fn run_blocking(&mut self, tx: mpsc::Sender<Event>) -> Result<(), Error> {
        loop {
            let events = self.read_events()?;

            for event in events {
                let _ = tx
                    .send(event.clone())
                    .and_then(|_| {
                        println!("Event reported: {} {:?}", event.event_type, event.file_name);
                        Ok(())
                    })
                    .is_err_and(|e| {
                        println!(
                            "Failed to send event of a file: {:?}, reason: {}",
                            event.file_name, e
                        );
                        false
                    });
            }
        }
    }

    fn read_events(&mut self) -> Result<Vec<Event>, Error> {
        let mut buffer = [0; 1024];
        let events = self
            .inotify
            .read_events_blocking(&mut buffer)?;

        extract_events(events)
    }
}

// Inotify event is a mask, which means that potentially more events
// are encoded within one inotify::Event. We want to create a separate
// event for each of them. Stat of a file should be checked just once for each file
// every time when `inotify.read_events_blocking` returns,
// so every time when this function is called.
fn extract_events(inotify_events: inotify::Events) -> Result<Vec<Event>, Error> {
    let mut file_infos: HashMap<String, FileInfo> = HashMap::new();
    let mut ret_events = Vec::new();

    for ie in inotify_events {
        let file_name = match ie.name {
            // We care only about the events with associated file names
            None => continue,
            Some(n) => n
                .to_string_lossy()
                .to_string(),
        };

        let event_types = match EventType::from_mask(ie.mask) {
            // No known events found
            None => continue,
            Some(et) => et,
        };

        // Now we get the file metadata, if not present in the map.
        let file_info = match file_infos.get(&file_name) {
            Some(fi) => Some(fi.clone()),
            None => match read_file_metadata(&file_name) {
                Err(e) => {
                    println!("Failed to read file info of a file {}: {}", file_name, e);
                    None
                }
                Ok(fi) => {
                    // We want to store it in our map, maybe there are more events
                    // associated with this file in the input events.
                    file_infos.insert(file_name.clone(), fi.clone());
                    Some(fi)
                }
            },
        };

        for event_type in event_types {
            ret_events.push(Event {
                event_type: event_type,
                file_name: file_name.clone(),
                file_info: file_info.clone(),
            });
        }
    }
    Ok(ret_events)
}

fn read_file_metadata(file_name: &str) -> Result<FileInfo, Error> {
    let metadata = fs::metadata(&file_name)?;

    Ok(FileInfo {
        size: metadata.len(),
        mode: metadata.permissions().mode(),
        access_ts: metadata.accessed().ok(),
        modify_ts: metadata.modified().ok(),
        created_ts: metadata.created().ok(),
    })
}
