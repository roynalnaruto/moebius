# Setup
### Solana
* Set config and create keys
```shell
$ solana config set --url http://127.0.0.1:8899
$ solana-keygen new -o ~/.config/solana/uniswap_program.json
```
* Update Uniswap Program ID in appropriate places
```shell
# Log address
$ solana address --keypair ~/.config/solana/uniswap_program.json
# update in `solana/uniswap_oracle/program/src/lib.rs`
# solana_program::declare_id!("UniswapProgramBase58AddressGoesHere");
# update in `solana/uniswap_oracle/program/program-id.md`
```
* Note Moebius account's ID
```shell
$ solana address --keypair ~/.config/solana/moebius_account.json
```
* Build and deploy Uniswap Program
```shell
$ cd uniswap_oracle/program
$ cargo build-bpf
$ solana deploy uniswap_program.so ~/.config/solana/uniswap_program.json
```
* Initialize UNI-WETH account to store its pricefeed on Solana
```shell
# UNI address : 0x1f9840a85d5af5bf1d1762f925bdaddc4201f984
# WETH address: 0xc778417e063141139fce010982780140aa0cd5ab
$ cd solana/
$ cargo build
$ ./target/debug/uniswap_program initialize --token0 1f9840a85d5af5bf1d1762f925bdaddc4201f984 --decimal0 18 --token1 c778417e063141139fce010982780140aa0cd5ab --decimal1 18
```
The `initialize` instruction derives a deterministic address for the UNI-WETH
pair's pricefeed. The address is derived using the addresses `token0` and `token1`, which means, there will be just one Solana account (via Moebius) for
a specific Uniswap pair. The `initialize` instruction will also print out the
derived address, note that for the next step.

### Ethereum
Since we do not have a localnet deployment script for the entire Uniswap ecosystem, we will make use of the Ropsten testnet.

* Add the required environment variables (make sure the respective address is funded with rETH)
```shell
$ touch .env
# export INFURA_API_KEY=yourapikeygoeshere
# export ETH_PRIVATE_KEY=yourhexprivatekeygoeshere
$ source .env
```
* Edit the Uniswap program ID and Uniswap pair's account ID (in Solana) in the
`scripts/deploy-uniswap-oracle.js`
```javascript
// Solana program ID and account in the base58 format.
uniswapProgramId = "UniswapProgramBase58AddressGoesHere";
uniswapAccountId = "UniswapDerivedAccountBase58AddressGoeshere";
```
* [This step has been done](https://ropsten.etherscan.io/tx/0x8b81bb2815f47876c30a73d6c4f2d094663279e80d6c0194465de694115c8a9b) Deploy the `Moebius` contract to Ropsten
```shell
$ npx hardhat run scripts/deploy-moebius.js --network ropsten
```
* [This step has been done](https://ropsten.etherscan.io/tx/0x5ca876d1787b264d71ce00fee11f690987e0bb2ca9c07be87aca54aa9d0bc291) Deploy the `UniswapOracle` contract to Ropsten
```shell
$ npx hardhat run scripts/deploy-uniswap-oracle.js --network ropsten
```

_Note_: If re-using the already deployed contracts, you need not make any changes to
`scripts/update-uniswap-oracle.js`. On the other hand, if you re-deployed those
contracts in the above steps, you will need to edit the contract addresses in:
```javascript
// scripts/update-uniswap-oracle.js
const moebius = await hre.ethers.getContractAt("Moebius", "0x4f2a9ac3a70400636190e1df213fd7aa0bcf794d");
```
and
```javascript
// scripts/update-uniswap-oracle.js
const uniswapOracle = await hre.ethers.getContractAt("UniswapOracle", "0x20412cA3DA74560695529C7c5D34C1e766B52AeB");
```

* Run script to periodically broadcast appropriate tx
```shell
npx hardhat run scripts/update-uniswap-oracle.js --network ropsten
```

### Moebius Bridge
* Place ETH private key (hex format without `0x`) in `moebius-bridge/config/ropsten.key`
* Place Ropsten testnet configs in `moebius-bridge/config/ropsten.json`
```json
{
  "MoebiusContract": "0x4f2a9ac3a70400636190e1df213fd7aa0bcf794d",
  "EthRpcUrl": "https://ropsten.infura.io/v3/yourInfuraApiKeyGoesHere",
  "SolanaRpcUrl": "http://127.0.0.1:8899",
  "MoebiusAccount": "MoebiusAccountBase58AddressGoesHere"
}
```
* Run Moebius bridge
```shell
$ cd moebius-bridge/
$ ./target/debug/moebius-bridge \
      --config config/ropsten.json \
      --eth-private-key config/ropsten.key \
      --authority-key ~/.config/solana/id.json
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
    uniswapOracle(token0: "1f9840a85d5af5bf1d1762f925bdaddc4201f984", token1: "c778417e063141139fce010982780140aa0cd5ab") {
        amount0
        amount1
        priceToken0Token1
        priceToken1Token0
    }
}
```
