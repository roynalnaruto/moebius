use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum AnotherError {
    #[error("Invalid instruction")]
    InvalidInstruction,
}

impl From<AnotherError> for ProgramError {
    fn from(e: AnotherError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for AnotherError {
    fn type_of() -> &'static str {
        "AnotherError"
    }
}
