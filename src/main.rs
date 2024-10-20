mod dir_watcher;
mod error;
mod event;
mod solana_client;

fn main() {
    let dir = "./tmp"; // path relative to Cargo.toml
    let solana_url = "http://127.0.0.1:8899";

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
    solana_client::SolanaClient::new(solana_url)
        .process_events(rx)
        .expect("Should never return");
}
