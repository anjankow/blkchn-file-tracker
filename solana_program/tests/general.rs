use std::env;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::program, instruction::Instruction, pubkey::Pubkey, signer::Signer,
    system_instruction, system_program, transaction::Transaction,
};

#[test]
fn test_accounts() {
    let (client, program, wallet) = get_client_program_wallet_accounts();

    let wallet_account = client
        .get_account(&wallet)
        .expect("Wallet account doesn't exist");
    // owner of this account is the System Program
    assert_eq!(
        wallet_account
            .owner
            .to_string(),
        "11111111111111111111111111111111"
    );
    println!("Wallet account state: {}", wallet_account.lamports);

    // check the program

    let program_account = client
        .get_account(&program)
        .expect("Program account doesn't exist");
    // owner of a program account is BPF Loader
    assert_eq!(
        program_account
            .owner
            .to_string(),
        "BPFLoaderUpgradeab1e11111111111111111111111"
    );
    assert_eq!(program_account.executable, true);
    println!("Program account state: {}", program_account.lamports);
}

#[test]
fn test_create_account() {
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

    // new account keys
    let new_keypair =
        solana_sdk::signer::keypair::keypair_from_seed_phrase_and_passphrase("seed", "").unwrap(); // any seed you like

    let reserved_space = 0;
    let rent = client
        .get_minimum_balance_for_rent_exemption(reserved_space)
        .unwrap();
    let create_account_instruction = system_instruction::create_account(
        &wallet.pubkey(), // payer
        &new_keypair.pubkey(),
        rent,
        reserved_space as u64,
        &system_program::ID,
    );

    let recent_blockhash = client
        .get_latest_blockhash()
        .unwrap();
    let transaction: Transaction = Transaction::new_signed_with_payer(
        &[create_account_instruction],
        Some(&wallet.pubkey()),
        &[&wallet, &new_keypair],
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
fn get_client_program_wallet_accounts() -> (
    solana_client::rpc_client::RpcClient,
    Pubkey, /* program */
    Pubkey, /* wallet */
) {
    // program owner
    let program_id = env::var("PROGRAM_ID")
        .unwrap_or("EqWRXakMW7rJvT4dMUd1uUdNyKbbvEP48kK9NCevfAJ4".to_string());

    let wallet_address = env::var("WALLET_ADDR")
        .unwrap_or("E617pwHkquBHUPAqcThYYmw1Wbhcwy1vq8V4vnPpAfMY".to_string());

    let wallet: Pubkey = wallet_address
        .parse::<Pubkey>()
        .expect("Invalid wallet address value (check ${WALLET_ADDR})");

    let program: Pubkey = program_id
        .parse::<Pubkey>()
        .expect("Invalid program id (check ${PROGRAM_ID})");

    (get_rpc_client(), program, wallet)
}
