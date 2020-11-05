//! Program state processor

use crate::{error::MoebiusError, instruction::MoebiusInstruction, state::Moebius};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    info,
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
    program_error::{PrintProgramError, ProgramError},
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
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
            MoebiusInstruction::UpdateData { data } => {
                info!("Instruction: UpdateData");
                Self::process_update_data(program_id, accounts, data)
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

    /// Processes an [UpdateData](enum.MoebiusInstruction.html) instruction.
    pub fn process_update_data(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: Vec<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let moebius_account_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let caller_account_info = next_account_info(account_info_iter)?;
        let target_program_account_info = next_account_info(account_info_iter)?;
        let target_account_account_info = next_account_info(account_info_iter)?;

        // Unpack moebius' state and verify that moebius authority is in fact the account that has
        // submitted this signed transaction.
        let state = Moebius::unpack_unchecked(&moebius_account_info.data.borrow())?;
        if authority_info.key != &state.authority {
            return Err(MoebiusError::Unauthorized.into());
        }
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Make sure that the target program account is in fact an executable program, and that it
        // owns the target account whose state must be updated.
        if !target_program_account_info.executable {
            return Err(MoebiusError::TargetProgramNotExecutable.into());
        }
        if *target_program_account_info.key != *target_account_account_info.owner {
            return Err(MoebiusError::TargetOwnershipMismatch.into());
        }

        // Find the derived program address that should be the authority of the target program.
        let (caller_address, bump_seed) = Pubkey::find_program_address(
            &[
                &target_program_account_info.key.to_bytes(),
                &target_account_account_info.key.to_bytes(),
            ],
            &program_id,
        );
        if caller_address != *caller_account_info.key {
            return Err(MoebiusError::DerivedAccountMismatch.into());
        }

        // Construct the seeds for the above derived program address.
        let caller_account_signer_seeds: &[&[_]] = &[
            &target_program_account_info.key.to_bytes(),
            &target_account_account_info.key.to_bytes(),
            &[bump_seed],
        ];

        // Construct the instruction to be invoked in the target program, that will be signed by
        // Moebius program.
        let target_update_instruction = Instruction {
            program_id: *target_program_account_info.key,
            accounts: vec![
                AccountMeta::new(*caller_account_info.key, true),
                AccountMeta::new(*target_account_account_info.key, false),
            ],
            data,
        };

        // Invoke the instruction in the target program. An authorised invocation should update
        // the target account's state via the target program.
        invoke_signed(
            &target_update_instruction,
            &[
                caller_account_info.clone(),
                target_account_account_info.clone(),
            ],
            &[caller_account_signer_seeds],
        )?;

        Ok(())
    }
}

impl PrintProgramError for MoebiusError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            MoebiusError::InvalidInstruction => info!("Error: Invalid instruction"),
            MoebiusError::AlreadyInUse => info!("Error: account or token already in use"),
            MoebiusError::NotRentExempt => {
                info!("Error: Lamport balance below rent-exempt threshold")
            }
            MoebiusError::Unauthorized => {
                info!("Error: Account not authorized to do the transaction")
            }
            MoebiusError::DerivedAccountMismatch => {
                info!("Error: The derived program account does not match the expected account")
            }
            MoebiusError::TargetProgramNotExecutable => {
                info!("Error: The target program account is not an executable")
            }
            MoebiusError::TargetOwnershipMismatch => {
                info!("Error: The target account is not owned by the target program account")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use solana_program::{
        instruction::Instruction,
        program_error::{PrintProgramError, ProgramError},
        sysvar::rent,
    };
    use solana_sdk::account::{
        create_account, create_is_signer_account_infos, Account as SolanaAccount,
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
        create_account(&Rent::default(), 42)
    }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Moebius::LEN)
    }

    fn return_moebius_error_as_program_error() -> ProgramError {
        MoebiusError::Unauthorized.into()
    }

    fn pubkey_rand() -> Pubkey {
        Pubkey::new(&rand::random::<[u8; 32]>())
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

    #[test]
    fn test_initialize() {
        let program_id = pubkey_rand();
        let authority = pubkey_rand();
        let moebius_account_id = pubkey_rand();
        let mut moebius_account = SolanaAccount::new(42, Moebius::LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();

        // when the moebius account is not rent exempt.
        assert_eq!(
            Err(MoebiusError::NotRentExempt.into()),
            do_process_instruction(
                initialize(&program_id, &moebius_account_id, &authority).unwrap(),
                vec![&mut moebius_account, &mut rent_sysvar]
            )
        );

        moebius_account.lamports = mint_minimum_balance();

        // create new moebius account.
        do_process_instruction(
            initialize(&program_id, &moebius_account_id, &authority).unwrap(),
            vec![&mut moebius_account, &mut rent_sysvar],
        )
        .unwrap();

        // create twice.
        assert_eq!(
            Err(MoebiusError::AlreadyInUse.into()),
            do_process_instruction(
                initialize(&program_id, &moebius_account_id, &authority).unwrap(),
                vec![&mut moebius_account, &mut rent_sysvar]
            )
        );

        let moebius = Moebius::unpack(&moebius_account.data).unwrap();

        assert_eq!(moebius.is_initialized, true);
        assert_eq!(moebius.authority, authority);
    }

    #[test]
    #[ignore]
    fn test_update_data() {
        todo!();
    }
}
