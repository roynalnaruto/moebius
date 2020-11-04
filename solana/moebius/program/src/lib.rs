#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Moebius bridge

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("CFKmqRvfEH1JHKXMLVJe2mu4kFQUXb8UCS75LVjF9ZUt");
