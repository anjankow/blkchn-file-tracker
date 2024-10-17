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
    system_instruction,
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
    log_accounts(accounts);

    let start = time::OffsetDateTime::now_utc();

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
        return Err(ProgramError::InvalidInstructionData);
    }
    let event = event?;

    msg!(
        "Event generated: {:?} | Received by this program in: {} ns",
        event
            .received_at
            .0
            .time()
            .as_hms_nano(),
        (start.unix_timestamp_nanos()
            - event
                .received_at
                .0
                .unix_timestamp_nanos()),
    );

    let mut account_data =
        AccountData::try_from_slice(&account.data.borrow()).unwrap_or(AccountData::default());
    msg!("Borrowed account data");

    // track only the latest event in the account data,
    // all events are available from the transactions payload
    // (stored on the blockchain)
    let _ = account_data
        .last_file_events
        .get(&event.file_name)
        .is_some_and(|old_event| {
            msg!(
                "{file_name} | Replacing last file event {old_event} with a new one {new_event}",
                file_name = &event.file_name,
                old_event = old_event.event_type,
                new_event = &event.event_type
            );
            true
        });

    // store new account data
    account_data
        .last_file_events
        .insert(event.file_name.clone(), event.clone());
    account_data.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!(
        "New event: {} {} | TOTAL: {} files",
        event.event_type,
        event.file_name,
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
        solana_program::{
            account_info::IntoAccountInfo, program_error::ProgramError, pubkey::Pubkey,
        },
        solana_sdk::account::Account,
    };

    #[test]
    fn test_utf8_memo() {
        let program_id = Pubkey::new_from_array([0; 32]);

        let string = b"letters and such";
        assert_eq!(Ok(()), process_instruction(&program_id, &[], string));

        let emoji = "🐆".as_bytes();
        let bytes = [0xF0, 0x9F, 0x90, 0x86];
        assert_eq!(emoji, bytes);
        assert_eq!(Ok(()), process_instruction(&program_id, &[], emoji));

        let mut bad_utf8 = bytes;
        bad_utf8[3] = 0xFF; // Invalid UTF-8 byte
        assert_eq!(
            Err(ProgramError::InvalidInstructionData),
            process_instruction(&program_id, &[], &bad_utf8)
        );
    }

    #[test]
    fn test_signers() {
        let program_id = Pubkey::new_from_array([0; 32]);
        let memo = "🐆".as_bytes();

        let pubkey0 = Pubkey::new_unique();
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();
        let mut account0 = Account::default();
        let mut account1 = Account::default();
        let mut account2 = Account::default();

        let signed_account_infos = vec![
            (&pubkey0, true, &mut account0).into_account_info(),
            (&pubkey1, true, &mut account1).into_account_info(),
            (&pubkey2, true, &mut account2).into_account_info(),
        ];
        assert_eq!(
            Ok(()),
            process_instruction(&program_id, &signed_account_infos, memo)
        );

        assert_eq!(Ok(()), process_instruction(&program_id, &[], memo));

        let unsigned_account_infos = vec![
            (&pubkey0, false, &mut account0).into_account_info(),
            (&pubkey1, false, &mut account1).into_account_info(),
            (&pubkey2, false, &mut account2).into_account_info(),
        ];
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            process_instruction(&program_id, &unsigned_account_infos, memo)
        );

        let partially_signed_account_infos = vec![
            (&pubkey0, true, &mut account0).into_account_info(),
            (&pubkey1, false, &mut account1).into_account_info(),
            (&pubkey2, true, &mut account2).into_account_info(),
        ];
        assert_eq!(
            Err(ProgramError::MissingRequiredSignature),
            process_instruction(&program_id, &partially_signed_account_infos, memo)
        );
    }
}
