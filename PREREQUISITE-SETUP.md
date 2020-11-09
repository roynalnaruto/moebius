# Setup
### Solana
* Setup Solana CLI tools [ref](https://docs.solana.com/cli/install-solana-cli-tools)
```shell
$ sh -c "$(curl -sSfL https://release.solana.com/v1.4.4/install)"
$ solana --version
```
* Start a localnet Solana cluster
```shell
$ cd solana/
$ yarn install
$ npx solana-localnet update
$ npx solana-localnet up
```
* Set config and create keys
```shell
$ solana config set --url http://127.0.0.1:8899
$ solana-keygen new -o ~/.config/solana/id.json
$ solana-keygen new -o ~/.config/solana/moebius_program.json
$ solana-keygen new -o ~/.config/solana/moebius_account.json
```
* Update Moebius program ID in appropriate places
```shell
# Log address
$ solana address --keypair ~/.config/solana/moebius_program.json
# update in `solana/moebius/program/src/lib.rs`
# solana_program::declare_id!("MoebiusProgramBase58AddressGoesHere");
# update in `solana/moebius/program/program-id.md`
```
* Receive funds into default account
```shell
$ solana airdrop 10
```
* Build and deploy Moebius
```shell
$ cd moebius/program
$ cargo build-bpf
$ solana deploy moebius.so ~/.config/solana/moebius_program.json
```
* Initialize Moebius and Simple Program's accounts
```shell
$ cd solana/
$ cargo build
$ ./target/debug/moebius initialize --account-keypair ~/.config/solana/moebius_account.json
```

### Ethereum
* Setup dependencies
```shell
$ cd ethereum/
$ npm install
$ npm install -f ganache-cli
```
* Compile and test contracts
```shell
$ npx hardhat compile
$ npx hardhat test
```

### Moebius Bridge
* Build Rust bindings and bridge
```shell
$ cd moebius-bridge/
$ cargo build
```
