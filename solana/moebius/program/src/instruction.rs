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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_packing() {
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
}
