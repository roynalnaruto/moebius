<p align="center"><img src="/assets/transparent-background.png"/></p>
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

### Testnet Deployment
#### Ethereum
The below contracts are deployed on the Ethereum's [Ropsten testnet](https://ropsten.etherscan.io/)

| Contract       | Address                                                                                                                       |
|----------------|-------------------------------------------------------------------------------------------------------------------------------|
| Moebius        | [0x4f2A9aC3A70400636190e1df213Fd7Aa0BCF794d](https://ropsten.etherscan.io/address/0x4f2a9ac3a70400636190e1df213fd7aa0bcf794d) |
| Uniswap Oracle | [0x20412cA3DA74560695529C7c5D34C1e766B52AeB](https://ropsten.etherscan.io/address/0x20412cA3DA74560695529C7c5D34C1e766B52AeB) |

#### Solana
The below programs are deployed on Solana's [Devnet](https://explorer.solana.com/?cluster=devnet)

| Program/Account   | Address                                                                                                                                         |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------|
| Moebius           | [6kAHanNCT1LKFoMn3fBdyvJuvHLcWhLpJbTpbHpqRiG4](https://explorer.solana.com/address/6kAHanNCT1LKFoMn3fBdyvJuvHLcWhLpJbTpbHpqRiG4?cluster=devnet) |
| Uniswap Program   | [G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen](https://explorer.solana.com/address/G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen?cluster=devnet) |
| Moebius Authority | [BREEUNEkUnR7TidwEGptGDREGD3aVHu5Qv5Bvan9fjP6](https://explorer.solana.com/address/BREEUNEkUnR7TidwEGptGDREGD3aVHu5Qv5Bvan9fjP6?cluster=devnet) |

### Localnet Setup Guides
* **Prerequisite** [setup guide](./PREREQUISITE-SETUP.md)
* **Simple Program** [setup and run guide](./SIMPLE-SETUP.md)
* **Uniswap Oracle** [setup and run guide](./UNISWAP-SETUP.md)

### Create a New Moebius-compatible Program
Please follow this [setup guide](./NEW-PROGRAM-SETUP.md)
