mod dir_watcher;
mod event;

fn main() {
    let dir = "./c"; // path relative to Cargo.toml
    let (tx, rx) = std::sync::mpsc::channel();
    let event_types = vec![
        event::EventType::AttributeChanged,
        event::EventType::Created,
        event::EventType::Deleted,
        event::EventType::MovedFrom,
        event::EventType::MovedTo,
        event::EventType::Written,
    ];

    // Run the dir watcher in a thread
    std::thread::spawn(move || {
        dir_watcher::DirWatcher::new(dir, event_types)
            .unwrap()
            .run_blocking(tx)
            .expect("Should never return");
    });

    // And pass the events to the consumer
    for event in rx {
        println!("Consumer received event: {}", event);
    }
}
