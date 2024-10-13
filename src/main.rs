mod dir_watcher;

fn main() {
    let dir = "./c"; // path relative to Cargo.toml
    let (tx, rx) = std::sync::mpsc::channel();
    dir_watcher::DirWatcher::new(dir)
        .unwrap()
        .run_blocking(tx)
        .expect("Should never return");
}
