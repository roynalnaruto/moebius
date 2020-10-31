//! State transition types

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// Moebius data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moebius {
    /// Is `true` if this structure has been initialized.
    pub is_initialized: bool,
    /// Authority that can submit data over Moebius bridge.
    pub authority: Pubkey,
}
impl IsInitialized for Moebius {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for Moebius {}
impl Pack for Moebius {
    const LEN: usize = 33;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 33];
        let (is_initialized, authority) = array_refs![src, 1, 32];
        Ok(Moebius {
            is_initialized: is_initialized[0] == 1,
            authority: Pubkey::new_from_array(*authority),
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 33];
        let (is_initialized_dst, authority_dst) = mut_array_refs![dst, 1, 32];
        let &Moebius {
            is_initialized,
            ref authority,
        } = self;
        authority_dst.copy_from_slice(authority.as_ref());
        is_initialized_dst[0] = is_initialized as u8;
    }
}
