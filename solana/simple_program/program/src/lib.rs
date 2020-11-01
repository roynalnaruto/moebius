
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Simple program

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "exclude_entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("46eMqP2zKxbu2frsX2fCsQ9tRdKa4sn54ryd5WeS1yG8");
