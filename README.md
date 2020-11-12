<p align="center"><img src="/assets/transparent-background.png"/></p>
Moebius is a cross-chain oracle that bridges on-chain Ethereum data to Solana.

## Overview
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

## Moebius-Uniswap
Using Moebius Bridge, the first use-case implemented was to bridge Uniswap time-weighted average prices ([TWAP](https://uniswap.org/docs/v2/core-concepts/oracles/)) from Ethereum contracts to Solana programs.

* A transaction that updated the Uniswap UNI/WETH average price on Ethereum [here](https://ropsten.etherscan.io/tx/0x6997d907e1c7626dbe840933857d2133237485e426abaacd98a6ae28a4905261)
* A transaction that updated the Uniswap UNI/WETH pricefeed state on Solana [here](https://explorer.solana.com/tx/4RDVwaeGAJJ2jarJZra7cAFFQfWEZk4iny4TwmdEVV9P446tayAHiwyuTBq5pPT27Jc9aDK93yeQewrYfcxiQARA?cluster=devnet)

#### Ethereum
The below contracts are deployed on the Ethereum's [Ropsten testnet](https://ropsten.etherscan.io/)

| Contract       | Address                                                                                                                       |
|----------------|-------------------------------------------------------------------------------------------------------------------------------|
| [Moebius](ethereum/contracts/Moebius.sol)        | [0x4f2A9aC3A70400636190e1df213Fd7Aa0BCF794d](https://ropsten.etherscan.io/address/0x4f2a9ac3a70400636190e1df213fd7aa0bcf794d) |
| [Uniswap Oracle](ethereum/contracts/UniswapOracle.sol) | [0x20412cA3DA74560695529C7c5D34C1e766B52AeB](https://ropsten.etherscan.io/address/0x20412cA3DA74560695529C7c5D34C1e766B52AeB) |

#### Solana
The below programs are deployed on Solana's [Devnet](https://explorer.solana.com/?cluster=devnet)

| Program/Account   | Address                                                                                                                                         |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| [Moebius](solana/moebius)           | [6kAHanNCT1LKFoMn3fBdyvJuvHLcWhLpJbTpbHpqRiG4](https://explorer.solana.com/address/6kAHanNCT1LKFoMn3fBdyvJuvHLcWhLpJbTpbHpqRiG4?cluster=devnet) |
| [Uniswap Program](solana/uniswap_oracle)   | [G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen](https://explorer.solana.com/address/G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen?cluster=devnet) |
| Trusted Moebius Authority | [BREEUNEkUnR7TidwEGptGDREGD3aVHu5Qv5Bvan9fjP6](https://explorer.solana.com/address/BREEUNEkUnR7TidwEGptGDREGD3aVHu5Qv5Bvan9fjP6?cluster=devnet) |

* Run script that periodically updates and consults the Uniswap oracle. Make sure the address you use has sufficient [Ropsten ETH](https://faucet.dimensions.network/) balance.
```shell
$ cd ethereum/
# export INFURA_API_KEY=your-infura-api-key
# export ETH_PRIVATE_KEY=your-hex-private-key-excluding-0x
$ npx hardhat run scripts/update-uniswap-oracle.js --network ropsten
```
* Run [Moebius Reporter](solana/reporter), a GraphQL server that exposes Uniswap pricefeed from Solana account.
```shell
$ cd solana/
$ ./target/debug/reporter
```
Visit [http://localhost:8080/graphiql](http://localhost:8080/graphiql) and query:
```
query {
    uniswapOracle(token0: "1f9840a85d5af5bf1d1762f925bdaddc4201f984", token1: "c778417e063141139fce010982780140aa0cd5ab") {
        token0
        decimal0
        amount0
        token1
        decimal1
        amount1
        priceToken0Token1
        priceToken1Token0
    }
}
```
where `1f9840a85d5af5bf1d1762f925bdaddc4201f984` is [UNI](https://ropsten.etherscan.io/token/0x1f9840a85d5af5bf1d1762f925bdaddc4201f984) and `c778417e063141139fce010982780140aa0cd5ab` is [WETH](https://ropsten.etherscan.io/token/0xc778417e063141139fce010982780140aa0cd5ab). If Moebius bridge is disabled, the reporter will respond with the most recently updated pricefeed.

* Use [Moebius API](solana/moebius-api) to fetch Uniswap pricefeed in your Solana programs.
```rust
//
// [dependencies]
// moebius-api = { git = "https://github.com/roynalnaruto/moebius/solana/moebius-api" }
//
// Instantiate API client
let api = moebius_api::MoebiusApi::new()
    .with_rpc_url(String::from("https://devnet.solana.com"));
// Fetch UNI-WETH pricefeed
let pricefeed = api.uniswap_oracle(
    "1f9840a85d5af5bf1d1762f925bdaddc4201f984",
    "c778417e063141139fce010982780140aa0cd5ab"
)?;
// UNI/WETH
let price_token0_token1 = pricefeed.price_token0_token1();
// WETH/UNI
let price_token1_token0 = pricefeed.price_token1_token0();
```

## Setup Guides
To setup Moebius and its components locally, follow:
* **Prerequisite** [setup guide](./PREREQUISITE-SETUP.md)
* **Simple Program** [setup and run guide](./SIMPLE-SETUP.md)
* **Uniswap Oracle** [setup and run guide](./UNISWAP-SETUP.md)
* **Create New Moebius-compatible Program** [setup guide](./NEW-PROGRAM-SETUP.md)
