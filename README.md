# Moebius
Moebius is a bridge acts as a cross-chain oracle, bringing on-chain data from
Ethereum to Solana.

### Overview
Moebius is a program deployed on Solana that accepts update instructions from
the Moebius authority. The authority is simply a Solana-compatible keypair that
is listening for specific events on the Ethereum blockchain.

The update instruction received by Moebius is then invoked on the data-specific
Solana program, and should be successfully executed as long as the target
program follows Moebius-compatible structure and is capable of deserializing
the received data.

In order to enable a cross-chain oracle via Moebius, a user will need to:
* Deploy a target program on Solana, e.g. [simple_program](./solana/simple_program/program).
* Deploy a source contract on Ethereum, e.g. [SimpleContract](./ethereum/contracts/SimpleContract.sol).
* Broadcast a transaction to the [Moebius contract](./ethereum/contracts/Moebius.sol)'s `execute` function.

A successful execution of the above transaction would emit the event:
```
event MoebiusData(bytes32 _programId, bytes32 _accountId, bytes _packedData);
```
that the Moebius bridge is listening for.

An event with the above signature would be intercepted, parsed and then
broadcasted to the Moebius program deployed to Solana by a trusted Moebius
authority.

### Setup
#### Solana
* Setup Solana CLI tools [ref](https://docs.solana.com/cli/install-solana-cli-tools)
```
$ sh -c "$(curl -sSfL https://release.solana.com/v1.4.4/install)"
$ solana --version
```
* Start a localnet Solana cluster
```
$ cd solana/
$ yarn install
$ npx solana-localnet update
$ npx solana-localnet up
```
* Set config and create keys
```
$ solana config set --url http://127.0.0.1:8899
$ solana-keygen new -o ~/.config/solana/id.json
$ solana-keygen new -o ~/.config/solana/moebius_program.json
$ solana-keygen new -o ~/.config/solana/moebius_account.json
$ solana-keygen new -o ~/.config/solana/simple_program.json
$ solana-keygen new -o ~/.config/solana/simple_program_account.json
```
* Receive funds into default account
```
$ solana airdrop 10
```
* Test Moebius and Simple Program.
```
$ cargo test
```
* Build and deploy Moebius
```
$ cd moebius/program
$ cargo build-bpf
$ solana deploy moebius.so ~/.config/solana/moebius_program.json
```
* Build and deploy Simple Program
```
$ cd simple_program/program
$ cargo build-bpf
$ solana deploy simple_program.so ~/.config/solana/simple_program.json
```
* Initialize Moebius and Simple Program's accounts
```
$ cd solana/
$ cargo build
$ ./target/debug/moebius initialize --account-keypair ~/.config/solana/moebius_account.json
$ ./target/debug/simple_program initialize --account-keypair ~/.config/solana/simple_program_account.json
```

#### Ethereum
* Setup dependencies
```
$ cd ethereum/
$ npm install
$ npm install -f ganache-cli
```
* Compile and test contracts
```
$ npx hardhat compile
$ npx hardhat test
```
* Run script to periodically broadcast appropriate tx
```
$ npx hardhat run scripts/sample_script.js
```
_Note Moebius contract's address_

#### Moebius Bridge
* Update Moebius contract's address in `config/localnet.json`
* Build Rust bindings and bridge
```
$ cargo build
```
* Run Moebius bridge
```
$ ./target/debug/moebius-bridge
```
