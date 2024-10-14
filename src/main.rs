mod dir_watcher;

fn main() {
    let dir = "./c"; // path relative to Cargo.toml
    let (tx, rx) = std::sync::mpsc::channel();
    let event_types = vec![
        dir_watcher::EventType::AttributeChanged,
        dir_watcher::EventType::Created,
        dir_watcher::EventType::Deleted,
        dir_watcher::EventType::MovedFrom,
        dir_watcher::EventType::MovedTo,
        dir_watcher::EventType::Written,
    ];
    dir_watcher::DirWatcher::new(dir, event_types)
        .unwrap()
        .run_blocking(tx)
        .expect("Should never return");
}
