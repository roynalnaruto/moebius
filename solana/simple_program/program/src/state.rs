//! State transition types

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// SimpleProgram data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SimpleProgram {
    /// Is `true` if this structure has been initialized.
    pub is_initialized: bool,
    /// Authority that can update SimpleProgram data.
    pub authority: Pubkey,

    /// Bytes32 value from Ethereum.
    pub val_bytes32: [u8; 32],
    /// Address value from Ethereum.
    pub val_address: [u8; 20],
    /// Uint256 value from Ethereum.
    pub val_uint256: [u8; 32],
}
impl IsInitialized for SimpleProgram {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for SimpleProgram {}
impl Pack for SimpleProgram {
    const LEN: usize = 117;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 117];
        let (is_initialized, authority, val_bytes32, val_address, val_uint256) =
            array_refs![src, 1, 32, 32, 20, 32];
        Ok(SimpleProgram {
            is_initialized: is_initialized[0] == 1,
            authority: Pubkey::new_from_array(*authority),
            val_bytes32: *val_bytes32,
            val_address: *val_address,
            val_uint256: *val_uint256,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 117];
        let (is_initialized_dst, authority_dst, val_bytes32_dst, val_address_dst, val_uint256_dst) =
            mut_array_refs![dst, 1, 32, 32, 20, 32];
        let &SimpleProgram {
            is_initialized,
            ref authority,
            ref val_bytes32,
            ref val_address,
            ref val_uint256,
        } = self;
        authority_dst.copy_from_slice(authority.as_ref());
        is_initialized_dst[0] = is_initialized as u8;
        val_bytes32_dst.copy_from_slice(&val_bytes32[..]);
        val_address_dst.copy_from_slice(&val_address[..]);
        val_uint256_dst.copy_from_slice(&val_uint256[..]);
    }
}
