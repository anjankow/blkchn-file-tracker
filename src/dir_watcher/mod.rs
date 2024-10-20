use crate::error::Error;
use crate::event::{self, Event, EventType, FileInfo};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;
use std::{collections::HashMap, fmt::Display, io, sync::mpsc};

pub struct DirWatcher {
    inotify: inotify::Inotify,
    dir: String,
}

impl DirWatcher {
    pub fn new(directory: &str, event_types: Vec<EventType>) -> Result<DirWatcher, Error> {
        let watch_mask = event_types_to_watch_mask(event_types);
        if watch_mask.is_empty() {
            return Err(Error::new("No known event types found in event_types"));
        }

        let inotify = inotify::Inotify::init()?;

        inotify
            .watches()
            .add(directory, watch_mask)?;

        Ok(DirWatcher {
            inotify,
            dir: directory.to_string(),
        })
    }

    pub fn run_blocking(&mut self, tx: mpsc::Sender<Event>) -> Result<(), Error> {
        loop {
            // Read events from inotify
            let mut buffer = [0; 1024];
            let events = self
                .inotify
                .read_events_blocking(&mut buffer)?;

            // Extract them and enrich with file metadata
            let events = self.extract_events(events)?;

            // Send events to the listener
            for event in events {
                let _ = tx
                    .send(event.clone())
                    .and_then(|_| {
                        // println!("Event reported: {} {:?}", event.event_type, event.file_path);
                        Ok(())
                    })
                    .is_err_and(|e| {
                        println!(
                            "Failed to send event of a file: {:?}, reason: {}",
                            event.file_path, e
                        );
                        false
                    });
            }
        }
    }

    // Inotify event is a mask, which means that potentially more events
    // are encoded within one inotify::Event. We want to create a separate
    // event for each of them. Stat of a file should be checked just once for each file
    // every time when `inotify.read_events_blocking` returns,
    // so every time when this function is called.
    fn extract_events(&self, inotify_events: inotify::Events) -> Result<Vec<Event>, Error> {
        let mut file_infos: HashMap<String, FileInfo> = HashMap::new();
        let mut ret_events = Vec::new();

        for ie in inotify_events {
            let file_path = match ie.name {
                // We care only about the events with associated file names
                None => continue,
                Some(n) => std::path::Path::new(&self.dir)
                    .join(
                        n.to_string_lossy()
                            .to_string(),
                    )
                    .to_str()
                    .unwrap()
                    .to_string(),
            };

            let event_types = match EventType::from_event_mask(ie.mask) {
                // No known events found
                None => continue,
                Some(et) => et,
            };

            // Now we get the file metadata, if not present in the map
            let file_info = match file_infos.get(&file_path) {
                Some(fi) => Some(fi.clone()),
                None => match self.read_file_metadata(&file_path) {
                    Err(e) => {
                        if !e
                            .io_kind()
                            // To skip errors reported by potentially deleted files
                            .is_some_and(|e| e == io::ErrorKind::NotFound)
                        {
                            println!("Failed to read file info of a file {}: {}", file_path, e);
                        }
                        None
                    }
                    Ok(fi) => {
                        // We want to store it in our map, maybe there are more events
                        // associated with this file in the input events.
                        file_infos.insert(file_path.clone(), fi.clone());
                        Some(fi)
                    }
                },
            };

            for event_type in event_types {
                // Enrich with metadata only if the event type is not 'Deleted'
                let file_info = match event_type {
                    event::EventType::Deleted => None,
                    _ => file_info.clone(),
                };

                ret_events.push(Event {
                    event_type: event_type,
                    file_path: file_path.clone(),
                    solana_ts_received_at: 0, // filled in by the listener
                    file_info: file_info,
                });
            }
        }
        Ok(ret_events)
    }

    fn read_file_metadata(&self, file_path: &str) -> Result<FileInfo, Error> {
        let metadata = fs::metadata(file_path)?;

        let to_unix_ts = |t: std::time::SystemTime| -> i128 {
            match t.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => n.as_secs().into(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            }
        };

        Ok(FileInfo {
            size: metadata.len(),
            mode: metadata.permissions().mode(),
            access_ts: metadata
                .accessed()
                .ok()
                .map(|t| to_unix_ts(t)),
            modify_ts: metadata
                .modified()
                .ok()
                .map(|t| to_unix_ts(t)),
            created_ts: metadata
                .created()
                .ok()
                .map(|t| to_unix_ts(t)),
        })
    }
}

impl EventType {
    fn from_event_mask(mask: inotify::EventMask) -> Option<Vec<EventType>> {
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

    fn to_watch_mask(&self) -> inotify::WatchMask {
        match self {
            EventType::AttributeChanged => inotify::WatchMask::ATTRIB,
            EventType::Created => inotify::WatchMask::CREATE,
            EventType::Deleted => inotify::WatchMask::DELETE,
            EventType::MovedFrom => inotify::WatchMask::MOVED_FROM,
            EventType::MovedTo => inotify::WatchMask::MOVED_TO,
            EventType::Opened => inotify::WatchMask::OPEN,
            EventType::Written => inotify::WatchMask::CLOSE_WRITE,
        }
    }
}

fn event_types_to_watch_mask(event_types: Vec<EventType>) -> inotify::WatchMask {
    let mut ret = inotify::WatchMask::empty();
    for et in event_types {
        ret |= et.to_watch_mask();
    }
    ret
}
