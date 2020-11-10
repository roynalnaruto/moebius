use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use moebius_derive::moebius_state;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[moebius_state]
pub struct AnotherState {
    id: bytes32,
    token: address,
    amount: uint256,
}
