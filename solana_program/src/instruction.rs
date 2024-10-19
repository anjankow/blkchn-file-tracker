//! Program instructions

use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::mem::size_of,
};

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct AddEventInstructionData {
    pub event: crate::event::Event,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct InitializeInstructionData {
    pub lamports: u64, // to pay for rent of the PDA
    pub pda_bump_seed: u8,
}

// #[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
// pub struct ReallocateInstructionData {
//     data_length: u64,
// }

/// Instructions supported by the program
#[derive(Clone, Debug, PartialEq)]
pub enum EventTrackerInstruction {
    /// Create a PDA for the user.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` User account, PDA owner.
    /// 1. `[]` PDA found with Pubkey::find_program_address for this user.
    Initialize(InitializeInstructionData),

    /// Add new event to the user's PDA
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` User account, PDA owner.
    /// 1. `[writable]` User's PDA
    AddEvent(AddEventInstructionData),

    /// Close the provided PDA account, draining lamports to recipient
    /// account
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[writable, signer]` User account, PDA owner.
    /// 1. `[writable]` User's PDA
    CloseAccount,
    // /// Reallocate additional space in the user's PDA
    // ///
    // /// If the record account already has enough space to hold the specified
    // /// data length, then the instruction does nothing.
    // ///
    // /// Accounts expected by this instruction:
    // ///
    // /// 0. `[writable, signer]` User account, PDA owner.
    // /// 1. `[writable]` User's PDA
    // Reallocate(ReallocateInstructionData),
}

impl EventTrackerInstruction {
    /// Unpacks a byte buffer into a [EventTrackerInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        const U32_BYTES: usize = 4;
        const U64_BYTES: usize = 8;

        let (&tag, mut data) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match tag {
            0 => {
                let instruction_data =
                    InitializeInstructionData::deserialize(&mut data).map_err(|e| {
                        msg!("Failed to deserialize instruction body: {}", e);
                        return ProgramError::InvalidInstructionData;
                    })?;
                Self::Initialize(instruction_data)
            }
            1 => {
                let instruction_data =
                    AddEventInstructionData::deserialize(&mut data).map_err(|e| {
                        msg!("Failed to deserialize instruction body: {}", e);
                        return ProgramError::InvalidInstructionData;
                    })?;
                Self::AddEvent(instruction_data)
            }
            2 => Self::CloseAccount,

            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    /// Packs a [EventTrackerInstruction] into a byte buffer.
    pub fn pack(&self) -> Result<Vec<u8>, borsh::io::Error> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize(data) => {
                buf.push(0);
                data.serialize(&mut buf)?;
            }
            Self::AddEvent(data) => {
                buf.push(1);
                data.serialize(&mut buf)?;
            }
            Self::CloseAccount => buf.push(2),
        };
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_initialize() {
        let instruction = EventTrackerInstruction::Initialize(InitializeInstructionData {
            lamports: 3213,
            pda_bump_seed: 255,
        });

        let packed = instruction.pack().unwrap();
        assert_eq!(0, *packed.get(0).unwrap());
        let unpacked = EventTrackerInstruction::unpack(&packed).unwrap();
        assert_eq!(instruction, unpacked);
    }
}
