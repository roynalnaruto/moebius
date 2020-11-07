//! Program state processor

use crate::{
    error::UniswapOracleError, instruction::UniswapOracleInstruction, state::UniswapOracle,
};
use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    info,
    program_error::{PrintProgramError, ProgramError},
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [Instruction](enum.UniswapOracleInstruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = UniswapOracleInstruction::unpack(input)?;

        match instruction {
            UniswapOracleInstruction::Initialize { moebius_program_id } => {
                info!("Instruction: Initialize");
                Self::process_initialize(program_id, accounts, moebius_program_id)
            }
            UniswapOracleInstruction::UpdateState {
                token0,
                amount0,
                token1,
                amount1,
            } => {
                info!("Instruction: UpdateState");
                Self::process_update_state(accounts, token0, amount0, token1, amount1)
            }
        }
    }

    /// Processes an [Initialize](enum.UniswapOracleInstruction.html) instruction.
    pub fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        moebius_program_id: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let uniswap_oracle_account_info = next_account_info(account_info_iter)?;
        let uniswap_oracle_data_len = uniswap_oracle_account_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        // Ensure that this account has not already been put into use.
        let state = UniswapOracle::unpack_unchecked(&uniswap_oracle_account_info.data.borrow())?;
        if state.is_initialized {
            return Err(UniswapOracleError::AlreadyInUse.into());
        }

        if !rent.is_exempt(
            uniswap_oracle_account_info.lamports(),
            uniswap_oracle_data_len,
        ) {
            return Err(UniswapOracleError::NotRentExempt.into());
        }

        // Calculate the program derived address that will be used as authority from Moebius'
        // program.
        let (authority, _) = Pubkey::find_program_address(
            &[
                &program_id.to_bytes(),
                &uniswap_oracle_account_info.key.to_bytes(),
            ],
            &moebius_program_id,
        );

        // Initialize the state of uniswap oracle's account and write it.
        let state = UniswapOracle {
            is_initialized: true,
            authority,
            token0: [0u8; 20],
            amount0: [0u8; 32],
            token1: [0u8; 20],
            amount1: [0u8; 32],
        };
        UniswapOracle::pack(state, &mut uniswap_oracle_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes an [UpdateState](enum.UniswapOracleInstruction.html) instruction.
    pub fn process_update_state(
        accounts: &[AccountInfo],
        token0: [u8; 20],
        amount0: [u8; 32],
        token1: [u8; 20],
        amount1: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let authority_info = next_account_info(account_info_iter)?;
        let uniswap_oracle_account_info = next_account_info(account_info_iter)?;

        let mut state =
            UniswapOracle::unpack_unchecked(&uniswap_oracle_account_info.data.borrow())?;

        if authority_info.key != &state.authority {
            return Err(UniswapOracleError::Unauthorized.into());
        }
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        state.token0 = token0;
        state.amount0 = amount0;
        state.token1 = token1;
        state.amount1 = amount1;

        UniswapOracle::pack(state, &mut uniswap_oracle_account_info.data.borrow_mut())?;

        Ok(())
    }
}

