use std::env;

use borsh::BorshSerialize;
use file_event_tracker as et;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::program,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signer::Signer,
    system_instruction, system_program,
    transaction::Transaction,
};
use time::OffsetDateTime;

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

    // create event data
    let data = et::event::Event {
        event_type: et::event::EventType::Written,
        file_name: "file1.txt".to_string(),
        received_at: et::event::OffsetDateTime::from(time::OffsetDateTime::now_utc()),
        file_info: None,
    };
    let mut serialized_data = Vec::<u8>::new();
    data.serialize(&mut serialized_data)
        .unwrap();

    // check what is the rent for storing such data
    let reserved_space = serialized_data.len();

    // prepare the instruction
    // first, find the program id
    let program_id = env::var("PROGRAM_ID")
        .unwrap_or("EqWRXakMW7rJvT4dMUd1uUdNyKbbvEP48kK9NCevfAJ4".to_string());
    let program: Pubkey = program_id
        .parse::<Pubkey>()
        .expect("Invalid program id (check ${PROGRAM_ID})");
    let accounts = [AccountMeta::new(wallet.pubkey(), true)].to_vec();
    let instruction = Instruction::new_with_borsh(program, &serialized_data, accounts);

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
