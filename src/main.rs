use std::env;

mod dir_watcher;
mod error;
mod event;
mod solana_client;

const DEFAULT_PROGRAM_ID: &str = "BtzKw3sZRdNd8DqToNSd8KRLVU9jYemcEJEgHWupKDjd";

fn main() {
    let dir = "./tmp"; // path relative to Cargo.toml
    let solana_url = "http://127.0.0.1:8899";
    let mut client = solana_client::SolanaClient::new(solana_url, get_program(), get_wallet());
    client.init_account().unwrap();

    let event_types = vec![
        event::EventType::AttributeChanged,
        event::EventType::Created,
        event::EventType::Deleted,
        event::EventType::MovedFrom,
        event::EventType::MovedTo,
        event::EventType::Written,
    ];

    let (tx, rx) = std::sync::mpsc::channel();

    // Start a solana client processing events
    std::thread::spawn(move || {
        client
            .process_events(rx)
            .expect("Should never return");
    });

    // And run a dir watcher
    dir_watcher::DirWatcher::new(dir, event_types)
        .unwrap()
        .run_blocking(tx)
        .expect("Should never return");
}

fn get_program() -> solana_sdk::pubkey::Pubkey {
    let program_id = env::var("PROGRAM_ID").unwrap_or(DEFAULT_PROGRAM_ID.to_string());
    let program: solana_sdk::pubkey::Pubkey = program_id
        .parse::<solana_sdk::pubkey::Pubkey>()
        .expect("Invalid program id (check ${PROGRAM_ID})");
    program
}

fn get_wallet() -> solana_sdk::signer::keypair::Keypair {
    let default_path_env = env::var("HOME")
        .map(|mut s| {
            s.push_str("/.config/solana/id.json");
            s
        })
        .unwrap();
    let wallet_keypair_path = env::var("WALLET_KEYPAIR").unwrap_or(default_path_env);
    println!("Wallet keys obtained from: {}", wallet_keypair_path);
    let wallet = solana_sdk::signer::keypair::read_keypair_file(wallet_keypair_path).unwrap();
    return wallet;
}
