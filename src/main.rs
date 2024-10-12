use inotify::{Inotify, WatchMask};

fn main() {
    let mut inotify = Inotify::init().expect("Error while initializing inotify instance");

    // Watch for modify and close events.
    inotify
        .watches()
        .add(
            "/tmp/inotify-rs-test-file",
            WatchMask::MODIFY | WatchMask::CLOSE,
        )
        .expect("Failed to add file watch");

    // Read events that were added with `Watches::add` above.
    let mut buffer = [0; 1024];
    let events = inotify
        .read_events_blocking(&mut buffer)
        .expect("Error while reading events");

    for event in events {
        // Handle event
    }
}
