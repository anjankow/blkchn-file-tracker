//! Program state processor
use std::{io, thread::panicking};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, sysvar,
    sysvar::Sysvar,
};

use crate::{
    event,
    instruction::{self, EventTrackerInstruction},
};

pub const VAULT_ACCOUNT_SIZE: u64 = 1024;
pub const PDA_SEED_PREFIX: &[u8] = b"vault";

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
struct AccountData {
    last_file_events: std::collections::HashMap<String, event::Event>,
}

impl Default for AccountData {
    fn default() -> Self {
        AccountData {
            last_file_events: std::collections::HashMap::<String, event::Event>::new(),
        }
    }
}

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    // The first account is always the pda owner, the end user's wallet.
    // This user needs to sign each transaction.
    let account_info_iter = &mut accounts.iter();
    let payer = solana_program::account_info::next_account_info(account_info_iter)?;
    if !payer.is_signer || payer.signer_key().is_none() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let instr = EventTrackerInstruction::unpack(input)?;
    match instr {
        EventTrackerInstruction::Initialize(initialize_instruction_data) => {
            process_initialize(program_id, accounts, initialize_instruction_data)
        }
        EventTrackerInstruction::AddEvent(add_event_instruction_data) => {
            process_add_event(program_id, accounts, add_event_instruction_data)
        }
        EventTrackerInstruction::CloseAccount => todo!(),
    }
}

pub fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: instruction::InitializeInstructionData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = solana_program::account_info::next_account_info(account_info_iter)?;
    if !payer.is_writable {
        return Err(ProgramError::Immutable);
    }
    if !payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let pda = solana_program::account_info::next_account_info(account_info_iter)?;
    if !pda.is_writable {
        return Err(ProgramError::Immutable);
    }
    // System program needs to come from the outside
    let system_program = solana_program::account_info::next_account_info(account_info_iter)?;

    // Used to uniquely identify this PDA among others.
    let pda_seed = &[
        /* passed to find_program_address */ PDA_SEED_PREFIX,
        /* passed to find_program_address */ payer.key.as_ref(),
        /* pda_bump_seed calculated by find_program_address
        and expected on account retrieval */
        &[input.pda_bump_seed],
    ];

    // Invoke the system program to create an account while virtually
    // signing with the vault PDA, which is owned by this caller program.
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer.key,
            pda.key,
            input.lamports,
            VAULT_ACCOUNT_SIZE,
            program_id,
        ),
        &[payer.clone(), pda.clone(), system_program.clone()],
        &[pda_seed],
    )
}

pub fn process_add_event(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: instruction::AddEventInstructionData,
) -> ProgramResult {
    // let now = sysvar::clock::Clock::get()
    //     .ok()
    //     .unwrap()
    //     .unix_timestamp as i128;
    let event = input.event;
    // msg!(
    //     "Event generated: {:?} | Received by this program in: {} s",
    //     event.solana_ts_received_at,
    //     (now - event.solana_ts_received_at),
    // );

    // log_accounts(accounts);

    let account_info_iter = &mut accounts.iter();
    let payer = solana_program::account_info::next_account_info(account_info_iter)?;
    if !payer.is_writable {
        return Err(ProgramError::Immutable);
    }
    // Vault is the user's PDA created with Initialize.
    let vault = solana_program::account_info::next_account_info(account_info_iter)?;
    if !vault.is_writable {
        return Err(ProgramError::Immutable);
    }

    let mut vault_data =
        AccountData::try_from_slice(&vault.data.borrow()).unwrap_or(AccountData::default());

    // track only the latest event in the account data,
    // all events are available from the transactions payload
    // (stored on the blockchain)
    let _ = vault_data
        .last_file_events
        .get(&event.file_path)
        .is_some_and(|old_event| {
            msg!(
                "{file_path} | Replacing last file event {old_event} with a new one {new_event}",
                file_path = &event.file_path,
                old_event = old_event.event_type,
                new_event = &event.event_type
            );
            true
        });

    // update the account_data value with the new event
    vault_data
        .last_file_events
        .insert(event.file_path.clone(), event.clone());
    let mut serialized = Vec::<u8>::new();
    vault_data.serialize(&mut serialized)?;

    // check how much space is needed and increase it
    vault.realloc(serialized.len(), false)?;
    // store new account data
    vault.data.borrow_mut()[..].copy_from_slice(&serialized);
    // println!("Value updated, data size: {}", vault.data_len());

    msg!(
        "New event: {} {} | TOTAL: {} files | New size: {}",
        event.event_type,
        event.file_path,
        vault_data
            .last_file_events
            .len(),
        serialized.len(),
    );

    Ok(())
}

fn log_accounts(accounts: &[AccountInfo]) {
    msg!("Accounts num: {}", accounts.len());
    for account_info in accounts.iter() {
        let mut account_log = String::from(format!(
            "account: {:x?} | owner: {:x?} | signer: {:x?} | lamp: {}",
            fmt_pubkey(account_info.key),
            account_info
                .signer_key()
                .map(|k| fmt_pubkey(k))
                .unwrap_or(String::from("NO SIGNER")),
            fmt_pubkey(account_info.owner),
            account_info.lamports()
        ));
        if account_info.data_is_empty() {
            account_log += " | NO DATA | ";
        } else {
            account_log += " |  data existing";
        }
        msg!(&account_log);
    }
}

fn fmt_pubkey(key: &Pubkey) -> String {
    let mut res = key.to_string();
    res.truncate(6);
    res
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        event::EventType,
        solana_program::{
            account_info::IntoAccountInfo, program_error::ProgramError, pubkey::Pubkey,
        },
        solana_sdk::account::{Account, ReadableAccount, WritableAccount},
        std::io::Write,
    };

    #[test]
    fn test_serialize_account_data() {
        let mut account_data = AccountData::default();
        account_data
            .last_file_events
            .insert(
                "path".to_string(),
                event::Event {
                    file_path: "path".to_string(),
                    event_type: EventType::AttributeChanged,
                    solana_ts_received_at: 123,
                    file_info: None,
                },
            );

        let pubkey = Pubkey::new_unique();
        let mut account = Account::default();
        // account.data.reserve(100); // <-- this panics
        account.data.resize(111, 0); // <-- this works
        let account_info = (&pubkey, true, &mut account).into_account_info();

        account_data
            .serialize(&mut &mut account_info.data.borrow_mut()[..])
            .unwrap();
        println!(
            "account_info data len: {}, account data len: {}",
            account_info.data_len(),
            account.data().len()
        );
    }
}
