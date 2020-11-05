#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Moebius bridge

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("6kAHanNCT1LKFoMn3fBdyvJuvHLcWhLpJbTpbHpqRiG4");
