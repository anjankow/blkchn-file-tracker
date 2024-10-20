use std::env;

use borsh::BorshSerialize;
use file_event_tracker as et;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use time::OffsetDateTime;

const DEFAULT_PROGRAM_ID: &str = "BtzKw3sZRdNd8DqToNSd8KRLVU9jYemcEJEgHWupKDjd";

#[test]
fn test_create_pda() {
    let client = get_rpc_client();

    // payer keys
    let payer = get_wallet();
    let payer_key = payer.pubkey();

    let program = get_program();

    // Get the amount of lamports needed to pay for the vault's rent
    let vault_account_size = usize::try_from(et::processor::VAULT_ACCOUNT_SIZE).unwrap();
    let lamports = client
        .get_minimum_balance_for_rent_exemption(vault_account_size)
        .unwrap();

    let (pda_pubkey, pda_bump_seed) = derive_user_pda(&program, &payer_key);

    // The on-chain program's instruction data, imported from that program's crate.
    let instr_data = et::instruction::EventTrackerInstruction::Initialize(
        et::instruction::InitializeInstructionData {
            lamports,
            pda_bump_seed,
        },
    )
    .pack()
    .unwrap();

    // The accounts required by both our on-chain program and the system program's
    // `create_account` instruction, including the vault's address.
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true /* is_signer */),
        AccountMeta::new(pda_pubkey, false),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];

    // Create the instruction by serializing our instruction data via borsh
    let instruction = Instruction::new_with_bytes(program, &instr_data, accounts);

    let blockhash = client
        .get_latest_blockhash()
        .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    client
        .send_and_confirm_transaction(&transaction)
        .unwrap();
}

fn derive_user_pda(program: &Pubkey, payer: &Pubkey) -> (Pubkey, u8) {
    // Derive the PDA from the payer account, a string representing the unique
    // purpose of the account ("vault"), and the address of our on-chain program.
    let seeds = &[et::processor::PDA_SEED_PREFIX, payer.as_ref()];
    let (pda_pubkey, pda_bump_seed) = Pubkey::find_program_address(seeds, &program);
    (pda_pubkey, pda_bump_seed)
}

#[test]
fn test_add_event() {
    let client = get_rpc_client();

    // payer keys
    let wallet = get_wallet();

    // solana current time
    let solana_current_time = get_solana_unix_timestamp(&client.url()).unwrap() as i128;

    // create event data
    let data = et::event::Event {
        event_type: et::event::EventType::Written,
        file_path: "/home/user/file1.txt".to_string(),
        solana_ts_received_at: solana_current_time as i128,
        file_info: None,
    };
    let mut serialized_data = Vec::<u8>::new();
    data.serialize(&mut serialized_data)
        .unwrap();

    // prepare the instruction
    // first, find the program id
    let program = get_program();
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

fn get_event_for_testing() -> et::event::Event {
    et::event::Event {
        event_type: et::event::EventType::Deleted,
        file_path: "/home/user/file1.txt".to_string(),
        solana_ts_received_at: OffsetDateTime::now_utc().unix_timestamp() as i128,
        file_info: None,
    }
}

fn get_solana_unix_timestamp(url: &str) -> Result<i64, ureq::Error> {
    let sysvar_clock_address = "SysvarC1ock11111111111111111111111111111111";

    let req_body = ureq::json!({
    "jsonrpc": "2.0",
    "id": 1,
        "method": "getAccountInfo",
        "params": [
            sysvar_clock_address,
            {
                "encoding": "jsonParsed",
            },
        ],
    });

    // https://solana.com/docs/rpc/http/getaccountinfo
    let recv_body: std::collections::HashMap<String, serde_json::Value> = ureq::post(&url)
        .send_json(&req_body)?
        .into_json()?;
    // println!("recv body: {:?}", recv_body);

    let res = recv_body
        .get("result")
        .and_then(|res| res.get("value"))
        .and_then(|res| res.get("data"))
        .and_then(|res| res.get("parsed"))
        .and_then(|res| res.get("info"))
        .and_then(|res| res.get("unixTimestamp"));

    let res = res
        .map(|r| r.as_i64().unwrap())
        .or_else(|| {
            println!(
            "Failed to find unixTimestamp in the response, returning the system's unix_timestamp");
            println!("{:?}", recv_body);
            // we will ignore this error and just return current system time for tests
            Some(time::OffsetDateTime::now_utc().unix_timestamp())
        })
        .unwrap();
    Ok(res)
}

#[test]
fn test_get_solana_unix_timestamp() {
    let res = get_solana_unix_timestamp("http://localhost:8899").unwrap();
    assert!(res > 0);
    println!("{}", res);
}

fn get_rpc_client() -> solana_client::rpc_client::RpcClient {
    let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:8899".to_string());
    let client = RpcClient::new(rpc_url);
    client
}

fn get_program() -> Pubkey {
    let program_id = env::var("PROGRAM_ID").unwrap_or(DEFAULT_PROGRAM_ID.to_string());
    let program: Pubkey = program_id
        .parse::<Pubkey>()
        .expect("Invalid program id (check ${PROGRAM_ID})");
    program
}

fn get_wallet() -> Keypair {
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
