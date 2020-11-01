//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Simple program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SimpleProgramError {
    /// Invalid instruction.
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// The sender is not authorized to do the transaction.
    #[error("Unauthorized")]
    Unauthorized,
}

impl From<SimpleProgramError> for ProgramError {
    fn from(e: SimpleProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SimpleProgramError {
    fn type_of() -> &'static str {
        "SimpleProgramError"
    }
}
