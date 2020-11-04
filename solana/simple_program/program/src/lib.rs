#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Simple program

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("DRHSJwXrNNnqgQ8MrRqyBUwUAFfMgBufDdzbKFR1p8Ug");
