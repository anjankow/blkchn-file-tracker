use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
};

#[tokio::main]
async fn main() {
    let mut inotify = inotify::Inotify::init().expect("Error while initializing inotify instance");

    let event_mask = inotify::WatchMask::ATTRIB
        | inotify::WatchMask::CLOSE_WRITE
        | inotify::WatchMask::CREATE
        | inotify::WatchMask::DELETE
        | inotify::WatchMask::MOVE
        | inotify::WatchMask::OPEN;
    // Watch for modify and close events.
    inotify
        .watches()
        .add("./c", event_mask)
        .expect("Failed to add file watch");

    loop {
        let events = read_events(&mut inotify)
            .await
            .expect("Unexpected error while reading events");
        println!("FILES: {:?}", events.file_names);
    }
}

#[derive(Debug, Clone)]
struct Error;

struct Event {
    pub event_type: EventType,
    pub name: String,
}

#[derive(Debug)]
enum EventType {
    AttributeChanged,
    Created,
    Deleted,
    Moved,
    Opened,
    Written,
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
        if mask.contains(inotify::EventMask::MOVED_FROM | inotify::EventMask::MOVED_TO) {
            ret.push(EventType::Moved);
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

struct Events {
    file_names: HashSet<String>,
    events: Vec<Event>,
}

struct EventsCollector {
    file_names: Box<HashSet<String>>,
    events: Box<Vec<Event>>,
    // memo checking for MOVE duplicates
}

impl EventsCollector {
    pub fn new() -> EventsCollector {
        EventsCollector {
            events: Box::new(Vec::new()),
            file_names: Box::new(HashSet::new()),
        }
    }
    pub fn add(&mut self, e: inotify::Event<&std::ffi::OsStr>) {
        let name_opt = e.name.map(|s| {
            s.to_string_lossy()
                .to_string()
        });
        if name_opt.is_none() {
            // we want only the events with associated file names
            return;
        }
        let event_types_opt = EventType::from_mask(e.mask);
        if event_types_opt.is_none() {
            // no known event types
            return;
        }
        let event_types = event_types_opt.unwrap();

        let name = name_opt.unwrap();

        let mut events = Vec::with_capacity(event_types.len());

        for event_type in event_types {
            println!("event type: {:?} for file {}", &event_type, name);

            events.push(Event {
                event_type: event_type,
                name: name.clone(),
            });
        }

        self.file_names
            .insert(name.clone());
    }

    pub fn get(&mut self) -> Events {
        let ret = Events {
            file_names: std::mem::take(&mut self.file_names),
            events: std::mem::take(&mut self.events),
        };

        println!("Returning events, len: {}", ret.events.len());
        ret
    }
}

async fn read_events<'a>(inotify: &mut inotify::Inotify) -> Result<Events, Error> {
    let mut buffer = [0; 1024];
    let events = inotify
        .read_events_blocking(&mut buffer)
        .expect("Error while reading events");

    let mut collector = EventsCollector::new();
    for event in events {
        collector.add(event);
    }

    Ok(collector.get())
}
