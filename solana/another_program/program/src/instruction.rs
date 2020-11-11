use crate::error::AnotherError::*;
use moebius_derive::moebius_instruction;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::mem::size_of;

#[moebius_instruction]
pub enum AnotherInstruction {
    Initialize {
        id: bytes32,
        token: address,
    },
    UpdateState {
        id: bytes32,
        token: address,
        amount: uint256,
    },
}
