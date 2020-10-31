//! Program state processor

#![cfg(feature = "program")]

use crate::{
    error::MoebiusError,
    instruction::{is_valid_signer_index, MoebiusInstruction},
    state::Moebius,
};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    info,
    program_error::{PrintProgramError, ProgramError},
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{instructions, rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [Instruction](enum.MoebiusInstruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = MoebiusInstruction::unpack(input)?;

        match instruction {
            MoebiusInstruction::Initialize { authority } => {
                info!("Instruction: Initialize");
                Self::process_initialize(accounts, authority)
            }
        }
    }

    /// Processes an [Initialize](enum.MoebiusInstruction.html) instruction.
    pub fn process_initialize(accounts: &[AccountInfo], authority: Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let moebius_account_info = next_account_info(account_info_iter)?;
        let moebius_data_len = moebius_account_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if moebius_account_info.data.borrow()[0] != 0u8 {
            return Err(MoebiusError::AlreadyInUse.into());
        }

        if !rent.is_exempt(moebius_account_info.lamports(), moebius_data_len) {
            return Err(MoebiusError::NotRentExempt.into());
        }

        let moebius = Moebius {
            is_initialized: true,
            authority: authority,
        };
        Moebius::pack(moebius, &mut moebius_account_info.data.borrow_mut())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use solana_program::{
        account::Account as SolanaAccount, account_info::create_is_signer_account_infos,
        clock::Epoch, instruction::Instruction, sysvar::rent,
    };

    fn return_moebius_error_as_program_error() -> ProgramError {
        MoebiusError::Unauthorized.into()
    }

    #[test]
    fn test_print_error() {
        let error = return_moebius_error_as_program_error();
        error.print::<MoebiusError>();
    }

    #[test]
    #[should_panic(expected = "Custom(3)")]
    fn test_error_unwrap() {
        Err::<(), ProgramError>(return_moebius_error_as_program_error()).unwrap();
    }
}
