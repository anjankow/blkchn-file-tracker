pub mod instruction;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use crate::event::Event;

const VAULT_ACCOUNT_SIZE: u64 = 1024;
const PDA_SEED_PREFIX: &[u8] = b"vault";

pub struct SolanaClient {
    program: Pubkey,
    wallet: Keypair,
    pda: Option<Pubkey>,
    url: String,
    rpc_client: solana_client::rpc_client::RpcClient,
}

impl SolanaClient {
    pub fn new(url: &str, program: Pubkey, wallet: Keypair) -> SolanaClient {
        SolanaClient {
            program: program,
            wallet: wallet,
            url: url.to_string(),
            rpc_client: solana_client::rpc_client::RpcClient::new(&url),
            pda: None,
        }
    }

    pub fn process_events(
        &self,
        rx: std::sync::mpsc::Receiver<Event>,
    ) -> Result<(), crate::error::Error> {
        if self.pda.is_none() {
            return Err(crate::error::Error::new(
                "PDA has to be initialized for this call",
            ));
        }

        for event in rx {
            println!("Consumer received an event: {}", event);

            if let Err(err) = self.process_event(event) {
                println!("Failed to process the event: {}", err);
            }
        }
        Ok(())
    }

    fn process_event(&self, mut event: Event) -> Result<(), crate::error::Error> {
        // todo: add cache to call not more often than every second
        let ts = self.get_solana_unix_timestamp();
        if let Ok(ts_ok) = ts {
            event.solana_ts_received_at = ts_ok as i128;
        } else {
            event.solana_ts_received_at = -1;
        }

        // accounts needed by the transaction
        let accounts = [
            AccountMeta::new(self.wallet.pubkey(), true),
            AccountMeta::new(self.pda.unwrap(), false),
            // no CPIs, no more accounts needed
        ]
        .to_vec();

        //// prepare instruction

        let instr_data =
            instruction::EventTrackerInstruction::AddEvent(instruction::AddEventInstructionData {
                event: event,
            })
            .pack()?;

        let instruction = Instruction::new_with_bytes(self.program, &instr_data, accounts);

        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.wallet.pubkey()),
            &[&self.wallet],
            blockhash,
        );

        println!("Sending to RPC client");
        let client_signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;
        println!("Client signature: {}", client_signature.to_string());

        Ok(())
    }

    pub fn init_account(&mut self) -> Result<(), crate::error::Error> {
        // Get the amount of lamports needed to pay for the vault's rent
        let vault_account_size = usize::try_from(VAULT_ACCOUNT_SIZE)
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(vault_account_size)
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;

        let wallet_pubkey = self.wallet.pubkey();
        // Derive the PDA from the payer account, a string representing the unique
        // purpose of the account ("vault"), and the address of our on-chain program.
        let seeds = &[PDA_SEED_PREFIX, wallet_pubkey.as_ref()];
        let (pda_pubkey, pda_bump_seed) = Pubkey::find_program_address(seeds, &self.program);

        // The on-chain program's instruction data, imported from that program's crate.
        let instr_data = instruction::EventTrackerInstruction::Initialize(
            instruction::InitializeInstructionData {
                lamports,
                pda_bump_seed,
            },
        )
        .pack()?;

        // The accounts required by both our on-chain program and the system program's
        // `create_account` instruction, including the vault's address.
        let accounts = vec![
            AccountMeta::new(self.wallet.pubkey(), true /* is_signer */),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Create the instruction by serializing our instruction data via borsh
        let instruction = Instruction::new_with_bytes(self.program.clone(), &instr_data, accounts);

        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.wallet.pubkey()),
            &[&self.wallet],
            blockhash,
        );

        let signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction);
        if let Err(e) = signature {
            // If the account already exists, instead of TransactionError::AccountInUse
            // the retuned error is custom program error: 0x0.
            // Therefore we will just parse the error string searching for
            // `already in use` term instead of checking the error type.
            if e.to_string()
                .contains("already in use")
            {
                println!("Wallet's PDA already exists");
                self.pda = Some(pda_pubkey);
                return Ok(());
            } else {
                return Err(crate::error::Error::new(&e.to_string()));
            }
        }

        println!("PDA created, transaction signature: {}", signature.unwrap());

        self.pda = Some(pda_pubkey);
        Ok(())
    }

    fn get_solana_unix_timestamp(&self) -> Result<i64, crate::error::Error> {
        let sysvar_clock_address = "SysvarC1ock11111111111111111111111111111111";

        let recv_body = self
            .get_account_info(&sysvar_clock_address)
            .map_err(|e| crate::error::Error::new(&e.to_string()))?;

        let res = recv_body
            .get("result")
            .and_then(|res| res.get("value"))
            .and_then(|res| res.get("data"))
            .and_then(|res| res.get("parsed"))
            .and_then(|res| res.get("info"))
            .and_then(|res| res.get("unixTimestamp"));

        res.map(|r| r.as_i64().unwrap())
            .ok_or(crate::error::Error::new(
                "unixTimestamp is missing in the response, incorrectly parsed?",
            ))
    }

    fn get_account_info(
        &self,
        account_key: &str,
    ) -> Result<std::collections::HashMap<String, serde_json::Value>, ureq::Error> {
        let req_body = ureq::json!({
        "jsonrpc": "2.0",
        "id": 1,
            "method": "getAccountInfo",
            "params": [
                account_key,
                {
                    "encoding": "jsonParsed",
                },
            ],
        });

        // https://solana.com/docs/rpc/http/getaccountinfo
        ureq::post(&self.url)
            .send_json(&req_body)?
            .into_json()
            .map_err(|e| e.into())
    }
}
