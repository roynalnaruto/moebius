//! Instruction types

use crate::error::MoebiusError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::mem::size_of;

/// Instructions supported by the Moebius program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum MoebiusInstruction {
    /// Initializes the Moebius program.
    ///
    /// The `Initialize` instruction requires no signers and MUST be included within the same
    /// Transaction as the system program's `CreateInstruction` that creates the account being
    /// initialized. Otherwise another party can acquire ownership of the uninitialized account.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The program to initialize.
    ///   1. `[]` Rent sysvar
    ///
    Initialize {
        /// The authority that can transport arbitrary data over Moebius.
        authority: Pubkey,
    },
    /// Represents the instruction to update an account's state via a program.
    UpdateData {
        /// Instruction data to update state.
        data: Vec<u8>,
    },
}

impl MoebiusInstruction {
    /// Packs a [MoebiusInstruction](enum.MoebiusInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize { ref authority } => {
                buf.push(0);
                buf.extend_from_slice(authority.as_ref());
            }
            Self::UpdateData { data } => {
                buf.push(1);
                buf.extend_from_slice(&(data.len() as u64).to_le_bytes());
                buf.extend_from_slice(data.as_slice());
            }
        }
        buf
    }

    /// Unpacks a byte buffer into a [MoebiusInstruction](enum.MoebiusInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use MoebiusError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (authority, _rest) = Self::unpack_pubkey(rest)?;
                Self::Initialize { authority }
            }
            1 => {
                let (len, rest) = rest.split_at(8);
                let mut data_len_le_bytes = [0u8; 8];
                data_len_le_bytes.copy_from_slice(len);
                let data_len = u64::from_le_bytes(data_len_le_bytes);
                let (data, _rest) = rest.split_at(data_len as usize);
                Self::UpdateData {
                    data: data.to_vec(),
                }
            }

            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok((pk, rest))
        } else {
            Err(MoebiusError::InvalidInstruction.into())
        }
    }
}

/// Creates a `Initialize` instruction.
pub fn initialize(
    program_id: &Pubkey,
    moebius_account_id: &Pubkey,
    authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = MoebiusInstruction::Initialize {
        authority: *authority,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*moebius_account_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates a `UpdateData` instruction.
pub fn update_data(
    program_id: &Pubkey,
    moebius_account_id: &Pubkey,
    authority_id: &Pubkey,
    caller_account_id: &Pubkey,
    target_program_id: &Pubkey,
    target_account_id: &Pubkey,
    data: Vec<u8>,
) -> Result<Instruction, ProgramError> {
    let instruction_data = MoebiusInstruction::UpdateData { data }.pack();

    let accounts = vec![
        AccountMeta::new(*moebius_account_id, false),
        AccountMeta::new(*authority_id, true), // signer of the transaction.
        AccountMeta::new(*caller_account_id, false),
        AccountMeta::new(*target_program_id, false),
        AccountMeta::new(*target_account_id, false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data: instruction_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_packing() {
        let check = MoebiusInstruction::Initialize {
            authority: Pubkey::new(&[2u8; 32]),
        };
        let packed = check.pack();
        let mut expect = vec![0u8]; // Initialize tag.
        expect.extend_from_slice(&[2u8; 32]);
        assert_eq!(packed, expect);
        let unpacked = MoebiusInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }

    #[test]
    fn test_update_data_packing() {
        let check = MoebiusInstruction::UpdateData {
            data: [5u8; 23].to_vec(),
        };
        let packed = check.pack();
        let mut expect = vec![1u8]; // UpdateData tag.
        expect.extend_from_slice(&(23u64.to_le_bytes()));
        expect.extend_from_slice(&[5u8; 23]);
        assert_eq!(packed, expect);
        let unpacked = MoebiusInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }
}
