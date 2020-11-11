# Moebius
Moebius is a cross-chain oracle that bridges on-chain Ethereum data to Solana.

### Overview
Moebius is a program deployed on Solana that accepts update instructions from
the Moebius authority. The authority key is loaded in Moebius Bridge and listen
for the Moebius events on the Ethereum blockchain. On noticing such an event,
the authority then broadcasts an update instruction to the Moebius program
deployed on Solana.

The update instruction received by Moebius then invokes an update instruction on
the target program and account. The update succeeds if the target program and
account follow Moebius-compatible structure, and are capable of decoding and
processing the instruction.

In order to enable a cross-chain oracle via Moebius, a user will need to:
* Deploy a target program on Solana, e.g. [simple_program](./solana/simple_program/program).
* Deploy a source contract on Ethereum, e.g. [SimpleContract](./ethereum/contracts/SimpleContract.sol).
* Broadcast a transaction to the [Moebius contract](./ethereum/contracts/Moebius.sol)'s `execute` function.

A successful execution of the above transaction would emit the event:
```
event MoebiusData(bytes32 _programId, bytes32 _accountId, bytes _packedData);
```
that the Moebius bridge is listening for.

An event with the above signature would be intercepted, parsed and broadcasted
to the Moebius program deployed to Solana by a trusted Moebius authority.

### Setup Guides
* **Prerequisite** [setup guide](./PREREQUISITE-SETUP.md)
* **Simple Program** [setup and run guide](./SIMPLE-SETUP.md)
* **Uniswap Oracle** [setup and run guide](./UNISWAP-SETUP.md)

### Setup New Program
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
