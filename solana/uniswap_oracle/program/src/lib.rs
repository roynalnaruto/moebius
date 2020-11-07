#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Uniswap oracle

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen");
