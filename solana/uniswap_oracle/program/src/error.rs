//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Uniswap oracle.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum UniswapOracleError {
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// The derived program account does not match the expected account.
    #[error("Derived program account mismatch")]
    DerivedAccountMismatch,
    /// This error is thrown if the invalid account is asked to be updated.
    #[error("Invalid account")]
    InvalidAccount,
    /// Invalid instruction.
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// The sender is not authorized to do the transaction.
    #[error("Unauthorized")]
    Unauthorized,
}

impl From<UniswapOracleError> for ProgramError {
    fn from(e: UniswapOracleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for UniswapOracleError {
    fn type_of() -> &'static str {
        "UniswapOracleError"
    }
}
