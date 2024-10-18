use std::env;

use borsh::BorshSerialize;
use file_event_tracker as et;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::Signer,
    transaction::Transaction,
};

#[test]
fn test_add_event() {
    let client = get_rpc_client();

    // payer keys
    let default_path_env = env::var("HOME")
        .map(|mut s| {
            s.push_str("/.config/solana/id.json");
            s
        })
        .unwrap();
    let wallet_keypair_path = env::var("WALLET_KEYPAIR").unwrap_or(default_path_env);
    println!("Wallet keys obtained from: {}", wallet_keypair_path);
    let wallet = solana_sdk::signer::keypair::read_keypair_file(wallet_keypair_path).unwrap();

    // solana current time
    let slot = client.get_slot().unwrap();
    let now = client
        .get_block_time(slot)
        .unwrap() as i128;
    // create event data
    let data = et::event::Event {
        event_type: et::event::EventType::Written,
        file_path: "/home/user/file1.txt".to_string(),
        solana_ts_received_at: now,
        file_info: None,
    };
    let mut serialized_data = Vec::<u8>::new();
    data.serialize(&mut serialized_data)
        .unwrap();

    // prepare the instruction
    // first, find the program id
    let program_id = env::var("PROGRAM_ID")
        .unwrap_or("BtzKw3sZRdNd8DqToNSd8KRLVU9jYemcEJEgHWupKDjd".to_string());
    let program: Pubkey = program_id
        .parse::<Pubkey>()
        .expect("Invalid program id (check ${PROGRAM_ID})");
    let accounts = [AccountMeta::new(wallet.pubkey(), true)].to_vec();
    let instruction = Instruction::new_with_bytes(program, &serialized_data, accounts);

    let recent_blockhash = client
        .get_latest_blockhash()
        .unwrap();

    // now invoke the instruction with a new transaction
    let transaction: Transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    let client_signature = client
        .send_transaction(&transaction)
        .unwrap();
    println!("Client signature: {}", client_signature.to_string());
}

fn get_rpc_client() -> solana_client::rpc_client::RpcClient {
    let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:8899".to_string());
    let client = RpcClient::new(rpc_url);
    client
}
