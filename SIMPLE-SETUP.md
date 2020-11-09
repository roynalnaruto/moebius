# Setup
### Solana
* Set config and create keys
```shell
$ solana config set --url http://127.0.0.1:8899
$ solana-keygen new -o ~/.config/solana/simple_program.json
$ solana-keygen new -o ~/.config/solana/simple_program_account.json
```
* Update Simple Program ID in appropriate places
```shell
# Log address
$ solana address --keypair ~/.config/solana/simple_program.json
# update in `solana/simple_program/program/src/lib.rs`
# solana_program::declare_id!("SimpleProgramBase58AddressGoesHere");
# update in `solana/simple_program/program/program-id.md`
```
* Build and deploy Simple Program
```shell
$ cd simple_program/program
$ cargo build-bpf
$ solana deploy simple_program.so ~/.config/solana/simple_program.json
```
* Initialize Simple Program's account
```shell
$ cd solana/
$ cargo build
$ ./target/debug/simple_program initialize --account-keypair ~/.config/solana/simple_program_account.json
```
* Note Simple program's program ID and its state's account ID
```shell
$ solana address --keypair ~/.config/solana/simple_program.json
$ solana address --keypair ~/.config/solana/simple_program_account.json
```
* Note Moebius account's ID
```shell
$ solana address --keypair ~/.config/solana/moebius_account.json
```

### Ethereum
* Edit the program ID and account ID in `scripts/update_simple_program.js`
```javascript
// Solana program ID and account in the base58 format.
simpleProgramId = "SimpleProgramBase58AddressGoesHere";
simpleAccountId = "SimpleAccountBase58AddressGoesHere";
```
* Run script to periodically broadcast appropriate tx
```shell
$ npx hardhat run scripts/sample_script.js
```

### Moebius Bridge
* Update Moebius' Ethereum contract address in `config/localnet.json`
* Update Moebius' Solana account ID in `config/localnet.json`
* Run Moebius bridge
```shell
$ cd moebius-bridge/
$ ./target/debug/moebius-bridge --authority-key ~/.config/solana/id.json
```

### GraphQL Server
* Run GraphQL server to query simple data from Solana
```shell
$ cd solana/
$ ./target/debug/reporter
```
* Go to `http://localhost:8080/graphiql` in your browser
* Query simple data
```
query {
    simpleData {
        valBytes32
        valAddress
        valUint256
    }
}
```
