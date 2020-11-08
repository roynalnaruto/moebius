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
    program::{invoke, invoke_signed},
    program_error::{PrintProgramError, ProgramError},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [Instruction](enum.UniswapOracleInstruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = UniswapOracleInstruction::unpack(input)?;

        match instruction {
            UniswapOracleInstruction::Initialize {
                moebius_program_id,
                token0,
                decimal0,
                token1,
                decimal1,
            } => {
                info!("Instruction: Initialize");
                Self::process_initialize(
                    program_id,
                    accounts,
                    moebius_program_id,
                    token0,
                    decimal0,
                    token1,
                    decimal1,
                )
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
        token0: [u8; 20],
        decimal0: u8,
        token1: [u8; 20],
        decimal1: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_info = next_account_info(account_info_iter)?;
        let uniswap_oracle_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;

        // Calculate the program derived address for this uniswap pair, and proceed only if the
        // account info is for the appropriate pubkey.
        let (uniswap_oracle_account_id, bump_seed) =
            Pubkey::find_program_address(&[&token0[..], &token1[..]], program_id);
        if uniswap_oracle_account_id != *uniswap_oracle_account_info.key {
            return Err(UniswapOracleError::DerivedAccountMismatch.into());
        }
        let create_account_seeds: &[&[_]] = &[&token0[..], &token1[..], &[bump_seed]];

        // Return an error if this account was already initialized
        if let Ok(state) =
            UniswapOracle::unpack_unchecked(&uniswap_oracle_account_info.data.borrow())
        {
            if state.is_initialized {
                return Err(UniswapOracleError::AlreadyInUse.into());
            }
        }

        // Fund the associated token account with the minimum balance to be rent exempt
        let rent = &Rent::from_account_info(rent_sysvar_info)?;
        let required_lamports = rent
            .minimum_balance(UniswapOracle::LEN)
            .max(1)
            .saturating_sub(uniswap_oracle_account_info.lamports());

        // Fund the account for rent.
        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(
                    &payer_info.key,
                    uniswap_oracle_account_info.key,
                    required_lamports,
                ),
                &[
                    payer_info.clone(),
                    uniswap_oracle_account_info.clone(),
                    system_program_info.clone(),
                ],
            )?;
        }

        // Allocate data size for this account.
        invoke_signed(
            &system_instruction::allocate(
                uniswap_oracle_account_info.key,
                UniswapOracle::LEN as u64,
            ),
            &[
                uniswap_oracle_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&create_account_seeds],
        )?;

        // Assign the current program as this account's owner.
        invoke_signed(
            &system_instruction::assign(uniswap_oracle_account_info.key, &program_id),
            &[
                uniswap_oracle_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&create_account_seeds],
        )?;

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
            token0,
            decimal0,
            amount0: [0u8; 32],
            token1,
            decimal1,
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

        // If the token pairs are not appropriate, we throw an error.
        if state.token0 != token0 || state.token1 != token1 {
            return Err(UniswapOracleError::InvalidAccount.into());
        }

        // Unauthorized action if the signed caller is not the authority.
        if authority_info.key != &state.authority {
            return Err(UniswapOracleError::Unauthorized.into());
        }
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Update the state.
        state.amount0 = amount0;
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
            UniswapOracleError::DerivedAccountMismatch => {
                info!("Error: The derived program account does not match the expected account")
            }
            UniswapOracleError::NotRentExempt => {
                info!("Error: Lamport balance below rent-exempt threshold")
            }
            UniswapOracleError::Unauthorized => {
                info!("Error: Account not authorized to do the transaction")
            }
            UniswapOracleError::InvalidAccount => info!("Error: Invalid account cannot be updated"),
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
    #[should_panic(expected = "Custom(5)")]
    fn test_error_unwrap() {
        Err::<(), ProgramError>(return_uniswap_oracle_error_as_program_error()).unwrap();
    }

    #[test]
    fn test_initialize() {
        let program_id = pubkey_rand();
        let rand_token0 = rand_bytes(20usize);
        let rand_token1 = rand_bytes(20usize);
        let mut token0 = [0u8; 20];
        let mut token1 = [0u8; 20];
        let decimal0 = 9u8;
        let decimal1 = 18u8;
        token0.copy_from_slice(rand_token0.as_slice());
        token1.copy_from_slice(rand_token1.as_slice());
        let (uniswap_oracle_account_id, _) =
            Pubkey::find_program_address(&[&token0[..], &token1[..]], &program_id);
        let mut uniswap_oracle_account = SolanaAccount::new(0, UniswapOracle::LEN, &program_id);
        let payer_id = pubkey_rand();
        let mut payer_account = SolanaAccount::default();
        payer_account.lamports = minimum_balance();
        let moebius_program_id = pubkey_rand();
        let mut rent_sysvar = rent_sysvar();
        let mut system_program_info = SolanaAccount::default();

        // create new uniswap_oracle account.
        do_process_instruction(
            initialize(
                &program_id,
                &uniswap_oracle_account_id,
                &moebius_program_id,
                &payer_id,
                token0,
                decimal0,
                token1,
                decimal1,
            )
            .unwrap(),
            vec![
                &mut payer_account,
                &mut uniswap_oracle_account,
                &mut system_program_info,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // create twice.
        assert_eq!(
            Err(UniswapOracleError::AlreadyInUse.into()),
            do_process_instruction(
                initialize(
                    &program_id,
                    &uniswap_oracle_account_id,
                    &moebius_program_id,
                    &payer_id,
                    token0,
                    decimal0,
                    token1,
                    decimal1,
                )
                .unwrap(),
                vec![
                    &mut payer_account,
                    &mut uniswap_oracle_account,
                    &mut system_program_info,
                    &mut rent_sysvar
                ],
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
        assert_eq!(uniswap_oracle.token0, token0);
        assert_eq!(uniswap_oracle.decimal0, decimal0);
        assert_eq!(uniswap_oracle.amount0, [0u8; 32]);
        assert_eq!(uniswap_oracle.token1, token1);
        assert_eq!(uniswap_oracle.decimal1, decimal1);
        assert_eq!(uniswap_oracle.amount1, [0u8; 32]);
    }

    #[test]
    fn test_update_state() {
        let program_id = pubkey_rand();
        let rand_token0 = rand_bytes(20usize);
        let rand_token1 = rand_bytes(20usize);
        let mut token0 = [0u8; 20];
        let mut token1 = [0u8; 20];
        token0.copy_from_slice(rand_token0.as_slice());
        token1.copy_from_slice(rand_token1.as_slice());
        let decimal0 = 18u8;
        let decimal1 = 18u8;
        let (uniswap_oracle_account_id, _) =
            Pubkey::find_program_address(&[&token0[..], &token1[..]], &program_id);
        let mut uniswap_oracle_account = SolanaAccount::new(0, UniswapOracle::LEN, &program_id);
        let moebius_program_id = pubkey_rand();
        let (authority_key, _) = Pubkey::find_program_address(
            &[
                &program_id.to_bytes(),
                &uniswap_oracle_account_id.to_bytes(),
            ],
            &moebius_program_id,
        );
        let mut authority = SolanaAccount::default();
        let payer_id = pubkey_rand();
        let mut payer_account = SolanaAccount::default();
        let mut rent_sysvar = rent_sysvar();
        let mut system_program_info = SolanaAccount::default();

        // create new uniswap_oracle account.
        do_process_instruction(
            initialize(
                &program_id,
                &uniswap_oracle_account_id,
                &moebius_program_id,
                &payer_id,
                token0,
                decimal0,
                token1,
                decimal1,
            )
            .unwrap(),
            vec![
                &mut payer_account,
                &mut uniswap_oracle_account,
                &mut system_program_info,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        let rand_amount0 = rand_bytes(32usize);
        let rand_amount1 = rand_bytes(32usize);
        let mut new_amount0 = [0u8; 32];
        let mut new_amount1 = [0u8; 32];
        new_amount0.copy_from_slice(rand_amount0.as_slice());
        new_amount1.copy_from_slice(rand_amount1.as_slice());
        let not_authority = pubkey_rand();
        assert_eq!(
            Err(UniswapOracleError::Unauthorized.into()),
            do_process_instruction(
                update_state(
                    &program_id,
                    &uniswap_oracle_account_id,
                    &not_authority,
                    token0,
                    new_amount0,
                    token1,
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
                token0,
                new_amount0,
                token1,
                new_amount1,
            )
            .unwrap(),
            vec![&mut authority, &mut uniswap_oracle_account],
        )
        .is_ok());

        let new_state = UniswapOracle::unpack(&uniswap_oracle_account.data).unwrap();

        assert_eq!(new_state.token0, token0);
        assert_eq!(new_state.amount0, new_amount0);
        assert_eq!(new_state.token1, token1);
        assert_eq!(new_state.amount1, new_amount1);
    }
}
