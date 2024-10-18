//! Program state processor
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

use crate::event;

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
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let now = sysvar::clock::Clock::get()
        .ok()
        .unwrap()
        .unix_timestamp as i128;

    log_accounts(accounts);
    if accounts.len() < 1 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let account_info_iter = &mut accounts.iter();
    let account = solana_program::account_info::next_account_info(account_info_iter)?;

    // parse the input event
    let mut data = input;
    let event = event::Event::deserialize(&mut data);
    if event.is_err() {
        // invalid input data
        msg!("Can't deserialize the input: {:?}", event.err().unwrap());
        return Err(ProgramError::InvalidInstructionData);
    }
    let event = event?;

    msg!(
        "Event generated: {:?} | Received by this program in: {} s",
        event.solana_ts_received_at,
        (now - event.solana_ts_received_at),
    );

    let mut account_data =
        AccountData::try_from_slice(&account.data.borrow()).unwrap_or(AccountData::default());

    // track only the latest event in the account data,
    // all events are available from the transactions payload
    // (stored on the blockchain)
    let _ = account_data
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

    // store new account data
    account_data
        .last_file_events
        .insert(event.file_path.clone(), event.clone());
    let mut serialized = Vec::<u8>::new();
    account_data.serialize(&mut serialized)?;
    // account_data.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!(
        "New event: {} {} | TOTAL: {} files",
        event.event_type,
        event.file_path,
        account_data
            .last_file_events
            .len(),
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
