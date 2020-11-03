//! Instruction types

use crate::error::SimpleProgramError;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::mem::size_of;

/// Instructions supported by the Simple program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum SimpleProgramInstruction {
    /// Initializes the Simple program.
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
    /// Updates the state of Simple program.
    UpdateState {
        /// Bytes32 field in Simple program's state.
        val_bytes32: [u8; 32],
        /// Address field in Simple program's state.
        val_address: [u8; 20],
        /// Uint256 field in Simple program's state.
        val_uint256: [u8; 32],
    },
}

impl SimpleProgramInstruction {
    /// Packs a [SimpleProgramInstruction](enum.SimpleProgramInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize { ref authority } => {
                buf.push(0);
                buf.extend_from_slice(authority.as_ref());
            }
            Self::UpdateState {
                val_bytes32,
                val_address,
                val_uint256,
            } => {
                buf.push(1);
                buf.extend_from_slice(&val_bytes32[..]);
                buf.extend_from_slice(&[0u8; 12]);
                buf.extend_from_slice(&val_address[..]);
                buf.extend_from_slice(&val_uint256[..]);
            }
        }
        buf
    }

    /// Unpacks a byte buffer into a [SimpleProgramInstruction](enum.SimpleProgramInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use SimpleProgramError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (authority, _rest) = Self::unpack_pubkey(rest)?;
                Self::Initialize { authority }
            }
            1 => {
                let (val_bytes32_slice, rest) = rest.split_at(32);
                let (val_address_slice, rest) = rest.split_at(32);
                let (val_uint256_slice, _rest) = rest.split_at(32);
                let mut val_bytes32 = [0u8; 32];
                let mut val_address = [0u8; 20];
                let mut val_uint256 = [0u8; 32];
                val_bytes32.copy_from_slice(&val_bytes32_slice[..]);
                val_address.copy_from_slice(&val_address_slice[12..]);
                val_uint256.copy_from_slice(&val_uint256_slice[..]);
                Self::UpdateState {
                    val_bytes32,
                    val_address,
                    val_uint256,
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
            Err(SimpleProgramError::InvalidInstruction.into())
        }
    }
}

/// Creates a `Initialize` instruction.
pub fn initialize(
    program_id: &Pubkey,
    simple_program_account_id: &Pubkey,
    authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = SimpleProgramInstruction::Initialize {
        authority: *authority,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*simple_program_account_id, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Creates an `UpdateState` instruction.
pub fn update_state(
    program_id: &Pubkey,
    simple_program_account_id: &Pubkey,
    authority: &Pubkey,
    val_bytes32: [u8; 32],
    val_address: [u8; 20],
    val_uint256: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let data = SimpleProgramInstruction::UpdateState {
        val_bytes32,
        val_address,
        val_uint256,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*simple_program_account_id, false),
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
    use rand::RngCore;

    fn rand_bytes(n: usize) -> Vec<u8> {
        let mut output = vec![0u8; n];
        rand::thread_rng().fill_bytes(output.as_mut_slice());
        output
    }

    #[test]
    fn test_initialize_packing() {
        let check = SimpleProgramInstruction::Initialize {
            authority: Pubkey::new(&[2u8; 32]),
        };
        let packed = check.pack();
        let mut expect = vec![0u8]; // Initialize tag.
        expect.extend_from_slice(&[2u8; 32]);
        assert_eq!(packed, expect);
        let unpacked = SimpleProgramInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }

    #[test]
    fn test_update_state_packing() {
        let rand_val_bytes32 = rand_bytes(32usize);
        let rand_val_address = rand_bytes(20usize);
        let rand_val_uint256 = rand_bytes(32usize);
        let mut val_bytes32 = [0u8; 32];
        let mut val_address = [0u8; 20];
        let mut val_uint256 = [0u8; 32];
        val_bytes32.copy_from_slice(rand_val_bytes32.as_slice());
        val_address.copy_from_slice(rand_val_address.as_slice());
        val_uint256.copy_from_slice(rand_val_uint256.as_slice());
        let check = SimpleProgramInstruction::UpdateState {
            val_bytes32: val_bytes32,
            val_address: val_address,
            val_uint256: val_uint256,
        };
        let packed = check.pack();
        let mut expect = vec![1u8]; // Initialize tag.
        expect.extend_from_slice(rand_val_bytes32.as_slice());
        expect.extend_from_slice(&[0u8; 12]);
        expect.extend_from_slice(rand_val_address.as_slice());
        expect.extend_from_slice(rand_val_uint256.as_slice());
        assert_eq!(packed, expect);
        let unpacked = SimpleProgramInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }
}
