//! State transition types

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// UniswapOracle data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UniswapOracle {
    /// Is `true` if this structure has been initialized.
    pub is_initialized: bool,
    /// Authority that can update SimpleProgram data.
    pub authority: Pubkey,

    /// Address of the first token in the Uniswap pair.
    pub token0: [u8; 20],
    /// Amount of token0 tokens such that amount0*price0 = amount1*price1.
    pub amount0: [u8; 32],
    /// Address of the second token in the Uniswap pair.
    pub token1: [u8; 20],
    /// Amount of token1 tokens such that amount1*price1 = amount0*price0.
    pub amount1: [u8; 32],
}
impl IsInitialized for UniswapOracle {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Sealed for UniswapOracle {}
impl Pack for UniswapOracle {
    const LEN: usize = 137;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, 137];
        let (is_initialized, authority, token0, amount0, token1, amount1) =
            array_refs![src, 1, 32, 20, 32, 20, 32];
        Ok(UniswapOracle {
            is_initialized: is_initialized[0] == 1,
            authority: Pubkey::new_from_array(*authority),
            token0: *token0,
            amount0: *amount0,
            token1: *token1,
            amount1: *amount1,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, 137];
        let (is_initialized_dst, authority_dst, token0_dst, amount0_dst, token1_dst, amount1_dst) =
            mut_array_refs![dst, 1, 32, 20, 32, 20, 32];
        let &UniswapOracle {
            is_initialized,
            ref authority,
            ref token0,
            ref amount0,
            ref token1,
            ref amount1,
        } = self;
        authority_dst.copy_from_slice(authority.as_ref());
        is_initialized_dst[0] = is_initialized as u8;
        token0_dst.copy_from_slice(&token0[..]);
        amount0_dst.copy_from_slice(&amount0[..]);
        token1_dst.copy_from_slice(&token1[..]);
        amount1_dst.copy_from_slice(&amount1[..])
    }
}
