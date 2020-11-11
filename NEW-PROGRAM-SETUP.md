# Setup New Program
If you wish to bridge your data through Moebius, you would need a Solana program
whose instructions the Moebius program would invoke.

A new program can be setup by:
* Create new project `another-program` (We already have an [example](solana/another_program))
```
$ mkdir another-program && cd another-program
$ cargo new --lib program
```
* Create the [state module](solana/another_program/program/src/state.rs) that would
hold the state/data that is bridged. We provide easy to use procedural macros `#[moebius_state]` that would map Solidity types to Rust types and implement packing/unpacking for them. An example is given below:
```rust
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use moebius_derive::moebius_state;
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
// The `moebius_state` proc macro will map the below solidity types `bytes32`,
// `address`, `uint256`, `bool`, etc. to appropriate Rust types and also
// implement the `solana_program::program_pack::Pack` trait for this state.
#[moebius_state]
pub struct AnotherState {
    id: bytes32,
    token: address,
    amount: uint256,
    active: bool,
}
```
_Note_: If you are curious to see what this expands to, please use the [cargo-expand](https://github.com/dtolnay/cargo-expand)
crate. `cargo expand --lib state` should expand the implementation for you.

* Create the [instruction module](solana/another_program/program/src/instruction.rs)
that would be responsible for packing/unpacking instructions that can be passed
to `another_program`, and which will further be processed in the [processor module](solana/another_program/program/src/processor.rs). Again, you can make
use of the procedural macros `#[moebius_instruction]`:
```rust
use crate::error::AnotherError::*;
use moebius_derive::moebius_instruction;
use solana_program::{program_error::ProgramError, pubkey::Pubkey};
use std::mem::size_of;
// The `moebius_instruction` proc macro behaves similar to the above state macro.
// In addition to mapping and implementing `Pack` trait, `AnotherInstruction::Initialize`
// is also added another field `moebius_program_id`.
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
        active: bool,
    },
}
```
