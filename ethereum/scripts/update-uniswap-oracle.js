const hre = require("hardhat");
const Logger = require("pretty-logger");

const customConfig = {
  showMillis: true,
  showTimestamp: true,
  info: "gray",
  error: ["bgRed", "bold"],
  debug: "rainbow"
};

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  const log = new Logger(customConfig);

  const moebius = await hre.ethers.getContractAt("Moebius", "0x4f2a9ac3a70400636190e1df213fd7aa0bcf794d");
  await moebius.deployed();
  log.info("Moebius deployed at: ", moebius.address);

  const uniswapOracle = await hre.ethers.getContractAt("UniswapOracle", "0x20412cA3DA74560695529C7c5D34C1e766B52AeB");
  await uniswapOracle.deployed();
  log.info("UniswapOracle deployed at: ", uniswapOracle.address);

  while (true) {
    const data = uniswapOracle.interface.encodeFunctionData(
      "updateAndConsult", [
        "0xc778417e063141139fce010982780140aa0cd5ab",     // token    : weth
        hre.ethers.BigNumber.from("1000000000000000000"), // amountIn : 1 weth
      ]);

    const tx = await moebius.execute(uniswapOracle.address, data, {
      gasLimit: 120000,     // 120,000 gas
      gasPrice: 4000000000, // 4 gwei  gas price
    });
    log.info("updateAndConsult: ", tx.hash);

    await hre.ethers.provider.waitForTransaction(tx.hash);

    log.debug("sleeping...\n");
    await sleep(60000);
  }
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
