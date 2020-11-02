//! Program state processor

#![cfg(feature = "program")]

use crate::{
    error::SimpleProgramError, instruction::SimpleProgramInstruction, state::SimpleProgram,
};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    info,
    program_error::PrintProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [Instruction](enum.SimpleProgramInstruction.html).
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = SimpleProgramInstruction::unpack(input)?;

        match instruction {
            SimpleProgramInstruction::Initialize { authority } => {
                info!("Instruction: Initialize");
                Self::process_initialize(accounts, authority)
            }
            SimpleProgramInstruction::UpdateState {
                val_bytes32,
                val_address,
                val_uint256,
            } => {
                info!("Instruction: UpdateState");
                Self::process_update_state(accounts, val_bytes32, val_address, val_uint256)
            }
        }
    }

    /// Processes an [Initialize](enum.SimpleProgramInstruction.html) instruction.
    pub fn process_initialize(accounts: &[AccountInfo], authority: Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let simple_program_account_info = next_account_info(account_info_iter)?;
        let simple_program_data_len = simple_program_account_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if simple_program_account_info.data.borrow()[0] != 0u8 {
            return Err(SimpleProgramError::AlreadyInUse.into());
        }

        if !rent.is_exempt(
            simple_program_account_info.lamports(),
            simple_program_data_len,
        ) {
            return Err(SimpleProgramError::NotRentExempt.into());
        }

        let simple_program = SimpleProgram {
            is_initialized: true,
            authority: authority,
            val_bytes32: [0u8; 32],
            val_address: [0u8; 20],
            val_uint256: [0u8; 32],
        };
        SimpleProgram::pack(
            simple_program,
            &mut simple_program_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    /// Processes an [UpdateState](enum.SimpleProgramInstruction.html) instruction.
    pub fn process_update_state(
        accounts: &[AccountInfo],
        val_bytes32: [u8; 32],
        val_address: [u8; 20],
        val_uint256: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let simple_program_account_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;

        let mut state =
            SimpleProgram::unpack_unchecked(&simple_program_account_info.data.borrow())?;

        if authority_info.key != &state.authority {
            return Err(SimpleProgramError::Unauthorized.into());
        }

        state.val_bytes32 = val_bytes32;
        state.val_address = val_address;
        state.val_uint256 = val_uint256;

        SimpleProgram::pack(state, &mut simple_program_account_info.data.borrow_mut())?;

        Ok(())
    }
}

impl PrintProgramError for SimpleProgramError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SimpleProgramError::InvalidInstruction => info!("Error: Invalid instruction"),
            SimpleProgramError::AlreadyInUse => info!("Error: account or token already in use"),
            SimpleProgramError::NotRentExempt => {
                info!("Error: Lamport balance below rent-exempt threshold")
            }
            SimpleProgramError::Unauthorized => {
                info!("Error: Account not authorized to do the transaction")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use rand::RngCore;
    use solana_program::{
        account::Account as SolanaAccount,
        account_info::create_is_signer_account_infos,
        instruction::Instruction,
        program_error::{PrintProgramError, ProgramError},
        sysvar::rent,
    };

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    fn rent_sysvar() -> SolanaAccount {
        rent::create_account(42, &Rent::default())
    }

    fn minimum_balance() -> u64 {
        Rent::default().minimum_balance(SimpleProgram::LEN)
    }

    fn return_simple_program_error_as_program_error() -> ProgramError {
        SimpleProgramError::Unauthorized.into()
    }

    fn pubkey_rand() -> Pubkey {
        Pubkey::new(&rand::random::<[u8; 32]>())
    }

    fn rand_bytes(n: usize) -> Vec<u8> {
        let mut output = vec![0u8; n];
        rand::thread_rng().fill_bytes(output.as_mut_slice());
        output
    }

    #[test]
    fn test_print_error() {
        let error = return_simple_program_error_as_program_error();
        error.print::<SimpleProgramError>();
    }

    #[test]
    #[should_panic(expected = "Custom(3)")]
    fn test_error_unwrap() {
        Err::<(), ProgramError>(return_simple_program_error_as_program_error()).unwrap();
    }

    #[test]
    fn test_initialize() {
        let program_id = pubkey_rand();
        let authority = pubkey_rand();
        let simple_program_account_id = pubkey_rand();
        let mut simple_program_account = SolanaAccount::new(42, SimpleProgram::LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();

        // when the simple_program account is not rent exempt.
        assert_eq!(
            Err(SimpleProgramError::NotRentExempt.into()),
            do_process_instruction(
                initialize(&program_id, &simple_program_account_id, &authority).unwrap(),
                vec![&mut simple_program_account, &mut rent_sysvar]
            )
        );

        simple_program_account.lamports = minimum_balance();

        // create new simple_program account.
        do_process_instruction(
            initialize(&program_id, &simple_program_account_id, &authority).unwrap(),
            vec![&mut simple_program_account, &mut rent_sysvar],
        )
        .unwrap();

        // create twice.
        assert_eq!(
            Err(SimpleProgramError::AlreadyInUse.into()),
            do_process_instruction(
                initialize(&program_id, &simple_program_account_id, &authority).unwrap(),
                vec![&mut simple_program_account, &mut rent_sysvar]
            )
        );

        let simple_program = SimpleProgram::unpack(&simple_program_account.data).unwrap();

        assert_eq!(simple_program.is_initialized, true);
        assert_eq!(simple_program.authority, authority);
    }

    #[test]
    fn test_update_state() {
        let program_id = pubkey_rand();
        let authority_key = pubkey_rand();
        let mut authority = SolanaAccount::default();
        let simple_program_account_id = pubkey_rand();
        let mut simple_program_account =
            SolanaAccount::new(minimum_balance(), SimpleProgram::LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();

        // create new simple_program account.
        do_process_instruction(
            initialize(&program_id, &simple_program_account_id, &authority_key).unwrap(),
            vec![&mut simple_program_account, &mut rent_sysvar],
        )
        .unwrap();

        let rand_val_bytes32 = rand_bytes(32usize);
        let rand_val_address = rand_bytes(20usize);
        let rand_val_uint256 = rand_bytes(32usize);
        let mut new_val_bytes32 = [0u8; 32];
        let mut new_val_address = [0u8; 20];
        let mut new_val_uint256 = [0u8; 32];
        new_val_bytes32.copy_from_slice(rand_val_bytes32.as_slice());
        new_val_address.copy_from_slice(rand_val_address.as_slice());
        new_val_uint256.copy_from_slice(rand_val_uint256.as_slice());
        let not_authority = pubkey_rand();
        assert_eq!(
            Err(SimpleProgramError::Unauthorized.into()),
            do_process_instruction(
                update_state(
                    &program_id,
                    &simple_program_account_id,
                    &not_authority,
                    new_val_bytes32,
                    new_val_address,
                    new_val_uint256,
                )
                .unwrap(),
                vec![&mut simple_program_account, &mut authority],
            )
        );

        assert!(do_process_instruction(
            update_state(
                &program_id,
                &simple_program_account_id,
                &authority_key,
                new_val_bytes32,
                new_val_address,
                new_val_uint256,
            )
            .unwrap(),
            vec![&mut simple_program_account, &mut authority],
        )
        .is_ok(),);

        let new_state = SimpleProgram::unpack(&simple_program_account.data).unwrap();

        assert_eq!(new_state.val_bytes32, new_val_bytes32);
        assert_eq!(new_state.val_address, new_val_address);
        assert_eq!(new_state.val_uint256, new_val_uint256);
    }
}
