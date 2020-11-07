//! Instruction types

use crate::error::UniswapOracleError;
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
pub enum UniswapOracleInstruction {
    /// Initializes a new Uniswap oracle account.
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
        /// Moebius program's ID
        moebius_program_id: Pubkey,
    },
    /// Updates the state of the Uniswap oracle.
    UpdateState {
        /// Address of the first token in the Uniswap pair.
        token0: [u8; 20],
        /// Amount of the first token.
        amount0: [u8; 32],
        /// Address of the second token in the Uniswap pair.
        token1: [u8; 20],
        /// Amount of the second token.
        amount1: [u8; 32],
    },
}

impl UniswapOracleInstruction {
    /// Packs a [UniswapOracleInstruction](enum.UniswapOracleInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::Initialize {
                ref moebius_program_id,
            } => {
                buf.push(0);
                buf.extend_from_slice(moebius_program_id.as_ref());
            }
            Self::UpdateState {
                token0,
                amount0,
                token1,
                amount1,
            } => {
                buf.push(1);

                buf.extend_from_slice(&[0u8; 12]);
                buf.extend_from_slice(&token0[..]);
                buf.extend_from_slice(&amount0[..]);

                buf.extend_from_slice(&[0u8; 12]);
                buf.extend_from_slice(&token1[..]);
                buf.extend_from_slice(&amount1[..]);
            }
        }
        buf
    }

    /// Unpacks a byte buffer into a [UniswapOracleInstruction](enum.UniswapOracleInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use UniswapOracleError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (moebius_program_id, _rest) = Self::unpack_pubkey(rest)?;
                Self::Initialize { moebius_program_id }
            }
            1 => {
                let (token0_slice, rest) = rest.split_at(32);
                let (amount0_slice, rest) = rest.split_at(32);
                let (token1_slice, rest) = rest.split_at(32);
                let (amount1_slice, _rest) = rest.split_at(32);
                let mut token0 = [0u8; 20];
                let mut amount0 = [0u8; 32];
                let mut token1 = [0u8; 20];
                let mut amount1 = [0u8; 32];
                token0.copy_from_slice(&token0_slice[12..]);
                amount0.copy_from_slice(&amount0_slice[..]);
                token1.copy_from_slice(&token1_slice[12..]);
                amount1.copy_from_slice(&amount1_slice[..]);
                Self::UpdateState {
                    token0,
                    amount0,
                    token1,
                    amount1,
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
            Err(UniswapOracleError::InvalidInstruction.into())
        }
    }
}

/// Creates a `Initialize` instruction.
pub fn initialize(
    program_id: &Pubkey,
    uniswap_oracle_account_id: &Pubkey,
    moebius_program_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = UniswapOracleInstruction::Initialize {
        moebius_program_id: *moebius_program_id,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*uniswap_oracle_account_id, false),
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
    uniswap_oracle_account_id: &Pubkey,
    authority: &Pubkey,
    token0: [u8; 20],
    amount0: [u8; 32],
    token1: [u8; 20],
    amount1: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let data = UniswapOracleInstruction::UpdateState {
        token0,
        amount0,
        token1,
        amount1,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*uniswap_oracle_account_id, false),
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
        let check = UniswapOracleInstruction::Initialize {
            moebius_program_id: Pubkey::new(&[2u8; 32]),
        };
        let packed = check.pack();
        let mut expect = vec![0u8]; // Initialize tag.
        expect.extend_from_slice(&[2u8; 32]);
        assert_eq!(packed, expect);
        let unpacked = UniswapOracleInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }

    #[test]
    fn test_update_state_packing() {
        let rand_token0 = rand_bytes(20usize);
        let rand_amount0 = rand_bytes(32usize);
        let rand_token1 = rand_bytes(20usize);
        let rand_amount1 = rand_bytes(32usize);
        let mut token0 = [0u8; 20];
        let mut amount0 = [0u8; 32];
        let mut token1 = [0u8; 20];
        let mut amount1 = [0u8; 32];
        token0.copy_from_slice(rand_token0.as_slice());
        amount0.copy_from_slice(rand_amount0.as_slice());
        token1.copy_from_slice(rand_token1.as_slice());
        amount1.copy_from_slice(rand_amount1.as_slice());
        let check = UniswapOracleInstruction::UpdateState {
            token0,
            amount0,
            token1,
            amount1,
        };
        let packed = check.pack();
        let mut expect = vec![1u8]; // Initialize tag.
        expect.extend_from_slice(&[0u8; 12]);
        expect.extend_from_slice(rand_token0.as_slice());
        expect.extend_from_slice(rand_amount0.as_slice());
        expect.extend_from_slice(&[0u8; 12]);
        expect.extend_from_slice(rand_token1.as_slice());
        expect.extend_from_slice(rand_amount1.as_slice());
        assert_eq!(packed, expect);
        let unpacked = UniswapOracleInstruction::unpack(&expect).unwrap();
        assert_eq!(unpacked, check);
    }
}