impl PrintProgramError for UniswapOracleError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            UniswapOracleError::InvalidInstruction => info!("Error: Invalid instruction"),
            UniswapOracleError::AlreadyInUse => info!("Error: account or token already in use"),
            UniswapOracleError::NotRentExempt => {
                info!("Error: Lamport balance below rent-exempt threshold")
            }
            UniswapOracleError::Unauthorized => {
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
        instruction::Instruction,
        program_error::{PrintProgramError, ProgramError},
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

    fn minimum_balance() -> u64 {
        Rent::default().minimum_balance(UniswapOracle::LEN)
    }

    fn return_uniswap_oracle_error_as_program_error() -> ProgramError {
        UniswapOracleError::Unauthorized.into()
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
        let error = return_uniswap_oracle_error_as_program_error();
        error.print::<UniswapOracleError>();
    }

    #[test]
    #[should_panic(expected = "Custom(3)")]
    fn test_error_unwrap() {
        Err::<(), ProgramError>(return_uniswap_oracle_error_as_program_error()).unwrap();
    }

    #[test]
    fn test_initialize() {
        let program_id = pubkey_rand();
        let uniswap_oracle_account_id = pubkey_rand();
        let mut uniswap_oracle_account = SolanaAccount::new(42, UniswapOracle::LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();
        let moebius_program_id = pubkey_rand();

        // when the uniswap_oracle account is not rent exempt.
        assert_eq!(
            Err(UniswapOracleError::NotRentExempt.into()),
            do_process_instruction(
                initialize(&program_id, &uniswap_oracle_account_id, &moebius_program_id).unwrap(),
                vec![&mut uniswap_oracle_account, &mut rent_sysvar],
            )
        );

        uniswap_oracle_account.lamports = minimum_balance();

        // create new uniswap_oracle account.
        do_process_instruction(
            initialize(&program_id, &uniswap_oracle_account_id, &moebius_program_id).unwrap(),
            vec![&mut uniswap_oracle_account, &mut rent_sysvar],
        )
        .unwrap();

        // create twice.
        assert_eq!(
            Err(UniswapOracleError::AlreadyInUse.into()),
            do_process_instruction(
                initialize(&program_id, &uniswap_oracle_account_id, &moebius_program_id).unwrap(),
                vec![&mut uniswap_oracle_account, &mut rent_sysvar],
            )
        );

        let uniswap_oracle = UniswapOracle::unpack(&uniswap_oracle_account.data).unwrap();

        let (expected_authority, _) = Pubkey::find_program_address(
            &[
                &program_id.to_bytes(),
                &uniswap_oracle_account_id.to_bytes(),
            ],
            &moebius_program_id,
        );

        assert_eq!(uniswap_oracle.is_initialized, true);
        assert_eq!(uniswap_oracle.authority, expected_authority);
    }

    #[test]
    fn test_update_state() {
        let program_id = pubkey_rand();
        let uniswap_oracle_account_id = pubkey_rand();
        let mut uniswap_oracle_account =
            SolanaAccount::new(minimum_balance(), UniswapOracle::LEN, &program_id);
        let mut rent_sysvar = rent_sysvar();
        let moebius_program_id = pubkey_rand();
        let (authority_key, _) = Pubkey::find_program_address(
            &[
                &program_id.to_bytes(),
                &uniswap_oracle_account_id.to_bytes(),
            ],
            &moebius_program_id,
        );
        let mut authority = SolanaAccount::default();

        // create new uniswap_oracle account.
        do_process_instruction(
            initialize(&program_id, &uniswap_oracle_account_id, &moebius_program_id).unwrap(),
            vec![&mut uniswap_oracle_account, &mut rent_sysvar],
        )
        .unwrap();

        let rand_token0 = rand_bytes(20usize);
        let rand_amount0 = rand_bytes(32usize);
        let rand_token1 = rand_bytes(20usize);
        let rand_amount1 = rand_bytes(32usize);
        let mut new_token0 = [0u8; 20];
        let mut new_amount0 = [0u8; 32];
        let mut new_token1 = [0u8; 20];
        let mut new_amount1 = [0u8; 32];
        new_token1.copy_from_slice(rand_token1.as_slice());
        new_amount0.copy_from_slice(rand_amount0.as_slice());
        new_token0.copy_from_slice(rand_token0.as_slice());
        new_amount1.copy_from_slice(rand_amount1.as_slice());
        let not_authority = pubkey_rand();
        assert_eq!(
            Err(UniswapOracleError::Unauthorized.into()),
            do_process_instruction(
                update_state(
                    &program_id,
                    &uniswap_oracle_account_id,
                    &not_authority,
                    new_token0,
                    new_amount0,
                    new_token1,
                    new_amount1,
                )
                .unwrap(),
                vec![&mut authority, &mut uniswap_oracle_account],
            )
        );

        assert!(do_process_instruction(
            update_state(
                &program_id,
                &uniswap_oracle_account_id,
                &authority_key,
                new_token0,
                new_amount0,
                new_token1,
                new_amount1,
            )
            .unwrap(),
            vec![&mut authority, &mut uniswap_oracle_account],
        )
        .is_ok());

        let new_state = UniswapOracle::unpack(&uniswap_oracle_account.data).unwrap();

        assert_eq!(new_state.token0, new_token0);
        assert_eq!(new_state.amount0, new_amount0);
        assert_eq!(new_state.token1, new_token1);
        assert_eq!(new_state.amount1, new_amount1);
    }
}
